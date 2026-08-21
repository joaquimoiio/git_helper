//! `porc` — ponto de entrada. Flags, lockfile, abertura do navegador e modos helper
//! (askpass, sequence-editor). Não conhece git nem HTTP.

mod askpass;
mod browser;
mod cli;
mod lock;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{filter::EnvFilter, fmt};

/// Nome de produto. O crate se chama `porc-cli` e o executável `porc`; o que o usuário
/// lê na tela é sempre `porcelain`.
const APP_NAME: &str = "porcelain";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    // Antes do `clap` e antes do log: quando o `git` nos executa como helper de senha, ele passa
    // o prompt em `argv[1]` — texto livre, que o `clap` recusaria como subcomando desconhecido.
    // Quem denuncia o modo é a variável de ambiente, que só existe no ambiente que nós montamos
    // para o processo filho.
    if let Some((socket, prompt)) = askpass::requested() {
        return askpass::run(&socket, &prompt);
    }

    let args = cli::Cli::parse();

    init_tracing();

    // Validado antes de qualquer coisa subir: caminho errado tem que falhar na hora, não
    // depois de o navegador já ter aberto.
    let repo = args.repo()?;

    if args.enable_terminal {
        tracing::warn!("terminal embutido habilitado por flag");
    }

    let lock_path = lock::path()?;

    // Já tem um `porc` vivo? Então este processo não sobe servidor nenhum: só abre a aba.
    if let Some(running) = running_instance(&lock_path).await {
        let url = format!("http://127.0.0.1:{}/", running.port);
        println!("{APP_NAME} {VERSION} já está rodando em {url}");
        if !args.no_browser {
            open_browser(&url);
        }
        return Ok(());
    }

    let bound = porc_server::bind(args.port).await?;

    lock::write(
        &lock_path,
        &lock::Lock {
            pid: std::process::id(),
            port: bound.port(),
            instance_id: bound.instance_id().to_owned(),
        },
    )?;

    let url = bound.handshake_url();

    println!("{APP_NAME} {VERSION}");
    println!("{url}");

    if let Some(repo) = &repo {
        // Guardado só para o Bloco C: por enquanto o servidor ainda não abre repositório.
        println!("repositório: {}", repo.display());
    }

    if !args.no_browser {
        open_browser(&url);
    }

    let outcome = bound.serve().await;

    // Saiu do laço do servidor: o lockfile já não descreve nada vivo.
    lock::remove(&lock_path);

    outcome?;
    Ok(())
}

/// Lockfile que aponta para um servidor que responde e se identifica como o mesmo boot.
///
/// Lockfile órfão — processo morto, porta tomada por outro programa, boot antigo — devolve
/// `None` e o boot segue normalmente, sobrescrevendo o arquivo.
async fn running_instance(lock_path: &std::path::Path) -> Option<lock::Lock> {
    let lock = lock::read(lock_path)?;
    let health = porc_server::probe(lock.port).await?;

    if health.pid != lock.pid || health.instance_id != lock.instance_id {
        tracing::debug!(
            port = lock.port,
            "quem está na porta não é a instância do lockfile"
        );
        return None;
    }

    Some(lock)
}

/// O usuário fica com a URL na tela de qualquer jeito, então não abrir é chato, não fatal.
fn open_browser(url: &str) {
    if let Err(err) = browser::open(url) {
        tracing::warn!(%err, "não consegui abrir o navegador; abra a URL acima à mão");
    }
}

/// Log em `RUST_LOG`, com um piso razoável para quem só roda o binário.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("porc_server=info,porc_cli=info,tower_http=warn"));

    fmt().with_env_filter(filter).with_target(false).init();
}
