//! Modo helper de `GIT_ASKPASS`/`SSH_ASKPASS`.
//!
//! Quem executa isto é o `git` (ou o `ssh`), não o usuário — e o contrato deles é **o programa
//! inteiro**, não um subcomando: o git roda `$GIT_ASKPASS "<prompt>"` e lê a primeira linha do
//! stdout. Por isso o modo não é `porc askpass …`: é o `porc` percebendo, pela presença de
//! `PORC_ASKPASS_SOCKET` no ambiente, que quem o chamou foi o git.
//!
//! O segredo não está em lugar nenhum deste processo: ele é buscado num socket unix cujo caminho
//! veio pelo ambiente. **O caminho** vai no ambiente; o **segredo**, nunca — nem em `argv`, que
//! qualquer `ps` da máquina mostra, nem em arquivo.
//!
//! Falhar aqui tem que sair com código diferente de zero: um helper que sai com 0 e stdout vazio
//! faz o ssh tentar com passphrase vazia e queimar uma tentativa.

use std::io::{Read, Write};

use anyhow::{bail, Context, Result};

/// `Some` quando este processo foi lançado como helper. O prompt vem em `argv[1]`.
pub fn requested() -> Option<(String, String)> {
    let socket = std::env::var(porc_server::askpass::ASKPASS_SOCKET_ENV).ok()?;
    // Sem prompt não há o que perguntar, mas o socket manda: responder vazio é melhor do que
    // subir um servidor por engano dentro de um clone.
    let prompt = std::env::args().nth(1).unwrap_or_default();

    Some((socket, prompt))
}

pub fn run(socket: &str, prompt: &str) -> Result<()> {
    let secret = ask(socket, prompt).context("não consegui buscar a senha no porcelain")?;

    // Sem `\n` extra além do que veio: o git lê a primeira linha e corta o fim de linha, e uma
    // passphrase com espaço no fim é uma passphrase válida.
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(secret.as_bytes())?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;

    Ok(())
}

#[cfg(unix)]
fn ask(socket: &str, prompt: &str) -> Result<String> {
    use std::os::unix::net::UnixStream;

    let mut stream = UnixStream::connect(socket).context("o porcelain não está mais esperando")?;

    stream.write_all(prompt.as_bytes())?;
    // Meia-fechada: é o EOF que diz ao servidor que o prompt acabou, sem precisar de um
    // protocolo com tamanho no cabeçalho.
    stream.shutdown(std::net::Shutdown::Write)?;

    let mut secret = String::new();
    stream.read_to_string(&mut secret)?;

    if secret.is_empty() {
        // O servidor fecha sem escrever quando o usuário desiste ou o pedido expira.
        bail!("ninguém respondeu ao pedido de senha");
    }

    Ok(secret)
}

#[cfg(not(unix))]
fn ask(_socket: &str, _prompt: &str) -> Result<String> {
    bail!("o canal de senha do porcelain ainda não existe nesta plataforma")
}
