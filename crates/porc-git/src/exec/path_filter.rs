//! `git log -- <path>` e `git log -S`/`-G`, os dois em streaming.
//!
//! Shell-out e não git2, por decisão do `CLAUDE.md`: paths e conteúdo de commits não são
//! indexados, e os dois filtros são sempre uma travessia do histórico igual à que o `git log`
//! do terminal já faz. Reimplementar isso em cima do libgit2 seria duplicar trabalho que o
//! próprio git já faz bem (`-S`/`-G` nem existem na API do libgit2), só para reconstruir a
//! mesma coisa.
//!
//! Filtro por caminho (Passo 45) e busca por conteúdo — pickaxe (Passo 46) — são o mesmo
//! mecanismo: só o argumento entre `log -z --format=…` e o `--` final muda. Os dois emitem o
//! mesmo formato `%x00`, lido por [`crate::parse::records`].

use std::path::Path;

use tokio_util::sync::CancellationToken;

use crate::{
    exec::{self, ExecError, Pipe},
    parse::records::{RecordSplitter, StreamedCommit},
};

/// Nenhum dos dois filtros tem uma cadência própria de "estou vivo" como o `--progress` de
/// rede: um caminho ou um conteúdo raro pode ficar dezenas de milhares de commits sem dar
/// match nenhum, e isso não é "travado". Sessenta segundos de silêncio total num comando 100%
/// local (sem espera de rede) é folga generosa mesmo assim — o que dispara isso na prática é
/// lock contestado ou repositório corrompido, não uma busca legítima demorada.
const IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

/// Como procurar por conteúdo. As duas formas do `git log` que o libgit2 não tem.
#[derive(Debug, Clone)]
pub enum ContentMode {
    /// `-S<string>`: acha commits em que o número de vezes que `string` aparece no arquivo
    /// mudou (a pickaxe clássica — "quem adicionou ou removeu isto").
    StringCount(String),
    /// `-G<regex>`: acha commits em que alguma linha adicionada ou removida do diff casa a
    /// expressão regular.
    Regex(String),
}

impl ContentMode {
    fn flag(&self) -> String {
        match self {
            ContentMode::StringCount(needle) => format!("-S{needle}"),
            ContentMode::Regex(pattern) => format!("-G{pattern}"),
        }
    }
}

/// O `git log -z --format=…` comum aos dois filtros. `extra` é o que diferencia um do outro
/// (`-- <path>` ou `-S`/`-G`), sempre como argumentos já prontos — quem monta a flag decide a
/// forma exata, esta função só executa.
async fn run<F>(
    repo: &Path,
    extra: &[&str],
    cancel: CancellationToken,
    mut on_commit: F,
) -> Result<(), ExecError>
where
    F: FnMut(StreamedCommit),
{
    let mut git = exec::command(Some(repo));
    git.arg("log")
        .arg("-z")
        .arg("--format=%H%x00%an%x00%ae%x00%at%x00%s")
        .args(extra);
    // Ninguém lê o stderr deste comando; sem isto, um pipe cheio de aviso poderia travar o git
    // esperando alguém esvaziá-lo.
    git.stderr(std::process::Stdio::null());

    let mut splitter = RecordSplitter::default();

    exec::stream(git, cancel, IDLE_TIMEOUT, Pipe::Stdout, |chunk| {
        for commit in splitter.push(chunk) {
            on_commit(commit);
        }
    })
    .await
}

/// Só os commits que tocaram `path`, do `HEAD` para trás. O pathspec sempre vem depois de
/// `--`: mesmo se `path` começar com `-`, ele nunca vira flag.
pub async fn by_path<F>(
    repo: &Path,
    path: &str,
    cancel: CancellationToken,
    on_commit: F,
) -> Result<(), ExecError>
where
    F: FnMut(StreamedCommit),
{
    run(repo, &["--", path], cancel, on_commit).await
}

/// Só os commits cujo diff casa `mode` — pickaxe. Sem pathspec: procura no repositório
/// inteiro, como `-S`/`-G` fazem por padrão.
pub async fn by_content<F>(
    repo: &Path,
    mode: &ContentMode,
    cancel: CancellationToken,
    on_commit: F,
) -> Result<(), ExecError>
where
    F: FnMut(StreamedCommit),
{
    // `-S<valor>`/`-G<valor>` é um token só (a flag e o valor grudados, sintaxe curta do
    // git): mesmo que `valor` comece com `-` ou pareça outra flag, ele nunca é lido como tal —
    // só existe *depois* do prefixo `-S`/`-G` dentro do mesmo argumento de `argv`.
    let flag = mode.flag();
    run(repo, &[&flag], cancel, on_commit).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .to_path_buf()
    }

    #[tokio::test]
    async fn traz_so_os_commits_que_tocaram_o_arquivo() {
        let root = project_root();
        let mut oids = Vec::new();

        by_path(&root, "PROGRESSO.md", CancellationToken::new(), |commit| {
            oids.push(commit.oid);
        })
        .await
        .unwrap();

        // `git log --oneline -- PROGRESSO.md` neste repositório, hoje: os três commits que
        // existem tocaram o arquivo (o andaime inicial já o cria).
        assert_eq!(
            oids,
            [
                "9db77b8e291c73877e0052d085d5c2236967b062",
                "0b61c6f62675d70dec6e486da71f89b9cd6a6561",
                "2cea64adc1a40be6a9388b50800daea31c850d84",
            ]
        );
    }

    #[tokio::test]
    async fn caminho_que_nunca_existiu_nao_traz_nada_e_nao_falha() {
        let root = project_root();
        let mut count = 0;

        by_path(
            &root,
            "arquivo/que/nunca/existiu.txt",
            CancellationToken::new(),
            |_| count += 1,
        )
        .await
        .unwrap();

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn cancelamento_interrompe_o_streaming() {
        let root = project_root();
        let cancel = CancellationToken::new();
        cancel.cancel();

        let result = by_path(&root, "PROGRESSO.md", cancel, |_| {}).await;

        assert!(matches!(result, Err(ExecError::Cancelled)));
    }

    #[tokio::test]
    async fn cada_campo_vem_correto_para_o_commit_raiz() {
        let root = project_root();
        let mut hits = Vec::new();

        by_path(&root, "PROGRESSO.md", CancellationToken::new(), |commit| {
            hits.push(commit);
        })
        .await
        .unwrap();

        let root_commit = hits
            .iter()
            .find(|commit| commit.oid == "2cea64adc1a40be6a9388b50800daea31c850d84")
            .expect("commit raiz tem que estar na lista");

        assert!(!root_commit.author.is_empty());
        assert!(!root_commit.email.is_empty());
        assert!(root_commit.time > 0);
        assert!(!root_commit.summary.is_empty());
    }

    #[tokio::test]
    async fn pickaxe_por_string_acha_onde_a_contagem_mudou() {
        let root = project_root();
        let mut oids = Vec::new();

        by_content(
            &root,
            &ContentMode::StringCount("porcelain".to_owned()),
            CancellationToken::new(),
            |commit| oids.push(commit.oid),
        )
        .await
        .unwrap();

        // `git log --oneline -S"porcelain"` neste repositório, hoje: os três commits — a
        // palavra aparece e muda de contagem em cada um dos três.
        assert_eq!(
            oids,
            [
                "9db77b8e291c73877e0052d085d5c2236967b062",
                "0b61c6f62675d70dec6e486da71f89b9cd6a6561",
                "2cea64adc1a40be6a9388b50800daea31c850d84",
            ]
        );
    }

    #[tokio::test]
    async fn pickaxe_por_regex_acha_a_linha_no_diff() {
        let root = project_root();
        let mut oids = Vec::new();

        by_content(
            &root,
            &ContentMode::Regex(r"fn open\(".to_owned()),
            CancellationToken::new(),
            |commit| oids.push(commit.oid),
        )
        .await
        .unwrap();

        // `git log --oneline -G"fn open\("` neste repositório, hoje: dois commits.
        assert_eq!(
            oids,
            [
                "9db77b8e291c73877e0052d085d5c2236967b062",
                "0b61c6f62675d70dec6e486da71f89b9cd6a6561",
            ]
        );
    }

    #[tokio::test]
    async fn pickaxe_sem_ocorrencia_nao_traz_nada_e_nao_falha() {
        let root = project_root();
        let mut count = 0;

        by_content(
            &root,
            &ContentMode::StringCount("string-que-nunca-existiu-em-lugar-nenhum".to_owned()),
            CancellationToken::new(),
            |_| count += 1,
        )
        .await
        .unwrap();

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn pickaxe_cancelamento_interrompe_o_streaming() {
        let root = project_root();
        let cancel = CancellationToken::new();
        cancel.cancel();

        let result = by_content(
            &root,
            &ContentMode::StringCount("porcelain".to_owned()),
            cancel,
            |_| {},
        )
        .await;

        assert!(matches!(result, Err(ExecError::Cancelled)));
    }
}
