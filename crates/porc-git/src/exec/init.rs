//! `git init`.
//!
//! Shell-out e não `git2::Repository::init` porque o `init` do git respeita `init.templateDir`,
//! `init.defaultBranch` e os hooks de template do usuário. Um repositório criado por dentro do
//! libgit2 nasceria diferente do que o `git init` dele criaria — e a máquina é dele.

use std::path::PathBuf;

use crate::exec::{self, ExecError, LOCAL_TIMEOUT};

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("já existe um repositório git em {0}")]
    AlreadyARepository(String),
    #[error("{0} já existe e não está vazio")]
    NotEmpty(String),
    #[error("nome de branch inválido: {0}")]
    InvalidBranch(String),
    #[error("não consegui criar {0}")]
    Create(String),
    #[error(transparent)]
    Exec(#[from] ExecError),
}

#[derive(Debug)]
pub struct InitOptions {
    /// Pasta onde o repositório nasce. Tem que existir — o confinamento só sabe validar caminho
    /// que existe, e criar árvore de diretório a partir de um caminho do cliente é convite.
    pub parent: PathBuf,
    /// Subpasta a criar. `None` inicializa a própria `parent`.
    pub name: Option<String>,
    /// `None` deixa o `init.defaultBranch` do usuário decidir — que é o certo: quem já
    /// configurou `main` não quer que a gente escolha por ele.
    pub branch: Option<String>,
}

/// Cria o repositório e devolve o caminho canônico dele.
pub async fn init(options: InitOptions) -> Result<PathBuf, InitError> {
    let target = match &options.name {
        Some(name) => options.parent.join(validate_name(name)?),
        None => options.parent.clone(),
    };

    if crate::discover::is_repo(&target) {
        return Err(InitError::AlreadyARepository(target.display().to_string()));
    }

    if let Some(branch) = &options.branch {
        validate_branch(branch)?;
    }

    if target.exists() {
        // Inicializar sobre uma pasta cheia é legítimo no git (é como se adota um projeto
        // existente), então isto não é erro — só precisa não ser surpresa. A UI pergunta antes.
        tracing::debug!(target = %target.display(), "init sobre pasta existente");
    } else {
        // `create_dir` e não `create_dir_all`: a `parent` já foi validada e existe. Criar uma
        // árvore inteira a partir de um nome vindo do cliente é o tipo de conveniência que vira
        // um bug de caminho seis meses depois.
        std::fs::create_dir(&target)
            .map_err(|_| InitError::Create(target.display().to_string()))?;
    }

    let mut git = exec::command(None);
    git.arg("init");
    if let Some(branch) = &options.branch {
        git.arg("--initial-branch").arg(branch);
    }
    // `--` fecha a lista de opções: sem ele, uma pasta chamada `--bare` viraria uma flag.
    git.arg("--").arg(&target);

    exec::run(git, LOCAL_TIMEOUT).await?;

    target
        .canonicalize()
        .map_err(|_| InitError::Create(target.display().to_string()))
}

/// O nome é uma **componente** de caminho, não um caminho.
///
/// É o que impede `../../algo` e `/etc/algo` de virarem um `init` fora da raiz confinada. O
/// caminho final ainda é revalidado pelo servidor, mas recusar aqui dá a mensagem certa.
fn validate_name(name: &str) -> Result<&str, InitError> {
    let invalid = name.trim().is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains('\0')
        || name == "."
        || name == "..";

    if invalid {
        return Err(InitError::Create(name.to_owned()));
    }

    Ok(name)
}

fn validate_branch(branch: &str) -> Result<(), InitError> {
    validate_branch_name(branch)
}

/// Nome de branch que o git aceitaria. A checagem completa é `git check-ref-format`; aqui vai o
/// suficiente para não montar uma linha de comando esquisita e para dar erro legível.
///
/// `pub(crate)` porque o `clone` valida `--branch` com a mesma régua.
pub(crate) fn validate_branch_name(branch: &str) -> Result<(), InitError> {
    let invalid = branch.is_empty()
        || branch.starts_with('-')
        || branch.starts_with('.')
        || branch.ends_with('/')
        || branch.ends_with(".lock")
        || branch.contains("..")
        || branch.contains(' ')
        || branch.contains('~')
        || branch.contains('^')
        || branch.contains(':')
        || branch.contains('?')
        || branch.contains('*')
        || branch.contains('[')
        || branch.contains('\\')
        || branch.chars().any(|c| c.is_control());

    if invalid {
        return Err(InitError::InvalidBranch(branch.to_owned()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("porc-init-{name}"));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[tokio::test]
    async fn cria_repositorio_com_a_branch_pedida() {
        let parent = temp("branch");

        let repo = init(InitOptions {
            parent: parent.clone(),
            name: Some("novo".to_owned()),
            branch: Some("main".to_owned()),
        })
        .await
        .unwrap();

        assert!(crate::discover::is_repo(&repo));

        let info = crate::read::Git2Repo::open(&repo).unwrap();
        let info = crate::read::RepoRead::info(&info).unwrap();
        // Repositório novo: tem branch, não tem commit.
        assert!(matches!(info.head, crate::model::Head::Unborn { .. }));
        assert_eq!(info.branch, "main");

        std::fs::remove_dir_all(&parent).ok();
    }

    #[tokio::test]
    async fn recusa_repositorio_que_ja_existe() {
        let parent = temp("existente");

        let options = || InitOptions {
            parent: parent.clone(),
            name: Some("novo".to_owned()),
            branch: None,
        };

        init(options()).await.unwrap();
        assert!(matches!(
            init(options()).await,
            Err(InitError::AlreadyARepository(_))
        ));

        std::fs::remove_dir_all(&parent).ok();
    }

    #[test]
    fn nome_e_componente_de_caminho() {
        assert!(validate_name("projeto").is_ok());
        assert!(validate_name("../fuga").is_err());
        assert!(validate_name("/etc").is_err());
        assert!(validate_name("").is_err());
        assert!(validate_name("..").is_err());
    }

    #[test]
    fn branch_recusa_o_que_o_git_recusaria() {
        assert!(validate_branch("main").is_ok());
        assert!(validate_branch("feat/log-canvas").is_ok());
        assert!(validate_branch("").is_err());
        assert!(validate_branch("-x").is_err());
        assert!(validate_branch("com espaco").is_err());
        assert!(validate_branch("a..b").is_err());
        assert!(validate_branch("x.lock").is_err());
    }
}
