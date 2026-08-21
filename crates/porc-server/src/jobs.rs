//! Infra de jobs: tudo que demora, pode falhar no meio e precisa ser cancelável.
//!
//! Nasce aqui para o clone (Bloco C) e é reaproveitada inteira por fetch, push, merge e rebase
//! (Blocos F e G). Três decisões governam o desenho:
//!
//! **O estado do job vive no servidor, não na aba.** `GET /api/v1/jobs/{id}` devolve o último
//! estado completo — progresso, log recente, resultado. É isso que faz recarregar a página no
//! meio de um clone de 2 GB não perder nada: o WebSocket é o caminho rápido, não a memória.
//!
//! **Cancelar é avisar todo mundo de uma vez.** Um `CancellationToken` por job, compartilhado
//! pelo processo, pelo leitor de stderr e pelo watchdog. Sem isso, cancelar viraria três
//! caminhos separados que se desencontram.
//!
//! **A limpeza é registrada antes de começar, não depois de falhar.** Um job que morre no meio
//! (ou cujo processo o SO matou) precisa saber o que desfazer sem ter chegado ao fim. E a
//! limpeza é sempre condicional a *nós* termos criado a coisa — nunca apagamos pasta do usuário.

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

/// Teto de jobs rodando ao mesmo tempo. Cada um pode ser um `git` falando com a rede; passar
/// disso não acelera nada e transforma um clique repetido em ataque de negação de serviço
/// contra a própria máquina do usuário.
pub const MAX_RUNNING: usize = 8;

/// Quantas linhas de log o servidor guarda por job. O suficiente para uma aba que reconecta
/// entender onde as coisas estão; não é um arquivo de log.
const LOG_TAIL: usize = 200;

/// Fôlego do canal de eventos. Um cliente lento que passe disso recebe um `resync` e recarrega
/// o estado por HTTP — é mais barato do que segurar histórico ilimitado em memória.
const EVENT_BUFFER: usize = 512;

const ID_BYTES: usize = 8;

/// Quanto o encerramento espera pelos jobs em andamento antes de desistir.
const SHUTDOWN_GRACE: std::time::Duration = std::time::Duration::from_secs(6);

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    /// Fase legível ("recebendo objetos", "resolvendo deltas").
    pub phase: String,
    /// 0.0–1.0 quando dá para saber. `None` é fase indeterminada — e a UI tem que saber
    /// desenhar isso, porque o git passa boa parte do clone sem total conhecido.
    pub fraction: Option<f32>,
    /// Uma linha curta com os números crus, para quem quiser olhar.
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JobState {
    Running,
    Done,
    Error,
    Cancelled,
}

impl JobState {
    pub fn is_final(self) -> bool {
        !matches!(self, JobState::Running)
    }
}

/// Um pedido de senha em aberto.
///
/// Vive **no snapshot**, e não só no evento do WebSocket, porque senão uma aba recarregada no
/// meio de um clone por SSH ficaria olhando uma barra parada até o pedido expirar — o job
/// estaria esperando uma resposta que ninguém sabe mais que foi pedida.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingPrompt {
    pub prompt_id: String,
    pub prompt: String,
}

/// O estado completo de um job, que é o mesmo objeto no `GET` e no evento `job.done`.
///
/// Um formato só para os dois caminhos: se o WebSocket cair, o cliente pede por HTTP e recebe
/// exatamente o que teria recebido pelo socket.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSnapshot {
    pub job_id: String,
    pub kind: String,
    pub state: JobState,
    pub progress: Option<Progress>,
    pub log: Vec<String>,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    /// Mensagem legível quando `state` é `error`. Nunca stderr cru.
    pub message: Option<String>,
    /// A próxima coisa a fazer. `None` quando não há conselho honesto a dar — inventar um
    /// mandaria o usuário para o lugar errado.
    pub action: Option<String>,
    /// Pedido de senha esperando resposta. `None` é o caso normal.
    pub pending_prompt: Option<PendingPrompt>,
    /// Resultado do job. O formato depende do tipo — o `clone` devolve o repositório aberto.
    pub result: Option<serde_json::Value>,
}

/// O que desfazer se o job não chegar ao fim.
#[derive(Debug, Clone)]
pub enum Cleanup {
    /// Remover um diretório **que nós criamos**. O caminho é guardado no momento da criação;
    /// pasta preexistente nunca entra aqui, e é essa distinção que impede o cancelamento de um
    /// clone de apagar o trabalho de alguém.
    RemoveCreatedDir(PathBuf),
}

impl Cleanup {
    fn run(&self) {
        match self {
            Cleanup::RemoveCreatedDir(path) => {
                match std::fs::remove_dir_all(path) {
                    Ok(()) => tracing::info!(path = %path.display(), "pasta parcial removida"),
                    // Já não existir é o estado que queríamos.
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                    Err(err) => {
                        tracing::warn!(%err, path = %path.display(), "não consegui remover a pasta parcial");
                    }
                }
            }
        }
    }
}

/// Mensagens que o servidor manda pelo WebSocket.
///
/// Um socket só, multiplexado por `topic` — não um por assunto. Um ponto para autenticar,
/// validar `Origin` e reconectar.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "pong")]
    Pong,
    /// O cliente ficou para trás e perdeu eventos. Ele reconsulta o estado por HTTP em vez de
    /// receber um histórico que o servidor teria que guardar para sempre.
    #[serde(rename = "resync")]
    Resync,
    #[serde(rename = "job.progress", rename_all = "camelCase")]
    JobProgress { job_id: String, progress: Progress },
    #[serde(rename = "job.log", rename_all = "camelCase")]
    JobLog { job_id: String, line: String },
    /// `Box` porque o snapshot é três vezes maior que qualquer outra variante, e esta enum é
    /// **clonada para cada assinante** do canal de eventos. No JSON não muda nada.
    #[serde(rename = "job.done")]
    JobDone { job: Box<JobSnapshot> },
    #[serde(rename = "job.error", rename_all = "camelCase")]
    JobError { job_id: String, message: String },
    /// O git (ou o ssh) está pedindo uma senha e o job está parado até alguém responder.
    #[serde(rename = "job.askpass", rename_all = "camelCase")]
    JobAskpass {
        job_id: String,
        prompt_id: String,
        /// O texto que o ssh escreveria no terminal — inclui o caminho da chave.
        prompt: String,
    },
    /// O pedido expirou ou o job morreu: a UI tira o prompt da tela.
    #[serde(rename = "job.askpassClosed", rename_all = "camelCase")]
    JobAskpassClosed { job_id: String, prompt_id: String },
}

impl ServerMessage {
    /// Assunto ao qual a mensagem pertence. O cliente assina os que lhe interessam.
    pub fn topic(&self) -> &'static str {
        match self {
            ServerMessage::Ready | ServerMessage::Pong | ServerMessage::Resync => "control",
            ServerMessage::JobProgress { .. }
            | ServerMessage::JobLog { .. }
            | ServerMessage::JobDone { .. }
            | ServerMessage::JobError { .. }
            | ServerMessage::JobAskpass { .. }
            | ServerMessage::JobAskpassClosed { .. } => "job",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JobsError {
    #[error("job não encontrado")]
    Unknown,
    #[error("já há {MAX_RUNNING} operações em andamento; espere uma terminar")]
    TooMany,
    #[error("não consegui gerar o identificador do job")]
    Id,
}

struct Record {
    snapshot: JobSnapshot,
    cancel: CancellationToken,
    cleanup: Vec<Cleanup>,
}

pub struct Jobs {
    records: RwLock<HashMap<String, Record>>,
    events: broadcast::Sender<ServerMessage>,
    /// Contador de pedidos de senha. Só precisa ser único dentro do processo.
    prompt_seq: std::sync::atomic::AtomicU64,
}

impl Default for Jobs {
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
            events: broadcast::channel(EVENT_BUFFER).0,
            prompt_seq: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl Jobs {
    pub fn subscribe(&self) -> broadcast::Receiver<ServerMessage> {
        self.events.subscribe()
    }

    pub fn next_prompt_seq(&self) -> u64 {
        self.prompt_seq
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Publica um pedido de senha e o anota no log do job.
    ///
    /// O log guarda **o prompt**, nunca a resposta: saber que o ssh pediu a passphrase da chave
    /// tal é diagnóstico; a passphrase em si não pode existir fora do socket.
    pub fn publish_askpass(&self, job_id: String, prompt_id: String, prompt: String) {
        self.with_record(&job_id, |record| {
            if record.snapshot.log.len() == LOG_TAIL {
                record.snapshot.log.remove(0);
            }
            record.snapshot.log.push(format!("senha pedida: {prompt}"));

            record.snapshot.pending_prompt = Some(PendingPrompt {
                prompt_id: prompt_id.clone(),
                prompt: prompt.clone(),
            });
        });

        self.publish(ServerMessage::JobAskpass {
            job_id,
            prompt_id,
            prompt,
        });
    }

    /// Fecha o pedido — respondido, expirado ou cancelado, tanto faz. O que importa é que ninguém
    /// fique olhando um campo de senha que já não leva a lugar nenhum.
    pub fn publish_askpass_closed(&self, job_id: String, prompt_id: String) {
        self.with_record(&job_id, |record| {
            let same = record
                .snapshot
                .pending_prompt
                .as_ref()
                .is_some_and(|pending| pending.prompt_id == prompt_id);

            if same {
                record.snapshot.pending_prompt = None;
            }
        });

        self.publish(ServerMessage::JobAskpassClosed { job_id, prompt_id });
    }

    /// Publica um evento. Ninguém escutando não é erro — é o caso normal de aba fechada.
    fn publish(&self, message: ServerMessage) {
        let _ = self.events.send(message);
    }

    /// Cria o job e devolve a alça com que a task o alimenta.
    ///
    /// O job já nasce `running` e visível no `GET`: a UI mostra "começando…" no mesmo instante
    /// em que recebe o `202`, sem uma janela em que o id existe e o job não.
    pub fn create(self: &Arc<Self>, kind: &str) -> Result<JobHandle, JobsError> {
        let mut records = self.records.write().expect("jobs envenenado");

        let running = records
            .values()
            .filter(|record| record.snapshot.state == JobState::Running)
            .count();
        if running >= MAX_RUNNING {
            return Err(JobsError::TooMany);
        }

        let job_id = new_id()?;
        let cancel = CancellationToken::new();

        records.insert(
            job_id.clone(),
            Record {
                snapshot: JobSnapshot {
                    job_id: job_id.clone(),
                    kind: kind.to_owned(),
                    state: JobState::Running,
                    progress: None,
                    log: Vec::new(),
                    started_at: now_ms(),
                    finished_at: None,
                    message: None,
                    action: None,
                    pending_prompt: None,
                    result: None,
                },
                cancel: cancel.clone(),
                cleanup: Vec::new(),
            },
        );

        Ok(JobHandle {
            job_id,
            cancel,
            jobs: self.clone(),
        })
    }

    pub fn snapshot(&self, job_id: &str) -> Option<JobSnapshot> {
        self.records
            .read()
            .expect("jobs envenenado")
            .get(job_id)
            .map(|record| record.snapshot.clone())
    }

    /// Mais recentes primeiro.
    pub fn list(&self) -> Vec<JobSnapshot> {
        let records = self.records.read().expect("jobs envenenado");

        let mut jobs: Vec<_> = records
            .values()
            .map(|record| record.snapshot.clone())
            .collect();
        jobs.sort_by_key(|job| std::cmp::Reverse(job.started_at));

        jobs
    }

    /// Pede o cancelamento e volta na hora.
    ///
    /// Quem finaliza é a task do job: ela vê o token, para o processo, roda a limpeza e publica
    /// o estado final. Marcar `cancelled` aqui mentiria enquanto o `git` ainda estivesse vivo —
    /// e é justamente durante essa janela que a pasta parcial ainda existe.
    pub fn cancel(&self, job_id: &str) -> Result<(), JobsError> {
        let records = self.records.read().expect("jobs envenenado");
        let record = records.get(job_id).ok_or(JobsError::Unknown)?;

        if record.snapshot.state.is_final() {
            return Ok(());
        }

        tracing::info!(job_id, kind = %record.snapshot.kind, "cancelamento pedido");
        record.cancel.cancel();

        Ok(())
    }

    /// Cancela tudo que estiver rodando e espera a limpeza acontecer.
    ///
    /// Chamado no encerramento. Sem isto, fechar o porcelain no meio de um clone deixaria a
    /// pasta parcial para trás: a task morre junto com o runtime e a limpeza registrada nunca
    /// roda. É o mesmo caminho do botão de cancelar, só que para todos de uma vez.
    pub async fn shutdown(&self) {
        let running: Vec<String> = self
            .list()
            .into_iter()
            .filter(|job| !job.state.is_final())
            .map(|job| job.job_id)
            .collect();

        if running.is_empty() {
            return;
        }

        tracing::info!(jobs = running.len(), "cancelando o que está em andamento");
        for job_id in &running {
            let _ = self.cancel(job_id);
        }

        // Teto: o `git` tem 2s de graça antes do `SIGKILL`, e a limpeza é um `remove_dir_all`.
        // Passando disso, sair é melhor do que segurar o encerramento para sempre.
        let deadline = std::time::Instant::now() + SHUTDOWN_GRACE;
        while std::time::Instant::now() < deadline {
            if running
                .iter()
                // Job que sumiu do registry conta como terminado — não há o que esperar.
                .all(|job_id| self.snapshot(job_id).is_none_or(|job| job.state.is_final()))
            {
                return;
            }

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        tracing::warn!("alguns jobs não terminaram a tempo; pode ter sobrado pasta parcial");
    }

    fn with_record<T>(&self, job_id: &str, f: impl FnOnce(&mut Record) -> T) -> Option<T> {
        self.records
            .write()
            .expect("jobs envenenado")
            .get_mut(job_id)
            .map(f)
    }
}

fn new_id() -> Result<String, JobsError> {
    let mut bytes = [0u8; ID_BYTES];
    getrandom::fill(&mut bytes).map_err(|_| JobsError::Id)?;

    Ok(bytes.iter().fold(String::new(), |mut hex, byte| {
        use std::fmt::Write;
        write!(hex, "{byte:02x}").expect("String nunca falha em write!");
        hex
    }))
}

/// A alça que a task do job usa para reportar.
///
/// Os métodos finais (`done`, `fail`, `cancelled`) consomem a alça: um job só termina uma vez, e
/// o tipo é quem garante isso em vez de um `if` esquecido no meio da task.
pub struct JobHandle {
    pub job_id: String,
    pub cancel: CancellationToken,
    jobs: Arc<Jobs>,
}

impl JobHandle {
    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    /// Registra o que desfazer. **Antes** de começar, sempre.
    pub fn add_cleanup(&self, cleanup: Cleanup) {
        self.jobs.with_record(&self.job_id, |record| {
            record.cleanup.push(cleanup);
        });
    }

    pub fn progress(&self, progress: Progress) {
        self.jobs.with_record(&self.job_id, |record| {
            record.snapshot.progress = Some(progress.clone());
        });

        self.jobs.publish(ServerMessage::JobProgress {
            job_id: self.job_id.clone(),
            progress,
        });
    }

    pub fn log(&self, line: impl Into<String>) {
        let line = line.into();

        self.jobs.with_record(&self.job_id, |record| {
            if record.snapshot.log.len() == LOG_TAIL {
                record.snapshot.log.remove(0);
            }
            record.snapshot.log.push(line.clone());
        });

        self.jobs.publish(ServerMessage::JobLog {
            job_id: self.job_id.clone(),
            line,
        });
    }

    pub fn done(self, result: serde_json::Value) {
        self.finish(JobState::Done, None, Some(result));
    }

    /// `message` é texto para humano. O stderr cru vai para o log do job, não para cá.
    pub fn fail(self, message: impl Into<String>) {
        self.fail_with(message, None);
    }

    /// Falha com a próxima coisa a fazer junto.
    pub fn fail_with(self, message: impl Into<String>, action: Option<String>) {
        let message = message.into();

        self.jobs.publish(ServerMessage::JobError {
            job_id: self.job_id.clone(),
            message: message.clone(),
        });

        self.finish_with(JobState::Error, Some(message), action, None);
    }

    /// Encerra por cancelamento, rodando a limpeza registrada.
    pub fn cancelled(self) {
        self.finish(JobState::Cancelled, None, None);
    }

    fn finish(self, state: JobState, message: Option<String>, result: Option<serde_json::Value>) {
        self.finish_with(state, message, None, result);
    }

    fn finish_with(
        self,
        state: JobState,
        message: Option<String>,
        action: Option<String>,
        result: Option<serde_json::Value>,
    ) {
        // A limpeza sai de dentro do lock e roda fora dele: `remove_dir_all` numa árvore grande
        // pode demorar, e segurar o lock do registry por isso travaria toda a UI.
        let (cleanup, snapshot) = self
            .jobs
            .with_record(&self.job_id, |record| {
                record.snapshot.state = state;
                record.snapshot.finished_at = Some(now_ms());
                record.snapshot.message = message;
                record.snapshot.action = action;
                record.snapshot.result = result;
                // Job encerrado não tem mais o que perguntar — inclusive quando encerrou
                // *porque* ninguém respondeu.
                record.snapshot.pending_prompt = None;

                let cleanup = if state == JobState::Done {
                    // Deu certo: não há o que desfazer, e a lista some para ninguém rodá-la
                    // depois por engano.
                    record.cleanup.clear();
                    Vec::new()
                } else {
                    std::mem::take(&mut record.cleanup)
                };

                (cleanup, record.snapshot.clone())
            })
            .unzip();

        for action in cleanup.into_iter().flatten() {
            action.run();
        }

        if let Some(job) = snapshot {
            tracing::info!(job_id = %self.job_id, ?state, "job encerrado");
            self.jobs
                .publish(ServerMessage::JobDone { job: Box::new(job) });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ciclo_de_vida_e_eventos() {
        let jobs = Arc::new(Jobs::default());
        let mut events = jobs.subscribe();

        let handle = jobs.create("teste").unwrap();
        let job_id = handle.job_id.clone();

        assert_eq!(jobs.snapshot(&job_id).unwrap().state, JobState::Running);

        handle.log("primeira linha");
        handle.progress(Progress {
            phase: "contando".to_owned(),
            fraction: Some(0.5),
            detail: None,
        });
        handle.done(serde_json::json!({ "ok": true }));

        assert!(matches!(
            events.recv().await.unwrap(),
            ServerMessage::JobLog { .. }
        ));
        assert!(matches!(
            events.recv().await.unwrap(),
            ServerMessage::JobProgress { .. }
        ));
        assert!(matches!(
            events.recv().await.unwrap(),
            ServerMessage::JobDone { .. }
        ));

        let snapshot = jobs.snapshot(&job_id).unwrap();
        assert_eq!(snapshot.state, JobState::Done);
        assert_eq!(snapshot.log, ["primeira linha"]);
        assert!(snapshot.finished_at.is_some());
    }

    #[tokio::test]
    async fn cancelar_dispara_o_token_e_a_limpeza_roda_no_fim() {
        let dir = std::env::temp_dir().join("porc-job-cleanup");
        std::fs::create_dir_all(&dir).unwrap();

        let jobs = Arc::new(Jobs::default());
        let handle = jobs.create("teste").unwrap();
        let job_id = handle.job_id.clone();

        handle.add_cleanup(Cleanup::RemoveCreatedDir(dir.clone()));

        jobs.cancel(&job_id).unwrap();
        assert!(handle.is_cancelled());
        // Cancelar não finaliza sozinho: quem termina é a task.
        assert_eq!(jobs.snapshot(&job_id).unwrap().state, JobState::Running);
        assert!(dir.exists(), "a pasta só some quando o job encerra");

        handle.cancelled();
        assert_eq!(jobs.snapshot(&job_id).unwrap().state, JobState::Cancelled);
        assert!(!dir.exists(), "a limpeza registrada tem que ter rodado");
    }

    #[tokio::test]
    async fn sucesso_nao_roda_limpeza() {
        let dir = std::env::temp_dir().join("porc-job-sem-limpeza");
        std::fs::create_dir_all(&dir).unwrap();

        let jobs = Arc::new(Jobs::default());
        let handle = jobs.create("teste").unwrap();
        handle.add_cleanup(Cleanup::RemoveCreatedDir(dir.clone()));
        handle.done(serde_json::Value::Null);

        assert!(dir.exists(), "job que deu certo não desfaz o que fez");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn teto_de_jobs_concorrentes() {
        let jobs = Arc::new(Jobs::default());

        let handles: Vec<_> = (0..MAX_RUNNING)
            .map(|_| jobs.create("teste").unwrap())
            .collect();

        assert!(matches!(jobs.create("teste"), Err(JobsError::TooMany)));

        // Terminar um abre vaga.
        handles
            .into_iter()
            .next()
            .unwrap()
            .done(serde_json::Value::Null);
        assert!(jobs.create("teste").is_ok());
    }
}
