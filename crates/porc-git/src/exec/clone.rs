//! `git clone`.
//!
//! Shell-out, e aqui não há nem discussão: clonar precisa do credential helper, do SSH agent, do
//! `~/.gitconfig` com `insteadOf` e do LFS que o usuário já tem configurados. Um clone por dentro
//! do libgit2 falharia justamente nos repositórios privados, que são a maioria dos que importam.
//!
//! Duas coisas separam este módulo de um `Command::new("git")` qualquer:
//!
//! **`prepare` antes de `run`.** O chamador precisa saber, *antes* de o git começar, qual é a
//! pasta de destino e se **nós** vamos criá-la. Sem isso não dá para registrar a limpeza antes
//! de começar, e cancelar viraria adivinhação sobre o que pode ser apagado.
//!
//! **`protocol.ext.allow=never`.** O transporte `ext::` do git executa um comando arbitrário
//! (`git clone "ext::sh -c ..."`). O `--` já impede a URL de virar flag, mas um `ext::` é uma URL
//! *válida* que roda shell. Desligá-lo por `-c` é override efêmero, exatamente como manda a casa.

use std::path::{Path, PathBuf};

use tokio_util::sync::CancellationToken;

use crate::{
    exec::{self, ExecError},
    parse::progress::{self, ProgressEvent},
};

/// Silêncio máximo tolerado. Um clone grande demora; um clone travado fica mudo.
pub const IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

/// Quantas linhas de stderr guardar para o diagnóstico de erro (Passo 34). O suficiente para a
/// mensagem do git caber; não é um arquivo de log.
const STDERR_TAIL: usize = 40;

#[derive(Debug, thiserror::Error)]
pub enum CloneError {
    #[error("informe a URL do repositório")]
    MissingUrl,
    #[error("não entendi a URL do repositório")]
    InvalidUrl,
    #[error("o transporte `ext::` executa comandos e está desabilitado")]
    ExtTransport,
    #[error("nome de pasta inválido: {0}")]
    InvalidFolder(String),
    #[error("nome de branch inválido: {0}")]
    InvalidBranch(String),
    #[error("nome de remote inválido: {0}")]
    InvalidRemote(String),
    #[error("{0} já existe e não está vazio")]
    NotEmpty(String),
    #[error("clone cancelado")]
    Cancelled,
    #[error(transparent)]
    Exec(#[from] ExecError),
}

#[derive(Debug, Clone)]
pub struct CloneOptions {
    pub url: String,
    /// Pasta existente onde o clone vai cair.
    pub parent: PathBuf,
    /// Nome da pasta a criar. Ausente, sai da URL (`.../porcelain.git` → `porcelain`).
    pub folder: Option<String>,
    pub branch: Option<String>,
    /// `--depth`. Clone raso.
    pub depth: Option<u32>,
    pub recurse_submodules: bool,
    /// Nome do remote. Ausente é `origin`, o padrão do git.
    pub remote: Option<String>,
}

/// O que se sabe antes de o git começar.
#[derive(Debug, Clone)]
pub struct Prepared {
    pub target: PathBuf,
    /// `true` quando a pasta **não existia** — é a única condição em que cancelar pode apagá-la.
    /// Pasta preexistente é do usuário, e nunca some por conta nossa.
    pub creates_target: bool,
    args: Vec<String>,
}

impl Prepared {
    pub fn target(&self) -> &Path {
        &self.target
    }
}

/// Valida tudo e monta a linha de comando, sem tocar na rede nem criar nada.
pub fn prepare(options: &CloneOptions) -> Result<Prepared, CloneError> {
    let url = options.url.trim();
    if url.is_empty() {
        return Err(CloneError::MissingUrl);
    }
    if url.contains(['\0', '\n', '\r']) {
        return Err(CloneError::InvalidUrl);
    }
    if url.starts_with("ext::") {
        return Err(CloneError::ExtTransport);
    }

    let folder = match &options.folder {
        Some(folder) => validate_component(folder)?.to_owned(),
        None => folder_from_url(url).ok_or(CloneError::InvalidUrl)?,
    };

    let target = options.parent.join(&folder);

    // Pasta cheia é o erro que o git daria de qualquer jeito, mas dizê-lo antes de abrir conexão
    // poupa o usuário de esperar um download para ouvir "não".
    if target.exists() && !is_empty_dir(&target) {
        return Err(CloneError::NotEmpty(target.display().to_string()));
    }

    if let Some(branch) = &options.branch {
        crate::exec::init::validate_branch_name(branch)
            .map_err(|_| CloneError::InvalidBranch(branch.clone()))?;
    }
    if let Some(remote) = &options.remote {
        validate_component(remote).map_err(|_| CloneError::InvalidRemote(remote.clone()))?;
    }

    let mut args = vec![
        // Override efêmero, nunca `git config --global`.
        "-c".to_owned(),
        "protocol.ext.allow=never".to_owned(),
        "clone".to_owned(),
        // Sem isto o git só mostra progresso quando a saída é um terminal — e a nossa não é.
        "--progress".to_owned(),
    ];

    if let Some(branch) = &options.branch {
        args.push("--branch".to_owned());
        args.push(branch.clone());
    }
    if let Some(depth) = options.depth {
        args.push(format!("--depth={depth}"));
        // `--depth` sem isto traz só a branch pedida. Explicitar evita a surpresa de um clone
        // raso que "perdeu" as outras branches.
        args.push("--no-single-branch".to_owned());
    }
    if options.recurse_submodules {
        args.push("--recurse-submodules".to_owned());
    }
    if let Some(remote) = &options.remote {
        args.push("--origin".to_owned());
        args.push(remote.clone());
    }

    // `--` fecha as opções: a partir daqui, uma URL que comece com `-` é uma URL, não uma flag.
    args.push("--".to_owned());
    args.push(url.to_owned());
    args.push(target.to_string_lossy().into_owned());

    Ok(Prepared {
        creates_target: !target.exists(),
        target,
        args,
    })
}

/// Roda o clone, entregando cada evento de progresso ao chamador.
///
/// Devolve o caminho canônico do repositório clonado. O `stderr_tail` é preenchido com o que o
/// git disse que não era progresso — é dele que sai a mensagem legível quando algo falha.
pub async fn run<F>(
    prepared: &Prepared,
    askpass: Option<&exec::Askpass>,
    cancel: CancellationToken,
    mut on_event: F,
) -> Result<PathBuf, CloneError>
where
    F: FnMut(ProgressEvent),
{
    let mut git = exec::command(None);
    git.args(&prepared.args);

    if let Some(askpass) = askpass {
        askpass.apply(&mut git);
    }
    // O clone não escreve nada de útil no stdout, e um pipe que ninguém lê enche e trava.
    git.stdout(std::process::Stdio::null());

    let mut splitter = progress::Splitter::default();

    let outcome = exec::stream(git, cancel, IDLE_TIMEOUT, exec::Pipe::Stderr, |chunk| {
        for update in splitter.push(chunk) {
            on_event(progress::parse(&update));
        }
    })
    .await;

    if let Some(rest) = splitter.flush() {
        on_event(progress::parse(&rest));
    }

    match outcome {
        Ok(()) => {}
        Err(ExecError::Cancelled) => return Err(CloneError::Cancelled),
        Err(err) => return Err(CloneError::Exec(err)),
    }

    prepared
        .target
        .canonicalize()
        .map_err(|_| CloneError::InvalidUrl)
}

/// Guarda as últimas linhas de stderr que não são progresso.
///
/// Progresso é ruído para diagnóstico — quinze mil "Receiving objects" não dizem por que o clone
/// falhou. O que importa é o que o git escreveu *fora* das barras.
#[derive(Debug, Default)]
pub struct StderrTail {
    lines: Vec<String>,
}

impl StderrTail {
    pub fn push(&mut self, event: &ProgressEvent) {
        let ProgressEvent::Other(line) = event else {
            return;
        };

        if self.lines.len() == STDERR_TAIL {
            self.lines.remove(0);
        }
        self.lines.push(line.clone());
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }
}

/// `https://github.com/rust-lang/log.git` → `log`; `git@host:org/proj.git` → `proj`.
fn folder_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim_end_matches('/');

    // Vale para `/` e para o `:` do SSH curto (`git@host:org/proj.git`).
    let last = trimmed.rsplit(['/', ':']).next()?;
    let name = last.strip_suffix(".git").unwrap_or(last);

    validate_component(name).ok().map(str::to_owned)
}

/// Uma **componente** de caminho, não um caminho. É o que impede a pasta de destino de escapar.
fn validate_component(raw: &str) -> Result<&str, CloneError> {
    let name = raw.trim();

    let invalid = name.is_empty()
        || name.starts_with('-')
        || name.contains('/')
        || name.contains('\\')
        || name.contains('\0')
        || name == "."
        || name == "..";

    if invalid {
        return Err(CloneError::InvalidFolder(raw.to_owned()));
    }

    Ok(name)
}

fn is_empty_dir(path: &Path) -> bool {
    std::fs::read_dir(path).is_ok_and(|mut entries| entries.next().is_none())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(url: &str) -> CloneOptions {
        CloneOptions {
            url: url.to_owned(),
            parent: std::env::temp_dir(),
            folder: None,
            branch: None,
            depth: None,
            recurse_submodules: false,
            remote: None,
        }
    }

    #[test]
    fn nome_da_pasta_sai_da_url() {
        assert_eq!(
            folder_from_url("https://github.com/rust-lang/log.git").as_deref(),
            Some("log")
        );
        assert_eq!(
            folder_from_url("https://github.com/rust-lang/log/").as_deref(),
            Some("log")
        );
        assert_eq!(
            folder_from_url("git@github.com:tokio-rs/tokio.git").as_deref(),
            Some("tokio")
        );
    }

    #[test]
    fn recusa_ext_e_url_com_controle() {
        assert!(matches!(
            prepare(&options("ext::sh -c 'rm -rf /'")),
            Err(CloneError::ExtTransport)
        ));
        assert!(matches!(
            prepare(&options("https://x/y\nrm")),
            Err(CloneError::InvalidUrl)
        ));
        assert!(matches!(
            prepare(&options("   ")),
            Err(CloneError::MissingUrl)
        ));
    }

    #[test]
    fn a_url_nunca_vira_flag() {
        let prepared = prepare(&CloneOptions {
            folder: Some("destino".to_owned()),
            ..options("--upload-pack=maldade")
        })
        .unwrap();

        let fim = &prepared.args[prepared.args.len() - 3..];
        assert_eq!(fim[0], "--");
        assert_eq!(fim[1], "--upload-pack=maldade");
    }

    #[test]
    fn opcoes_viram_flags() {
        let prepared = prepare(&CloneOptions {
            branch: Some("main".to_owned()),
            depth: Some(1),
            recurse_submodules: true,
            remote: Some("upstream".to_owned()),
            ..options("https://github.com/rust-lang/log.git")
        })
        .unwrap();

        let args = prepared.args.join(" ");
        assert!(args.contains("-c protocol.ext.allow=never"));
        assert!(args.contains("--progress"));
        assert!(args.contains("--branch main"));
        assert!(args.contains("--depth=1 --no-single-branch"));
        assert!(args.contains("--recurse-submodules"));
        assert!(args.contains("--origin upstream"));
        assert!(prepared.target.ends_with("log"));
    }

    #[test]
    fn pasta_que_nao_existe_e_nossa_para_apagar() {
        let parent = std::env::temp_dir().join("porc-clone-prepare");
        std::fs::create_dir_all(&parent).unwrap();
        std::fs::remove_dir_all(parent.join("novo")).ok();

        let nossa = prepare(&CloneOptions {
            parent: parent.clone(),
            folder: Some("novo".to_owned()),
            ..options("https://github.com/rust-lang/log.git")
        })
        .unwrap();
        assert!(nossa.creates_target);

        // Pasta vazia preexistente serve de destino, mas não é nossa para apagar.
        std::fs::create_dir_all(parent.join("vazia")).unwrap();
        let dele = prepare(&CloneOptions {
            parent: parent.clone(),
            folder: Some("vazia".to_owned()),
            ..options("https://github.com/rust-lang/log.git")
        })
        .unwrap();
        assert!(!dele.creates_target);

        // Pasta cheia é recusada antes de abrir conexão.
        std::fs::create_dir_all(parent.join("cheia")).unwrap();
        std::fs::write(parent.join("cheia/arquivo"), b"x").unwrap();
        assert!(matches!(
            prepare(&CloneOptions {
                parent: parent.clone(),
                folder: Some("cheia".to_owned()),
                ..options("https://github.com/rust-lang/log.git")
            }),
            Err(CloneError::NotEmpty(_))
        ));

        std::fs::remove_dir_all(&parent).ok();
    }

    #[test]
    fn tail_guarda_so_o_que_nao_e_progresso() {
        let mut tail = StderrTail::default();

        tail.push(&progress::parse("Receiving objects:  10% (1/10)"));
        tail.push(&progress::parse("fatal: repository not found"));

        assert_eq!(tail.text(), "fatal: repository not found");
    }
}
