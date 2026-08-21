//! Abre o navegador padrão do sistema na URL de handshake.
//!
//! Falhar aqui nunca é fatal: sem navegador o usuário ainda tem a URL impressa no
//! terminal, e o servidor continua servindo.

use std::process::{Command, Stdio};

pub fn open(url: &str) -> std::io::Result<()> {
    let status = command_for(url)
        // O launcher não fala nada de útil e sujaria o terminal do usuário.
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        return Err(std::io::Error::other(format!(
            "o abridor de URL do sistema saiu com {status}"
        )));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn command_for(url: &str) -> Command {
    let mut command = Command::new("open");
    command.arg(url);
    command
}

#[cfg(target_os = "linux")]
fn command_for(url: &str) -> Command {
    let mut command = Command::new("xdg-open");
    command.arg(url);
    command
}

#[cfg(target_os = "windows")]
fn command_for(url: &str) -> Command {
    let mut command = Command::new("cmd");
    // O primeiro argumento de `start` é o título da janela; sem o `""` o Windows trataria
    // a URL como título e não abriria nada.
    command.args(["/c", "start", "", url]);
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passa_a_url_para_o_launcher_do_sistema() {
        let command = command_for("http://127.0.0.1:7867/?t=abc");

        let args: Vec<_> = command.get_args().map(|arg| arg.to_owned()).collect();
        assert!(
            args.iter().any(|arg| arg == "http://127.0.0.1:7867/?t=abc"),
            "a URL tem que chegar ao launcher: {:?} {:?}",
            command.get_program(),
            args
        );
    }
}
