//! Shell-out para o `git` do sistema.
//!
//! Metade das operações do porcelain não passa pelo libgit2, e não é preguiça: `fetch`, `push`
//! e `clone` precisam do credential helper, do SSH agent e do LFS que o usuário já configurou;
//! `merge`, `rebase` e `cherry-pick` precisam disparar os hooks dele e deixar em disco o estado
//! que o `git` do terminal entende. Reimplementar isso seria construir um segundo git — pior.
//!
//! **As regras abaixo valem para todo shell-out, sem exceção**, e é por isso que ninguém monta
//! um `Command` na mão: todo mundo passa por [`command`].
//!
//! - `--no-optional-locks`, para uma leitura nossa nunca disputar o `index.lock` com o git do
//!   usuário;
//! - `GIT_TERMINAL_PROMPT=0`, senão um pedido de senha inesperado trava o job para sempre —
//!   não há terminal para respondê-lo;
//! - `LC_ALL=C`, para o stderr ser estável. O mapeamento de erro para português (Passo 34) lê
//!   texto em inglês; com a locale do usuário, "Permission denied" pode chegar traduzido;
//! - **grupo de processos próprio**, para cancelar matar o grupo inteiro. `git clone` é um pai
//!   que dispara `git-remote-https` e `ssh`; matar só o pai deixa os filhos órfãos segurando a
//!   rede;
//! - `stdin` fechado, para nada nunca ficar esperando digitação.
//!
//! Override de config é sempre efêmero, via `git -c chave=valor`. **Nunca** `git config
//! --global` sem o usuário pedir explicitamente na UI.

pub mod apply;
pub mod branch;
pub mod clone;
pub mod commit;
pub mod discard;
pub mod error;
pub mod init;
pub mod path_filter;
pub mod stage;
pub mod status;

use std::{
    path::Path,
    process::{ExitStatus, Stdio},
    time::Duration,
};

use tokio::{io::AsyncReadExt, process::Command};
use tokio_util::sync::CancellationToken;

/// Nome do executável. Não é caminho absoluto de propósito: o usuário pode ter um `git` mais
/// novo antes no `PATH`, e é o dele que queremos.
const GIT: &str = "git";

/// Onde o helper de askpass encontra o socket. **Não** é o segredo — é o caminho de um socket
/// unix efêmero, com dono e permissão nossos.
pub const ASKPASS_SOCKET_ENV: &str = "PORC_ASKPASS_SOCKET";

/// Como o git e o ssh pedem segredo quando não há terminal para responder.
///
/// A passphrase **nunca** aparece em `argv` (que qualquer `ps` mostra) nem em disco. O que vai
/// no ambiente é o caminho de um socket unix efêmero; quem pergunta é o próprio `porc`, rodando
/// em modo helper, e quem responde é o processo que já tem a passphrase em memória.
#[derive(Debug, Clone)]
pub struct Askpass {
    /// Binário que o git executa — o próprio `porc`, em modo `askpass`.
    pub helper: std::path::PathBuf,
    pub socket: std::path::PathBuf,
}

impl Askpass {
    pub fn apply(&self, command: &mut Command) {
        command
            // `GIT_ASKPASS` cobre credencial de HTTPS; `SSH_ASKPASS`, a passphrase da chave.
            .env("GIT_ASKPASS", &self.helper)
            .env("SSH_ASKPASS", &self.helper)
            // OpenSSH ≥ 8.4. Sem isto o ssh só chama o askpass quando não enxerga tty nenhum, e
            // o nosso processo pode ter herdado um do terminal em que o `porc` foi lançado.
            .env("SSH_ASKPASS_REQUIRE", "force")
            // O ssh antigo checava `DISPLAY` para decidir se havia askpass gráfico.
            .env("DISPLAY", ":0")
            .env(ASKPASS_SOCKET_ENV, &self.socket);
    }
}

/// Teto para comandos locais (`init`, `status`, `rev-parse`). Trinta segundos é absurdo para
/// qualquer um deles — se estourar, alguma coisa travou e matar é melhor do que esperar.
///
/// Comandos de rede **não** usam este teto: um clone legítimo de um repositório grande demora
/// mais que isso. Lá o relógio é de inatividade, contra o progresso (Passo 31).
pub const LOCAL_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("não encontrei o git no sistema. instale o git 2.30 ou mais novo e tente de novo")]
    NotFound,
    #[error("o git não respondeu em {}s e foi encerrado", .0.as_secs())]
    Timeout(Duration),
    #[error("operação cancelada")]
    Cancelled,
    #[error("não consegui executar o git")]
    Spawn(#[source] std::io::Error),
    #[error("o git falhou")]
    Failed(Failure),
}

/// Um `git` que rodou e saiu com erro.
///
/// O `stderr` cru fica guardado, mas **nunca** é a mensagem que o usuário lê: o Passo 34 o
/// traduz para uma frase em português com ação sugerida, e o cru vira o "ver detalhes".
#[derive(Debug)]
pub struct Failure {
    pub status: ExitStatus,
    pub stderr: String,
    /// O que ele escreveu no stdout antes de desistir. Quase sempre vazio — mas o `git commit`
    /// põe "nothing to commit" **ali**, não no stderr, e sem isto a única informação útil da
    /// falha se perderia.
    pub stdout: String,
}

#[derive(Debug)]
pub struct Output {
    pub stdout: Vec<u8>,
    pub stderr: String,
}

/// Monta um `git` com as regras da casa já aplicadas.
///
/// `repo` vira `-C <path>`, que é mais seguro que `current_dir`: não depende do cwd do processo,
/// que é global e compartilhado por todas as tasks.
pub fn command(repo: Option<&Path>) -> Command {
    let mut command = Command::new(GIT);

    command
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // `kill_on_drop` é a rede de segurança: se a task que espera o filho for cancelada, o
        // processo não fica para trás. O `killpg` do timeout continua sendo o caminho principal.
        .kill_on_drop(true);

    // Líder do próprio grupo (`setsid` em espírito): o pgid passa a ser o pid do filho, e é isso
    // que torna possível matar a árvore inteira depois.
    #[cfg(unix)]
    command.process_group(0);

    command.arg("--no-optional-locks");

    if let Some(repo) = repo {
        command.arg("-C").arg(repo);
    }

    command
}

/// Quanto se espera entre o `SIGTERM` e desistir. Tempo para o git remover `index.lock` e os
/// temporários dele — depois disso o `kill_on_drop` encerra à força.
const GRACE: Duration = Duration::from_secs(2);

/// Roda até o fim, com teto de tempo. Estourar o teto mata o **grupo**, não só o filho.
pub async fn run(command: Command, timeout: Duration) -> Result<Output, ExecError> {
    run_inner(command, timeout, None).await
}

/// O mesmo que [`run`], mas alimentando o `stdin` do git com um texto.
///
/// É como o patch parcial chega ao `git apply --cached` (Passo 50): por pipe, nunca por arquivo
/// temporário — um patch em disco é uma janela em que outro processo pode lê-lo ou trocá-lo, e
/// um arquivo a mais para limpar quando o comando morre no meio.
///
/// A escrita vai numa task à parte: com o `stdin` cheio e ninguém lendo o `stdout`, escrever e
/// esperar na mesma task se trancaria uma na outra.
pub async fn run_with_input(
    mut command: Command,
    timeout: Duration,
    input: Vec<u8>,
) -> Result<Output, ExecError> {
    // O `command` desta casa fecha o stdin por padrão (regra da casa: nada espera digitação).
    // Aqui a entrada é nossa, não do usuário, e é justamente por isso que dá para abri-la.
    command.stdin(Stdio::piped());
    run_inner(command, timeout, Some(input)).await
}

async fn run_inner(
    mut command: Command,
    timeout: Duration,
    input: Option<Vec<u8>>,
) -> Result<Output, ExecError> {
    let mut child = command.spawn().map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => ExecError::NotFound,
        _ => ExecError::Spawn(err),
    })?;

    // Guardado antes do `wait_with_output`, que consome o `Child`.
    let pid = child.id();

    if let Some(input) = input {
        let mut stdin = child
            .stdin
            .take()
            .expect("`run_with_input` acabou de pôr o stdin em pipe");

        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;

            // Falhar aqui é o git ter fechado o pipe antes de ler tudo — patch recusado, por
            // exemplo. Quem conta essa história é o status de saída, que o `run_inner` já lê;
            // um erro daqui só duplicaria a mesma notícia com palavras piores.
            let _ = stdin.write_all(&input).await;
            // O fecho é o EOF que faz o `git apply` parar de esperar mais patch.
            let _ = stdin.shutdown().await;
        });
    }

    // `wait_with_output` coleta stdout e stderr *enquanto* espera. Com os dois em `piped`, só
    // esperar o status deixaria um pipe cheio travar o git para sempre — e aí o teto de tempo
    // estaria disfarçando um deadlock em vez de cobrir um comando lento.
    let waiting = child.wait_with_output();
    tokio::pin!(waiting);

    // `select!` e não `timeout(…)`: o futuro precisa continuar vivo depois do estouro, para dar
    // ao git a chance de sair sozinho depois do `SIGTERM`. Envolver em `timeout` derrubaria o
    // futuro na hora, e o `kill_on_drop` mandaria `SIGKILL` sem período de graça nenhum.
    let finished = tokio::select! {
        output = &mut waiting => Some(output),
        _ = tokio::time::sleep(timeout) => None,
    };

    let output = match finished {
        Some(output) => output.map_err(ExecError::Spawn)?,
        None => {
            if let Some(pid) = pid {
                kill_group(pid);
            }
            // Se ele sair na graça, ótimo; se não, o `kill_on_drop` fecha a conta ao sair daqui.
            let _ = tokio::time::timeout(GRACE, &mut waiting).await;

            return Err(ExecError::Timeout(timeout));
        }
    };

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() {
        return Err(ExecError::Failed(Failure {
            status: output.status,
            stderr,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        }));
    }

    Ok(Output {
        stdout: output.stdout,
        stderr,
    })
}

/// Qual pipe [`stream`] lê. O outro fica sem uso — quem chama é responsável por não deixá-lo
/// em `piped()` sem ninguém lendo, senão um comando que escrever bastante ali trava esperando
/// o pipe esvaziar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pipe {
    /// Progresso de rede (`clone`, `fetch`): o git escreve `--progress` no stderr.
    Stderr,
    /// Dado de verdade (`log`, pickaxe): o resultado do comando é o stdout.
    Stdout,
}

/// Comando que fala enquanto trabalha: cada chunk do pipe escolhido chega ao chamador na hora.
///
/// **O relógio aqui é de inatividade, não de duração.** Um clone legítimo de um repositório
/// grande passa dos trinta segundos do [`LOCAL_TIMEOUT`] com folga; o que não é legítimo é ficar
/// minutos sem dizer nada. É a diferença entre "demora" e "travou", e só a segunda merece morrer.
///
/// Cancelar é `SIGTERM` no grupo, com a mesma graça do [`run`] — e a `.git` parcial que o git
/// deixar para trás é problema do chamador, que registrou a limpeza antes de começar.
pub async fn stream<F>(
    mut command: Command,
    cancel: CancellationToken,
    idle: Duration,
    pipe: Pipe,
    mut on_chunk: F,
) -> Result<(), ExecError>
where
    F: FnMut(&[u8]),
{
    let mut child = command.spawn().map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => ExecError::NotFound,
        _ => ExecError::Spawn(err),
    })?;

    let pid = child.id();
    let mut reader: Box<dyn tokio::io::AsyncRead + Unpin + Send> = match pipe {
        Pipe::Stderr => Box::new(
            child
                .stderr
                .take()
                .expect("o `command` desta casa sempre põe stderr em pipe"),
        ),
        Pipe::Stdout => Box::new(
            child
                .stdout
                .take()
                .expect("o `command` desta casa sempre põe stdout em pipe"),
        ),
    };

    let mut buffer = [0u8; 8 * 1024];

    loop {
        tokio::select! {
            read = tokio::time::timeout(idle, reader.read(&mut buffer)) => {
                match read {
                    // Silêncio longo demais. Um prompt de senha não trava aqui
                    // (`GIT_TERMINAL_PROMPT=0`), mas rede pendurada, sim.
                    Err(_) => {
                        if let Some(pid) = pid { kill_group(pid); }
                        let _ = tokio::time::timeout(GRACE, child.wait()).await;
                        return Err(ExecError::Timeout(idle));
                    }
                    // EOF: o git fechou o pipe, então acabou.
                    Ok(Ok(0)) => break,
                    Ok(Ok(read)) => on_chunk(&buffer[..read]),
                    Ok(Err(err)) => return Err(ExecError::Spawn(err)),
                }
            }

            _ = cancel.cancelled() => {
                if let Some(pid) = pid { kill_group(pid); }
                let _ = tokio::time::timeout(GRACE, child.wait()).await;
                return Err(ExecError::Cancelled);
            }
        }
    }

    let status = child.wait().await.map_err(ExecError::Spawn)?;

    if !status.success() {
        // O cancelamento pode ter chegado entre o EOF e o `wait`: o git morre por sinal e o
        // status vem sem sucesso. Chamar isso de "falhou" mostraria um erro para quem só
        // apertou cancelar.
        if cancel.is_cancelled() {
            return Err(ExecError::Cancelled);
        }

        return Err(ExecError::Failed(Failure {
            status,
            // Os dois já foram entregues pedaço a pedaço; quem quiser guardá-los, guardou.
            stderr: String::new(),
            stdout: String::new(),
        }));
    }

    Ok(())
}

/// Encerra o grupo de processos do filho.
///
/// `SIGTERM` e não `SIGKILL`: o git limpa `index.lock` e arquivos temporários ao receber TERM,
/// e um `.git` com lock órfão é exatamente o estado do qual o usuário não sabe sair sozinho.
#[cfg(unix)]
pub fn kill_group(pid: u32) {
    // O filho é líder do próprio grupo (`process_group(0)`), então pgid == pid.
    let killed = unsafe { libc::killpg(pid as libc::pid_t, libc::SIGTERM) };

    if killed != 0 {
        tracing::debug!(pid, "grupo de processos já não existia");
    }
}

/// No Windows não há grupo de processos com esta semântica. O `kill_on_drop` e o
/// `Child::start_kill` são o que existe, e cobrem o caso comum.
#[cfg(not(unix))]
pub fn kill_group(pid: u32) {
    tracing::debug!(pid, "sem killpg nesta plataforma");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn command_traz_as_regras_da_casa() {
        let command = command(Some(Path::new("/tmp")));

        let args: Vec<_> = command
            .as_std()
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, ["--no-optional-locks", "-C", "/tmp"]);

        let envs: Vec<_> = command
            .as_std()
            .get_envs()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().into_owned(),
                    value.map(|value| value.to_string_lossy().into_owned()),
                )
            })
            .collect();
        assert!(envs.contains(&("GIT_TERMINAL_PROMPT".to_owned(), Some("0".to_owned()))));
        assert!(envs.contains(&("LC_ALL".to_owned(), Some("C".to_owned()))));
    }

    #[tokio::test]
    async fn roda_e_devolve_a_saida() {
        let mut git = command(None);
        git.arg("--version");

        let output = run(git, LOCAL_TIMEOUT).await.unwrap();
        let version = String::from_utf8_lossy(&output.stdout).into_owned();

        assert!(version.starts_with("git version"), "saiu: {version}");
    }

    #[tokio::test]
    async fn teto_de_tempo_encerra_o_comando() {
        // `git` não tem subcomando que durma; `hash-object --stdin` com stdin fechado sai na
        // hora, então o jeito de exercitar o caminho é um teto impossível de cumprir.
        let mut git = command(None);
        git.args(["-c", "protocol.version=2", "version"]);

        let err = run(git, Duration::from_nanos(1)).await.unwrap_err();
        assert!(matches!(err, ExecError::Timeout(_)), "veio {err:?}");
    }

    #[tokio::test]
    async fn falha_vira_failure_com_o_stderr() {
        let mut git = command(None);
        git.arg("subcomando-que-nao-existe");

        match run(git, LOCAL_TIMEOUT).await {
            Err(ExecError::Failed(failure)) => {
                assert!(!failure.status.success());
                assert!(!failure.stderr.is_empty(), "stderr tem que chegar inteiro");
            }
            other => panic!("esperava Failed, veio {other:?}", other = other.err()),
        }
    }
}
