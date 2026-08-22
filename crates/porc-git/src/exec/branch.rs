//! `git branch` e `git switch --create` — criar branch (Passo 58).
//!
//! Shell-out e não `git2::Repository::branch`: escrever a ref por dentro do libgit2 funcionaria,
//! mas o passo pede **checkout opcional** e **upstream opcional** no mesmo gesto. Com checkout o
//! git escreve índice e worktree e dispara `post-checkout`; com tracking ele escreve
//! `branch.<nome>.remote`/`.merge` respeitando o `branch.autoSetupMerge` do usuário. Reimplementar
//! as duas coisas seria escrever um segundo git — pior, e na máquina dele.
//!
//! Local e rápido: roda até o fim sob o [`crate::exec::LOCAL_TIMEOUT`], não é streaming.

use std::path::Path;

use crate::exec::{self, ExecError, LOCAL_TIMEOUT};

#[derive(Debug, thiserror::Error)]
pub enum BranchError {
    #[error("nome de branch inválido: {0}")]
    InvalidName(String),
    #[error("ponto de partida inválido: {0}")]
    InvalidStart(String),
    #[error("já existe uma branch chamada {0}")]
    AlreadyExists(String),
    #[error("não encontrei {0} neste repositório")]
    UnknownStart(String),
    #[error("há mudanças locais que a troca de branch sobrescreveria — commite, guarde num stash ou descarte antes")]
    WouldOverwrite,
    /// O git — ou um **hook** dele, quase sempre o `post-checkout` — recusou. Vale o mesmo que
    /// no `commit`: o texto que sobra aqui costuma ser escrito pelo time do usuário, e é a única
    /// coisa útil que existe sobre a recusa.
    #[error("o git recusou criar a branch: {0}")]
    Refused(String),
    #[error(transparent)]
    Exec(#[from] ExecError),
}

#[derive(Debug, Clone, Default)]
pub struct CreateOptions {
    /// Nome da branch nova, sem `refs/heads/`.
    pub name: String,
    /// De onde ela nasce. `None` é o `HEAD` — que é o que o git faz sem ponto de partida nenhum.
    pub start: Option<String>,
    /// `true` deixa o `HEAD` na branch nova (`git switch --create`); `false` só cria a ref.
    pub checkout: bool,
    /// `None` respeita o `branch.autoSetupMerge` do usuário (que já liga o tracking sozinho ao
    /// partir de uma remota); `Some` força ligado ou desligado **só para esta criação**, por
    /// flag — nunca escrevendo na config global dele. Mesma régua do `gpg_sign` no commit.
    pub track: Option<bool>,
}

/// Cria a branch e devolve o oid para o qual ela nasceu apontando.
pub async fn create(repo: &Path, options: &CreateOptions) -> Result<String, BranchError> {
    validate_name(&options.name)?;
    if let Some(start) = &options.start {
        validate_start(start)?;
    }

    let mut git = exec::command(Some(repo));

    if options.checkout {
        // `switch --create` e não `checkout -b`: o `checkout` é dois comandos num só (trocar de
        // branch e restaurar arquivo), e é justamente a metade que restaura arquivo que destrói
        // trabalho sem aviso. O `switch` não tem essa metade. Exige git ≥ 2.23; o piso é 2.30.
        git.arg("switch");
        push_track(&mut git, options.track);
        // `--create` **depois** do tracking, e colado no nome: ele pede um valor, e um
        // `--create --track` faria o git tomar `--track` como o nome da branch.
        git.arg("--create").arg(&options.name);
    } else {
        git.arg("branch");
        push_track(&mut git, options.track);
        git.arg(&options.name);
    }

    if let Some(start) = &options.start {
        git.arg(start);
    }

    if let Err(ExecError::Failed(failure)) = exec::run(git, LOCAL_TIMEOUT).await {
        return Err(classify(&failure.stderr, options));
    }

    tip_of(repo, &options.name).await
}

fn push_track(git: &mut tokio::process::Command, track: Option<bool>) {
    match track {
        Some(true) => git.arg("--track"),
        Some(false) => git.arg("--no-track"),
        // Sem flag nenhuma: vale o `branch.autoSetupMerge` dele, que é o certo por padrão.
        None => git,
    };
}

/// O stderr de uma criação que falhou vira o erro certo. O específico antes do genérico, como no
/// `commit` e no `exec::error` — e todas as frases chegam em inglês por causa do `LC_ALL=C`.
fn classify(stderr: &str, options: &CreateOptions) -> BranchError {
    if stderr.contains("already exists") {
        return BranchError::AlreadyExists(options.name.clone());
    }

    if stderr.contains("not a valid object name")
        || stderr.contains("invalid reference")
        || stderr.contains("Not a valid branch point")
        || stderr.contains("unknown revision")
    {
        return BranchError::UnknownStart(
            options.start.clone().unwrap_or_else(|| "HEAD".to_owned()),
        );
    }

    if stderr.contains("would be overwritten by checkout")
        || stderr.contains("Please commit your changes or stash them")
    {
        return BranchError::WouldOverwrite;
    }

    let details: Vec<&str> = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(3)
        .collect();

    if details.is_empty() {
        return BranchError::Refused("o git não disse por quê".to_owned());
    }

    BranchError::Refused(details.join(" · "))
}

/// Nome da branch nova, com a mesma régua do `init` e do `clone --branch` — uma régua só para
/// nome de branch no projeto inteiro.
fn validate_name(name: &str) -> Result<(), BranchError> {
    crate::exec::init::validate_branch_name(name)
        .map_err(|_| BranchError::InvalidName(name.to_owned()))
}

/// O ponto de partida é **oid ou nome de ref**, não revisão livre.
///
/// O passo pede "a partir de `HEAD`, de um commit selecionado no log ou de qualquer ref", e é
/// exatamente isso que a UI tem na mão. Aceitar revisão livre daria a ela `HEAD@{…}`, `:/texto` e
/// companhia — sintaxes que o git resolve e que ninguém aqui está preparado para explicar quando
/// derem em outro commit. Mesmo raciocínio da validação do alvo do `--fixup` (Passo 54).
///
/// A recusa também é o que impede um valor começando com `-` de virar flag na linha de comando.
fn validate_start(start: &str) -> Result<(), BranchError> {
    let oid = (7..=40).contains(&start.len()) && start.chars().all(|c| c.is_ascii_hexdigit());
    // A régua de nome de branch mais o `@{…}`: o `git check-ref-format` já recusa `@{` num nome
    // de ref, mas ele é justamente a sintaxe de reflog (`main@{ontem}`) que não se quer aceitar
    // aqui, então a recusa é explícita em vez de herdada.
    let refname = crate::exec::init::validate_branch_name(start).is_ok()
        && !start.contains('{')
        && !start.contains('}');

    if oid || refname {
        Ok(())
    } else {
        Err(BranchError::InvalidStart(start.to_owned()))
    }
}

/// Para onde a branch recém-criada aponta.
///
/// `refs/heads/<nome>` inteiro, e não o nome curto: um arquivo chamado `nova` na worktree tornaria
/// `git rev-parse nova` ambíguo, e o git escolheria por conta própria.
async fn tip_of(repo: &Path, name: &str) -> Result<String, BranchError> {
    let mut git = exec::command(Some(repo));
    git.args(["rev-parse", "--verify"])
        .arg(format!("refs/heads/{name}"));

    let output = exec::run(git, LOCAL_TIMEOUT).await?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn git(dir: &Path, args: &[&str]) -> String {
        let mut command = exec::command(Some(dir));
        command.args(args);

        let output = exec::run(command, LOCAL_TIMEOUT).await.unwrap();
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    }

    /// Repositório com dois commits na `main`. Devolve a pasta.
    async fn repo(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("porc-branch-{name}"));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();

        let mut init = exec::command(None);
        // `--initial-branch` fixo: o teste compara nomes, e o `init.defaultBranch` da máquina de
        // quem roda a suíte não pode decidir o resultado.
        init.args(["init", "--initial-branch", "main"]).arg(&dir);
        exec::run(init, LOCAL_TIMEOUT).await.unwrap();

        // Identidade no config **local** do repositório de teste: nunca `--global`.
        for (key, value) in [("user.email", "teste@example.com"), ("user.name", "Teste")] {
            git(&dir, &["config", key, value]).await;
        }

        for (arquivo, mensagem) in [("a.txt", "primeiro"), ("b.txt", "segundo")] {
            std::fs::write(dir.join(arquivo), "conteudo\n").unwrap();
            crate::exec::stage::add(&dir, &[arquivo.to_owned()])
                .await
                .unwrap();
            git(&dir, &["commit", "-m", mensagem]).await;
        }

        dir
    }

    #[tokio::test]
    async fn cria_a_partir_do_head_sem_trocar_de_branch() {
        let dir = repo("head").await;
        let head = git(&dir, &["rev-parse", "HEAD"]).await;

        let oid = create(
            &dir,
            &CreateOptions {
                name: "nova".to_owned(),
                ..CreateOptions::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(oid, head);
        // O `HEAD` continua onde estava: criar não é trocar.
        assert_eq!(
            git(&dir, &["rev-parse", "--abbrev-ref", "HEAD"]).await,
            "main"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn cria_a_partir_de_um_commit_antigo() {
        let dir = repo("antigo").await;
        let antigo = git(&dir, &["rev-parse", "HEAD~1"]).await;

        let oid = create(
            &dir,
            &CreateOptions {
                name: "do-passado".to_owned(),
                start: Some(antigo.clone()),
                ..CreateOptions::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(oid, antigo);
        assert_eq!(
            git(&dir, &["rev-parse", "refs/heads/do-passado"]).await,
            antigo
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn com_checkout_o_head_passa_a_ser_a_nova() {
        let dir = repo("checkout").await;
        let antigo = git(&dir, &["rev-parse", "HEAD~1"]).await;

        let oid = create(
            &dir,
            &CreateOptions {
                name: "trabalhando".to_owned(),
                start: Some(antigo.clone()),
                checkout: true,
                ..CreateOptions::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(oid, antigo);
        assert_eq!(
            git(&dir, &["rev-parse", "--abbrev-ref", "HEAD"]).await,
            "trabalhando"
        );
        // O checkout de verdade aconteceu: o segundo commit não está mais na worktree.
        assert!(!dir.join("b.txt").exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn track_configura_o_upstream() {
        let dir = repo("track").await;

        create(
            &dir,
            &CreateOptions {
                name: "com-upstream".to_owned(),
                start: Some("main".to_owned()),
                track: Some(true),
                ..CreateOptions::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(
            git(&dir, &["config", "--get", "branch.com-upstream.merge"]).await,
            "refs/heads/main"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn nome_repetido_vira_erro_proprio() {
        let dir = repo("repetido").await;

        let options = CreateOptions {
            name: "main".to_owned(),
            ..CreateOptions::default()
        };
        let err = create(&dir, &options).await.unwrap_err();

        assert!(
            matches!(err, BranchError::AlreadyExists(nome) if nome == "main"),
            "veio outro erro"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn ponto_de_partida_desconhecido_vira_erro_proprio() {
        let dir = repo("sem-partida").await;

        let err = create(
            &dir,
            &CreateOptions {
                name: "nova".to_owned(),
                start: Some("naoexiste".to_owned()),
                ..CreateOptions::default()
            },
        )
        .await
        .unwrap_err();

        assert!(
            matches!(err, BranchError::UnknownStart(alvo) if alvo == "naoexiste"),
            "veio outro erro"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn nome_e_partida_invalidos_nao_chegam_ao_git() {
        for nome in ["", "-x", "com espaco", "a..b", "x.lock"] {
            assert!(
                matches!(validate_name(nome), Err(BranchError::InvalidName(_))),
                "aceitou o nome {nome:?}"
            );
        }

        assert!(validate_name("feat/log-canvas").is_ok());

        for start in ["HEAD~2", ":/assunto", "-f", "main@{1}", ""] {
            assert!(
                matches!(validate_start(start), Err(BranchError::InvalidStart(_))),
                "aceitou a partida {start:?}"
            );
        }

        for start in ["HEAD", "main", "origin/main", "v1.0", "9db77b8"] {
            assert!(validate_start(start).is_ok(), "recusou a partida {start:?}");
        }
    }
}
