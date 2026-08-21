//! Leitura de repositório, atrás do trait `RepoRead`.
//!
//! O trait não é cerimônia: o plano B do hot path de leitura é trocar libgit2 por `gix`, e a
//! única forma de essa troca não tocar rota nem UI é nenhuma delas conhecer `git2`. Hoje há
//! uma implementação só.
//!
//! Nada aqui é `Sync` do lado do libgit2 (`git2::Repository` não é), e tudo é bloqueante. Quem
//! chama tem que estar dentro de um `spawn_blocking` — o event loop do tokio nunca vê libgit2.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use git2::{ErrorCode, Oid, Repository, Sort};

use crate::model::{self, Commit, Head, LogPage, RepoInfo};

/// Teto da pré-alocação da página. O `limit` vem clampado da rota, mas este crate é uma
/// biblioteca: um chamador distraído não pode pedir uma alocação de gigabytes.
const MAX_PREALLOC: usize = 1024;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("{0} não é um repositório git")]
    NotARepository(String),
    #[error("não consegui ler o repositório")]
    Read(#[source] git2::Error),
    #[error("cursor de log inválido")]
    InvalidCursor,
}

/// Uma página do log. `cursor` ausente é a primeira.
#[derive(Debug, Clone)]
pub struct LogQuery {
    pub limit: usize,
    pub cursor: Option<String>,
}

/// O que o resto do programa pode perguntar a um repositório.
///
/// Cresce a cada bloco (diff, blame, refs). Hoje responde quem ele é e a página do log.
pub trait RepoRead: Send + Sync {
    fn info(&self) -> Result<RepoInfo, GitError>;

    /// Página do log a partir de `HEAD`, ou da fronteira guardada no cursor.
    fn log(&self, query: &LogQuery) -> Result<LogPage, GitError>;
}

/// Implementação sobre libgit2.
///
/// Guarda o caminho, não o `Repository`: o handle do libgit2 não atravessa thread, e este tipo
/// precisa viver dentro do estado compartilhado do servidor. Abrir custa um punhado de `stat`;
/// quando isso pesar (Bloco D), entra o pool de handles por repo com semáforo.
pub struct Git2Repo {
    path: PathBuf,
}

impl Git2Repo {
    /// Abre **exatamente** o caminho dado, sem procurar para cima.
    ///
    /// `Repository::discover` subiria a árvore até achar um `.git`, e uma pasta qualquer
    /// dentro de um repositório passaria a "ser" o repositório. Num app cujo confinamento é
    /// por caminho, isso é o tipo de surpresa que não vale a conveniência: o navegador de
    /// pastas já marca quais diretórios são repositórios.
    pub fn open(path: &Path) -> Result<Self, GitError> {
        Repository::open(path).map_err(|err| match err.code() {
            ErrorCode::NotFound => GitError::NotARepository(path.display().to_string()),
            _ => GitError::Read(err),
        })?;

        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl RepoRead for Git2Repo {
    fn info(&self) -> Result<RepoInfo, GitError> {
        let repo = Repository::open(&self.path).map_err(GitError::Read)?;

        let bare = repo.is_bare();
        // Em bare não há worktree; o próprio gitdir é o repositório.
        let root = if bare {
            repo.path()
        } else {
            repo.workdir().unwrap_or_else(|| repo.path())
        };

        // O libgit2 devolve worktree e gitdir com barra no fim (`/Users/x/repo/`). Recolher as
        // componentes tira essa barra, e é o que faz o caminho aqui bater byte a byte com o
        // que o `canonicalize` do servidor produz — que é de onde sai o `repo_id`.
        let root: PathBuf = root.components().collect();

        let head = head_of(&repo)?;

        Ok(RepoInfo {
            name: display_name(&root),
            path: root.to_string_lossy().into_owned(),
            bare,
            detached: matches!(head, Head::Detached { .. }),
            branch: head.label(),
            head,
        })
    }

    fn log(&self, query: &LogQuery) -> Result<LogPage, GitError> {
        let repo = Repository::open(&self.path).map_err(GitError::Read)?;

        let tips = match &query.cursor {
            Some(cursor) => cursor_tips(&repo, cursor)?,
            None => head_tip(&repo)?.into_iter().collect(),
        };

        // Repositório sem commit nenhum, ou fronteira que só tinha objetos ausentes: página
        // vazia é a resposta certa, não erro.
        if tips.is_empty() {
            return Ok(LogPage {
                commits: Vec::new(),
                next_cursor: None,
            });
        }

        let mut walk = repo.revwalk().map_err(GitError::Read)?;
        // `TOPOLOGICAL` é o que torna a paginação por fronteira correta: um commit só sai
        // depois de todos os filhos dele, então nenhum commit já emitido pode reaparecer
        // como ancestral de quem ficou na fronteira. `TIME` desempata pela data.
        walk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)
            .map_err(GitError::Read)?;
        for tip in &tips {
            walk.push(*tip).map_err(GitError::Read)?;
        }

        let mut commits = Vec::with_capacity(query.limit.min(MAX_PREALLOC));
        let mut emitted: HashSet<Oid> = HashSet::with_capacity(query.limit.min(MAX_PREALLOC));
        let mut discovered: Vec<Oid> = Vec::new();

        for oid in walk.by_ref().take(query.limit) {
            let oid = oid.map_err(GitError::Read)?;
            let commit = repo.find_commit(oid).map_err(GitError::Read)?;

            emitted.insert(oid);
            discovered.extend(commit.parent_ids());
            commits.push(to_commit(&commit));
        }

        // A fronteira são os pais descobertos e ainda não emitidos, **mais** os tips que a
        // página não alcançou (com muitas pontas, a página pode acabar antes de tocar uma
        // delas — e sem isso o histórico inteiro por trás dessa ponta sumiria).
        let odb = repo.odb().map_err(GitError::Read)?;
        let mut seen = HashSet::new();
        let frontier: Vec<String> = discovered
            .into_iter()
            .chain(tips)
            .filter(|oid| !emitted.contains(oid) && seen.insert(*oid))
            // Em clone raso o pai enxertado não existe no odb. Empurrá-lo na página seguinte
            // faria o revwalk falhar inteiro; a borda do histórico raso é o fim da paginação.
            .filter(|oid| odb.exists(*oid))
            .map(|oid| oid.to_string())
            .collect();

        Ok(LogPage {
            next_cursor: (!frontier.is_empty()).then(|| model::encode_cursor(&frontier)),
            commits,
        })
    }
}

/// Commit apontado pelo `HEAD`. `None` em repositório sem commit nenhum.
fn head_tip(repo: &Repository) -> Result<Option<Oid>, GitError> {
    match repo.head() {
        Ok(reference) => Ok(Some(
            reference.peel_to_commit().map_err(GitError::Read)?.id(),
        )),
        Err(err) if err.code() == ErrorCode::UnbornBranch => Ok(None),
        Err(err) => Err(GitError::Read(err)),
    }
}

/// Fronteira guardada no cursor, conferida contra este repositório.
///
/// Cursor de outro repositório, ou de um objeto que sumiu num `gc`, é pedido inválido — não
/// falha de leitura. A diferença aparece na rota: 400, não 500.
fn cursor_tips(repo: &Repository, cursor: &str) -> Result<Vec<Oid>, GitError> {
    let frontier = model::decode_cursor(cursor).ok_or(GitError::InvalidCursor)?;

    frontier
        .iter()
        .map(|hex| {
            let oid = Oid::from_str(hex).map_err(|_| GitError::InvalidCursor)?;
            repo.find_commit(oid).map_err(|_| GitError::InvalidCursor)?;
            Ok(oid)
        })
        .collect()
}

fn to_commit(commit: &git2::Commit<'_>) -> Commit {
    let author = commit.author();

    Commit {
        oid: commit.id().to_string(),
        parents: commit.parent_ids().map(|id| id.to_string()).collect(),
        // Nome, e-mail e mensagem podem não ser UTF-8 (repositório antigo, autor com encoding
        // legado). Lossy em vez de `Option`: perder um acento é melhor do que a linha do log
        // sumir, e melhor ainda do que a página inteira falhar por causa de um commit de 2009.
        author: String::from_utf8_lossy(author.name_bytes()).into_owned(),
        email: String::from_utf8_lossy(author.email_bytes()).into_owned(),
        time: author.when().seconds(),
        offset: author.when().offset_minutes(),
        summary: String::from_utf8_lossy(commit.summary_bytes().unwrap_or_default()).into_owned(),
    }
}

fn head_of(repo: &Repository) -> Result<Head, GitError> {
    let reference = match repo.head() {
        Ok(reference) => reference,
        // Repositório recém-criado: `HEAD` já aponta para `refs/heads/<branch>`, mas a
        // referência ainda não existe. Não é erro, é o estado normal depois de um `git init`.
        Err(err) if err.code() == ErrorCode::UnbornBranch => {
            return Ok(Head::Unborn {
                name: unborn_branch(repo),
            })
        }
        Err(err) => return Err(GitError::Read(err)),
    };

    let commit = reference
        .peel_to_commit()
        .map_err(GitError::Read)?
        .id()
        .to_string();

    // `shorthand` devolve `Result` porque o nome da ref pode não ser UTF-8. Uma ref com nome
    // ilegível não é motivo para recusar o repositório: cai em detached, que mostra o hash.
    Ok(match reference.shorthand() {
        Ok(name) if reference.is_branch() => Head::Branch {
            name: name.to_owned(),
            commit,
        },
        _ => Head::Detached { commit },
    })
}

/// Nome da branch para a qual o `HEAD` simbólico aponta antes do primeiro commit.
fn unborn_branch(repo: &Repository) -> String {
    // `HEAD` ilegível num repositório que acabou de abrir é raro o bastante para um fallback:
    // mostrar "HEAD" é melhor do que recusar a abrir o repositório.
    let fallback = || "HEAD".to_owned();

    let Ok(head) = repo.find_reference("HEAD") else {
        return fallback();
    };

    head.symbolic_target()
        .ok()
        .flatten()
        .and_then(|target| target.strip_prefix("refs/heads/"))
        .map(str::to_owned)
        .unwrap_or_else(fallback)
}

/// `~/Git/porcelain` → `porcelain`; `~/Git/porcelain.git` (bare) → `porcelain`.
fn display_name(root: &Path) -> String {
    let name = root
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.to_string_lossy().into_owned());

    // Em bare o caminho termina em `.git`; em worktree normal `root` é a worktree, então esta
    // poda não pega o gitdir por engano.
    name.strip_suffix(".git").unwrap_or(&name).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .to_path_buf()
    }

    fn query(limit: usize, cursor: Option<&str>) -> LogQuery {
        LogQuery {
            limit,
            cursor: cursor.map(str::to_owned),
        }
    }

    #[test]
    fn abre_o_repositorio_do_projeto_e_le_o_head() {
        let repo = Git2Repo::open(&project_root()).unwrap();
        let info = repo.info().unwrap();

        assert_eq!(info.name, "git_helper");
        assert!(!info.bare);
        assert!(!info.branch.is_empty());
    }

    #[test]
    fn nao_sobe_a_arvore_procurando_repositorio() {
        // `crates/` está dentro do repositório, mas não *é* o repositório.
        let inside = project_root().join("crates");
        assert!(matches!(
            Git2Repo::open(&inside),
            Err(GitError::NotARepository(_))
        ));
    }

    #[test]
    fn nome_de_bare_perde_o_sufixo_git() {
        assert_eq!(display_name(Path::new("/srv/porcelain.git")), "porcelain");
        assert_eq!(display_name(Path::new("/srv/porcelain")), "porcelain");
    }

    #[test]
    fn a_primeira_pagina_traz_o_head_e_o_commit_raiz_nao_tem_pai() {
        let repo = Git2Repo::open(&project_root()).unwrap();

        let page = repo.log(&query(500, None)).unwrap();
        let head = repo.info().unwrap().head;

        assert!(!page.commits.is_empty());
        if let Head::Branch { commit, .. } | Head::Detached { commit } = head {
            assert_eq!(page.commits[0].oid, commit);
        }
        assert!(!page.commits[0].author.is_empty());
        assert!(page.commits.last().unwrap().parents.is_empty());
    }

    #[test]
    fn paginar_pelo_cursor_da_a_mesma_sequencia_de_uma_pagina_so() {
        let repo = Git2Repo::open(&project_root()).unwrap();

        let inteiro = repo.log(&query(10_000, None)).unwrap();
        assert!(
            inteiro.next_cursor.is_none(),
            "10k commits cobrem este repositório"
        );
        let esperado: Vec<_> = inteiro.commits.iter().map(|c| c.oid.clone()).collect();

        // De um em um, seguindo o cursor. É o caso extremo da paginação: se a fronteira
        // estiver errada, ou some commit ou aparece repetido.
        let mut aos_pedacos = Vec::new();
        let mut cursor = None;
        loop {
            let page = repo.log(&query(1, cursor.as_deref())).unwrap();
            aos_pedacos.extend(page.commits.iter().map(|c| c.oid.clone()));

            match page.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }

            assert!(
                aos_pedacos.len() <= esperado.len(),
                "o cursor nunca termina"
            );
        }

        assert_eq!(aos_pedacos, esperado);
    }

    #[test]
    fn cursor_adulterado_e_recusado_como_pedido_invalido() {
        let repo = Git2Repo::open(&project_root()).unwrap();

        for cursor in [
            "",
            "v1.",
            "lixo",
            "v1.naoehumoid",
            // Formato certo, objeto que não existe neste repositório.
            "v1.0000000000000000000000000000000000000001",
        ] {
            assert!(
                matches!(
                    repo.log(&query(10, Some(cursor))),
                    Err(GitError::InvalidCursor)
                ),
                "cursor {cursor:?} deveria ser recusado"
            );
        }
    }
}
