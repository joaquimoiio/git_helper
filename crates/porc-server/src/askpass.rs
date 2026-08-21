//! Passphrase de chave SSH sem que ela toque o disco.
//!
//! O caminho todo existe para sustentar uma regra só: **o segredo vive em memória e em mais
//! lugar nenhum**. Nem em arquivo, nem em variável de ambiente, nem em `argv` — que qualquer
//! `ps` da máquina mostra, inclusive para outro usuário.
//!
//! Como funciona: antes de rodar o `git`, abrimos um socket unix num diretório efêmero só nosso
//! (`0700`, e o socket `0600`). O `git` recebe `GIT_ASKPASS`/`SSH_ASKPASS` apontando para o
//! **próprio binário do porcelain**, e `PORC_ASKPASS_SOCKET` com o caminho do socket. Quando o
//! ssh precisa da passphrase, ele executa `porc askpass "Enter passphrase for key '…'"`; esse
//! processo se conecta ao socket, manda o texto do prompt e espera. Deste lado, o prompt vira um
//! evento no WebSocket, o usuário digita na interface, a resposta volta pelo socket e o ssh
//! segue. A passphrase atravessa um socket de dono `0600` e some quando o processo morre.
//!
//! Só em Unix. No Windows o git usa o Credential Manager e não há socket unix em tokio; lá o
//! clone com chave protegida ainda não funciona pela interface, e isso está anotado.

#![cfg(unix)]

use std::{
    collections::HashMap,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixListener,
    sync::oneshot,
};
use tokio_util::sync::CancellationToken;

use crate::jobs::Jobs;

pub use porc_git::exec::ASKPASS_SOCKET_ENV;

/// Quanto o `ssh` fica esperando o usuário digitar. Passando disso, o prompt é recusado e o
/// clone falha com uma mensagem — melhor do que um job pendurado para sempre.
const ANSWER_TIMEOUT: Duration = Duration::from_secs(180);

/// Teto do texto do prompt. O do ssh cabe em duas linhas; qualquer coisa maior é ruído ou
/// tentativa de encher a memória do servidor.
const MAX_PROMPT: usize = 4 * 1024;

/// Teto da resposta. Passphrase de chave não passa disso.
const MAX_SECRET: usize = 4 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum AskpassError {
    #[error("não há nenhum pedido de senha esperando por essa resposta")]
    Unknown,
    #[error("não consegui abrir o canal de senha")]
    Socket(#[source] std::io::Error),
}

/// Os prompts abertos, esperando o usuário.
///
/// Vive no `AppState` porque quem responde é uma rota HTTP e quem pergunta é uma task de job —
/// os dois precisam de um ponto de encontro, e ele não pode ser disco.
#[derive(Default)]
pub struct Prompts {
    waiting: Mutex<HashMap<String, oneshot::Sender<String>>>,
}

impl Prompts {
    fn register(&self, prompt_id: String) -> oneshot::Receiver<String> {
        let (sender, receiver) = oneshot::channel();

        self.waiting
            .lock()
            .expect("prompts envenenado")
            .insert(prompt_id, sender);

        receiver
    }

    fn forget(&self, prompt_id: &str) {
        self.waiting
            .lock()
            .expect("prompts envenenado")
            .remove(prompt_id);
    }

    /// Entrega a resposta do usuário a quem estiver esperando.
    ///
    /// O segredo é movido, nunca copiado para um log ou para uma estrutura de longa duração:
    /// entra aqui e sai pelo socket.
    pub fn answer(&self, prompt_id: &str, secret: String) -> Result<(), AskpassError> {
        let sender = self
            .waiting
            .lock()
            .expect("prompts envenenado")
            .remove(prompt_id)
            .ok_or(AskpassError::Unknown)?;

        // `Err` significa que o outro lado desistiu (job cancelado, timeout). Não é erro do
        // usuário, e o segredo morre junto com o canal.
        let _ = sender.send(secret);

        Ok(())
    }
}

/// Um socket vivo, atrelado a um job. Some quando é derrubado.
pub struct Session {
    socket: PathBuf,
    dir: PathBuf,
    /// Derruba a task que aceita conexões junto com a sessão.
    stop: CancellationToken,
}

impl Session {
    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// Onde está o binário que o git vai executar como helper.
    ///
    /// `current_exe` e não um caminho fixo: o `porc` pode estar em qualquer lugar, inclusive num
    /// `target/debug` durante o desenvolvimento.
    pub fn helper() -> std::io::Result<PathBuf> {
        std::env::current_exe()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.stop.cancel();
        // O diretório inteiro é nosso e só tem o socket dentro.
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Abre o socket e começa a atender.
pub fn start(
    job_id: String,
    jobs: Arc<Jobs>,
    prompts: Arc<Prompts>,
    cancel: CancellationToken,
) -> Result<Session, AskpassError> {
    let dir = std::env::temp_dir().join(format!("porcelain-askpass-{job_id}"));

    std::fs::create_dir_all(&dir).map_err(AskpassError::Socket)?;
    // `0700` **antes** de o socket existir: entre criar o diretório e restringi-lo há uma janela,
    // e no `/tmp`, que é compartilhado, essa janela é a diferença entre privado e público.
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
        .map_err(AskpassError::Socket)?;

    let socket = dir.join("sock");
    let listener = UnixListener::bind(&socket).map_err(AskpassError::Socket)?;
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600))
        .map_err(AskpassError::Socket)?;

    let stop = cancel.child_token();

    tokio::spawn(accept_loop(listener, job_id, jobs, prompts, stop.clone()));

    Ok(Session { socket, dir, stop })
}

async fn accept_loop(
    listener: UnixListener,
    job_id: String,
    jobs: Arc<Jobs>,
    prompts: Arc<Prompts>,
    stop: CancellationToken,
) {
    loop {
        let accepted = tokio::select! {
            accepted = listener.accept() => accepted,
            _ = stop.cancelled() => return,
        };

        let Ok((stream, _)) = accepted else { continue };

        // Uma task por pedido: o ssh pode pedir duas vezes (chave errada na primeira), e a
        // segunda não pode ficar presa atrás da primeira.
        tokio::spawn(serve_one(
            stream,
            job_id.clone(),
            jobs.clone(),
            prompts.clone(),
            stop.clone(),
        ));
    }
}

async fn serve_one(
    mut stream: tokio::net::UnixStream,
    job_id: String,
    jobs: Arc<Jobs>,
    prompts: Arc<Prompts>,
    stop: CancellationToken,
) {
    let mut prompt = Vec::new();
    if (&mut stream)
        .take(MAX_PROMPT as u64)
        .read_to_end(&mut prompt)
        .await
        .is_err()
    {
        return;
    }

    let prompt = String::from_utf8_lossy(&prompt).trim().to_owned();
    let prompt_id = format!("{job_id}-{}", jobs.next_prompt_seq());

    let waiting = prompts.register(prompt_id.clone());

    jobs.publish_askpass(job_id.clone(), prompt_id.clone(), prompt);

    let answer = tokio::select! {
        answer = waiting => answer.ok(),
        _ = tokio::time::sleep(ANSWER_TIMEOUT) => None,
        _ = stop.cancelled() => None,
    };

    prompts.forget(&prompt_id);
    // Sempre, e não só quando ninguém respondeu: respondido também é fechado, e o campo de senha
    // tem que sair da tela em vez de esperar o evento seguinte.
    jobs.publish_askpass_closed(job_id, prompt_id);

    let Some(secret) = answer else {
        // Fechar o socket sem escrever nada faz o helper sair com erro, e o ssh desiste da chave
        // em vez de tentar com uma passphrase vazia e queimar uma tentativa.
        return;
    };

    let secret = secret.into_bytes();
    if secret.len() > MAX_SECRET {
        return;
    }

    let _ = stream.write_all(&secret).await;
    let _ = stream.shutdown().await;
}
