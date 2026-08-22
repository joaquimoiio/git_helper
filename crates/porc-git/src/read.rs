//! Leitura de repositório, atrás do trait `RepoRead`.
//!
//! O trait não é cerimônia: o plano B do hot path de leitura é trocar libgit2 por `gix`, e a
//! única forma de essa troca não tocar rota nem UI é nenhuma delas conhecer `git2`. Hoje há
//! uma implementação só.
//!
//! Nada aqui é `Sync` do lado do libgit2 (`git2::Repository` não é), e tudo é bloqueante. Quem
//! chama tem que estar dentro de um `spawn_blocking` — o event loop do tokio nunca vê libgit2.

use std::path::{Path, PathBuf};

use git2::{ErrorCode, Oid, Repository, Sort};

use crate::model::{
    self, Commit, CommitDetail, DiffHunk, DiffLine, DiffLineKind, FileChange, FileChangeKind,
    FileDiff, Head, LogPage, RefKind, RefMarker, RepoInfo, RepoState, Signature,
};

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
    #[error("commit inválido")]
    InvalidCommit,
    #[error("este arquivo não foi tocado por este commit")]
    FileNotInCommit,
}

/// Uma página do log. `cursor` ausente é a primeira.
#[derive(Debug, Clone)]
pub struct LogQuery {
    pub limit: usize,
    pub cursor: Option<String>,
}

/// Uma linha mínima para a indexação de busca (Passo 42/43) — não é o `Commit` da UI: sem
/// lane, sem parents, sem fuso. Índice de busca não precisa de nenhum dos três.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub oid: String,
    pub author: String,
    pub email: String,
    pub time: i64,
    pub summary: String,
}

/// O que o resto do programa pode perguntar a um repositório.
///
/// Cresce a cada bloco (diff, blame, refs). Hoje responde quem ele é e a página do log.
pub trait RepoRead: Send + Sync {
    fn info(&self) -> Result<RepoInfo, GitError>;

    /// Página do log a partir de `HEAD`, ou da fronteira guardada no cursor.
    fn log(&self, query: &LogQuery) -> Result<LogPage, GitError>;

    /// Toda ponta do repositório (branches locais, remotas, tags) e o `HEAD` destacado, se for
    /// o caso. Não pagina: o número de pontas não cresce com o histórico.
    fn refs(&self) -> Result<Vec<RefMarker>, GitError>;

    /// Mensagem completa, assinaturas e o diffstat de um commit contra o primeiro pai.
    fn commit_detail(&self, oid: &str) -> Result<CommitDetail, GitError>;

    /// Hunks de **um** arquivo dentro do commit, contra o primeiro pai — o patch de verdade,
    /// sob demanda. `path` é o caminho atual do arquivo (o novo lado de um rename).
    fn commit_diff(&self, oid: &str, path: &str) -> Result<FileDiff, GitError>;

    /// Varre o histórico completo a partir do `HEAD`, chamando `on_commit` uma vez por commit —
    /// para a indexação de busca, não para o log (que pagina e calcula lanes). Devolve o oid do
    /// `HEAD` varrido, `None` em repositório sem commit nenhum.
    ///
    /// `on_commit` devolvendo `false` interrompe a varredura cedo — é o cancelamento do job de
    /// indexação, checado a cada commit em vez de só entre lotes.
    fn walk_for_index(
        &self,
        on_commit: &mut dyn FnMut(IndexEntry) -> bool,
    ) -> Result<Option<String>, GitError>;

    /// Nomes que começam com `prefix`, dentro da árvore de `HEAD` — para o autocomplete do
    /// filtro por caminho (Passo 45). Só o nível imediato do diretório que `prefix` indica, não
    /// a árvore inteira: um monorepo pode ter dezenas de milhares de arquivos, e não há por
    /// quê descer em cada subpasta só para descartar a maioria por não bater o prefixo.
    /// Diretório vem com `/` no fim, arquivo sem — é o que diz à UI se dá para continuar
    /// completando ali dentro.
    fn list_paths(&self, prefix: &str, limit: usize) -> Result<Vec<String>, GitError>;

    /// Operação em andamento (merge, rebase, cherry-pick, revert, bisect) — o que falta saber
    /// antes de um `commit` normal (Bloco E) ser seguro, e o que o Bloco G usa para desenhar a
    /// barra de "resolvendo conflitos". Diferente do resto do `status` (Passo 47), que é
    /// shell-out: isto é um `git2::Repository::state()` — barato, mas uma chamada à parte.
    fn state(&self) -> Result<RepoState, GitError>;
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
        let odb = repo.odb().map_err(GitError::Read)?;

        // `lanes[i]` é o oid que a coluna `i` espera como próximo commit dela, ou `None` se a
        // coluna está livre. Primeira página: uma lane só, esperando o `HEAD`. Páginas
        // seguintes: o estado inteiro volta do cursor — é o que faz o desenho continuar sem
        // recomeçar (comentário completo em `model::encode_cursor`).
        let mut lanes: Vec<Option<Oid>> = match &query.cursor {
            Some(cursor) => decode_lanes(&repo, cursor)?,
            None => head_tip(&repo)?.into_iter().map(Some).collect(),
        };

        // Repositório sem commit nenhum, ou cursor cuja fronteira só tinha objetos ausentes:
        // página vazia é a resposta certa, não erro.
        if lanes.iter().all(Option::is_none) {
            return Ok(LogPage {
                commits: Vec::new(),
                next_cursor: None,
            });
        }

        let mut walk = repo.revwalk().map_err(GitError::Read)?;
        // `TOPOLOGICAL` é o que torna a paginação por lanes correta: um commit só sai depois
        // de todos os filhos dele, então toda lane que vai esperá-lo já o fez antes de ele ser
        // emitido. `TIME` desempata pela data.
        walk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)
            .map_err(GitError::Read)?;
        for oid in lanes.iter().flatten() {
            walk.push(*oid).map_err(GitError::Read)?;
        }

        let mut commits = Vec::with_capacity(query.limit.min(MAX_PREALLOC));

        for oid in walk.by_ref().take(query.limit) {
            let oid = oid.map_err(GitError::Read)?;
            let commit = repo.find_commit(oid).map_err(GitError::Read)?;

            // A lane mais à esquerda que esperava este commit fica com ele; qualquer outra
            // lane que também esperava o mesmo oid converge aqui e se liberta — é o ponto em
            // que duas colunas voltam a ser uma, por exemplo a base de uma branch que mesclou.
            let mut waiting = lanes
                .iter()
                .enumerate()
                .filter(|(_, slot)| **slot == Some(oid))
                .map(|(index, _)| index)
                .collect::<Vec<_>>()
                .into_iter();
            let lane = waiting
                .next()
                .expect("commit emitido tem que ter sido esperado por uma lane");
            for extra in waiting {
                lanes[extra] = None;
            }

            let parent_ids: Vec<Oid> = commit.parent_ids().collect();
            let mut parent_lanes = Vec::with_capacity(parent_ids.len());
            // Se nenhum pai reclamar a lane atual (raiz, ou clone raso cujo primeiro pai não
            // existe localmente), ela se liberta: a linha desta coluna termina aqui.
            let mut lane_continues = false;

            for (index, parent) in parent_ids.iter().enumerate() {
                // Em clone raso o pai enxertado não existe no odb. Empurrá-lo faria o revwalk
                // falhar inteiro; a borda do histórico raso é onde a aresta para de verdade,
                // não uma página seguinte.
                if !odb.exists(*parent) {
                    parent_lanes.push(None);
                    continue;
                }

                let target = if index == 0 {
                    lane_continues = true;
                    lane
                } else {
                    free_lane(&mut lanes)
                };

                lanes[target] = Some(*parent);
                parent_lanes.push(Some(target));
            }

            if !lane_continues {
                lanes[lane] = None;
            }

            commits.push(to_commit(&commit, lane, parent_lanes));
        }

        let has_pending = lanes.iter().any(Option::is_some);
        let next_cursor = has_pending.then(|| {
            let slots: Vec<model::LaneSlot> = lanes
                .iter()
                .map(|slot| match slot {
                    Some(oid) => model::LaneSlot::Waiting(oid.to_string()),
                    None => model::LaneSlot::Free,
                })
                .collect();
            model::encode_cursor(&slots)
        });

        Ok(LogPage {
            commits,
            next_cursor,
        })
    }

    fn refs(&self) -> Result<Vec<RefMarker>, GitError> {
        let repo = Repository::open(&self.path).map_err(GitError::Read)?;

        // Nome curto da branch atual, se o `HEAD` for uma: é contra isso que cada branch
        // local é comparada para decidir `is_head`. `HEAD` destacado vira um marcador à
        // parte, depois da varredura.
        let head_branch = match repo.head() {
            Ok(reference) if reference.is_branch() => reference.shorthand().ok().map(str::to_owned),
            _ => None,
        };

        let mut markers = Vec::new();

        for reference in repo.references().map_err(GitError::Read)? {
            let reference = reference.map_err(GitError::Read)?;

            // Refs simbólicas — o `origin/HEAD` que aponta para a branch padrão do remoto —
            // não são uma ponta própria: a branch real para a qual apontam já aparece na
            // própria varredura.
            if reference.kind() != Some(git2::ReferenceType::Direct) {
                continue;
            }

            let Ok(name) = std::str::from_utf8(reference.name_bytes()) else {
                continue;
            };

            let (kind, short) = if let Some(short) = name.strip_prefix("refs/heads/") {
                (RefKind::Branch, short)
            } else if let Some(short) = name.strip_prefix("refs/remotes/") {
                (RefKind::Remote, short)
            } else if let Some(short) = name.strip_prefix("refs/tags/") {
                (RefKind::Tag, short)
            } else {
                // `refs/stash`, `refs/notes/…`: não são pontas que o log marca.
                continue;
            };

            // `peel_to_commit` resolve tanto uma tag anotada (objeto tag, que aponta para o
            // commit) quanto uma leve (já é o commit) com o mesmo código. Tag para blob ou
            // árvore — rara, mas válida em git — não tem commit para marcar, e é descartada.
            let Ok(commit) = reference.peel_to_commit() else {
                continue;
            };

            markers.push(RefMarker {
                name: short.to_owned(),
                kind,
                commit: commit.id().to_string(),
                is_head: kind == RefKind::Branch && head_branch.as_deref() == Some(short),
            });
        }

        // `HEAD` destacado não tem branch para carregar o marcador — a própria ponta é
        // "HEAD". Repositório recém-criado (`UnbornBranch`) não tem commit nenhum para
        // apontar, e fica sem marcador de jeito nenhum, corretamente.
        if let Ok(reference) = repo.head() {
            if !reference.is_branch() {
                if let Ok(commit) = reference.peel_to_commit() {
                    markers.push(RefMarker {
                        name: "HEAD".to_owned(),
                        kind: RefKind::Head,
                        commit: commit.id().to_string(),
                        is_head: true,
                    });
                }
            }
        }

        Ok(markers)
    }

    fn commit_detail(&self, oid: &str) -> Result<CommitDetail, GitError> {
        let repo = Repository::open(&self.path).map_err(GitError::Read)?;

        let oid = Oid::from_str(oid).map_err(|_| GitError::InvalidCommit)?;
        let commit = repo.find_commit(oid).map_err(|_| GitError::InvalidCommit)?;

        let tree = commit.tree().map_err(GitError::Read)?;
        // Raiz: sem pai, o diff é contra a árvore vazia — todo arquivo aparece como `Added`,
        // igual ao que `git show` faz no primeiro commit de um repositório.
        let parent_tree = match commit.parent(0) {
            Ok(parent) => Some(parent.tree().map_err(GitError::Read)?),
            Err(_) => None,
        };

        let mut diff = repo
            .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)
            .map_err(GitError::Read)?;
        diff.find_similar(None).map_err(GitError::Read)?;

        // `Diff::stats()` só dá o agregado; a contagem por arquivo sai contando `+`/`-` linha a
        // linha, por caminho — é o único jeito que o git2 expõe isso.
        let mut per_file: std::collections::HashMap<String, (usize, usize)> =
            std::collections::HashMap::new();
        diff.foreach(
            &mut |_delta, _progress| true,
            None,
            None,
            Some(&mut |delta, _hunk, line| {
                let origin = line.origin();
                if origin != '+' && origin != '-' {
                    return true;
                }

                let path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .map(|path| path.to_string_lossy().into_owned())
                    .unwrap_or_default();

                let entry = per_file.entry(path).or_insert((0, 0));
                if origin == '+' {
                    entry.0 += 1;
                } else {
                    entry.1 += 1;
                }
                true
            }),
        )
        .map_err(GitError::Read)?;

        let mut files = Vec::with_capacity(diff.deltas().len());
        let (mut insertions, mut deletions) = (0, 0);

        for delta in diff.deltas() {
            let kind = match delta.status() {
                git2::Delta::Added => FileChangeKind::Added,
                git2::Delta::Deleted => FileChangeKind::Deleted,
                git2::Delta::Renamed => FileChangeKind::Renamed,
                git2::Delta::Copied => FileChangeKind::Copied,
                git2::Delta::Typechange => FileChangeKind::Typechange,
                // `Modified` é o padrão para o resto (inclui os estados que `diff_tree_to_tree`
                // não produz, como `Untracked`/`Ignored` — só existem contra o worktree).
                _ => FileChangeKind::Modified,
            };

            let new_path = delta
                .new_file()
                .path()
                .map(|path| path.to_string_lossy().into_owned());
            let old_path = delta
                .old_file()
                .path()
                .map(|path| path.to_string_lossy().into_owned());

            let path = new_path
                .clone()
                .or_else(|| old_path.clone())
                .unwrap_or_default();
            let (file_insertions, file_deletions) = per_file.get(&path).copied().unwrap_or((0, 0));

            insertions += file_insertions;
            deletions += file_deletions;

            files.push(FileChange {
                old_path: matches!(kind, FileChangeKind::Renamed | FileChangeKind::Copied)
                    .then_some(old_path)
                    .flatten()
                    .filter(|old| Some(old) != new_path.as_ref()),
                path,
                kind,
                insertions: file_insertions,
                deletions: file_deletions,
                binary: delta.flags().contains(git2::DiffFlags::BINARY),
            });
        }

        let author = to_signature(&commit.author());
        let committer = to_signature(&commit.committer());

        Ok(CommitDetail {
            oid: commit.id().to_string(),
            parents: commit.parent_ids().map(|id| id.to_string()).collect(),
            author,
            committer,
            message: String::from_utf8_lossy(commit.message_bytes()).into_owned(),
            insertions,
            deletions,
            files,
        })
    }

    fn commit_diff(&self, oid: &str, path: &str) -> Result<FileDiff, GitError> {
        let repo = Repository::open(&self.path).map_err(GitError::Read)?;

        let oid = Oid::from_str(oid).map_err(|_| GitError::InvalidCommit)?;
        let commit = repo.find_commit(oid).map_err(|_| GitError::InvalidCommit)?;

        let tree = commit.tree().map_err(GitError::Read)?;
        let parent_tree = match commit.parent(0) {
            Ok(parent) => Some(parent.tree().map_err(GitError::Read)?),
            Err(_) => None,
        };

        let mut diff = repo
            .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)
            .map_err(GitError::Read)?;
        diff.find_similar(None).map_err(GitError::Read)?;

        // O caminho pedido é sempre o lado **novo** de um delta (o próprio `path` de
        // `FileChange` do Passo 40 já é esse lado). O antigo só entra num rename, e mesmo
        // assim é o `old_path` que a UI mostrou — nunca o que ela manda de volta.
        let index = diff
            .deltas()
            .position(|delta| {
                delta
                    .new_file()
                    .path()
                    .map(|p| p.to_string_lossy())
                    .as_deref()
                    == Some(path)
            })
            .ok_or(GitError::FileNotInCommit)?;

        let patch = git2::Patch::from_diff(&diff, index)
            .map_err(GitError::Read)?
            .ok_or(GitError::FileNotInCommit)?;

        // O `delta` de antes do patch nunca tem a flag `BINARY`: libgit2 só inspeciona o
        // conteúdo (o teste de heurística, um `\0` nos primeiros bytes) ao montar o patch de
        // verdade, e é o `delta` **do patch** que sai marcado — não o do diff que o gerou.
        if patch.delta().flags().contains(git2::DiffFlags::BINARY) {
            return Ok(FileDiff::Binary);
        }

        let mut hunks = Vec::with_capacity(patch.num_hunks());

        for hunk_index in 0..patch.num_hunks() {
            let (hunk, line_count) = patch.hunk(hunk_index).map_err(GitError::Read)?;

            // O cabeçalho do git ("@@ -a,b +c,d @@") já vem em UTF-8 na prática — é gerado
            // pelo próprio libgit2, não copiado do arquivo. `lossy` aqui é só defensivo.
            let header = String::from_utf8_lossy(hunk.header())
                .trim_end_matches('\n')
                .to_owned();
            let mut lines = Vec::with_capacity(line_count);

            for line_index in 0..line_count {
                let line = patch
                    .line_in_hunk(hunk_index, line_index)
                    .map_err(GitError::Read)?;

                // Diferente do cabeçalho: o conteúdo da linha vem direto do arquivo do
                // usuário. Um encoding legado aqui não é binário (o git não marcou como
                // tal), mas também não é UTF-8 — e mostrar como `lossy` produziria mojibake
                // em vez de um patch legível. O commit inteiro vira `NotUtf8` nesse caso: um
                // arquivo pela metade em UTF-8 e pela metade em mojibake seria pior que os
                // dois estados claros.
                let Ok(content) = std::str::from_utf8(line.content()) else {
                    return Ok(FileDiff::NotUtf8);
                };

                lines.push(DiffLine {
                    kind: match line.origin() {
                        '+' => DiffLineKind::Addition,
                        '-' => DiffLineKind::Deletion,
                        _ => DiffLineKind::Context,
                    },
                    old_lineno: line.old_lineno(),
                    new_lineno: line.new_lineno(),
                    content: content.trim_end_matches('\n').to_owned(),
                });
            }

            hunks.push(DiffHunk { header, lines });
        }

        Ok(FileDiff::Text { hunks })
    }

    fn walk_for_index(
        &self,
        on_commit: &mut dyn FnMut(IndexEntry) -> bool,
    ) -> Result<Option<String>, GitError> {
        let repo = Repository::open(&self.path).map_err(GitError::Read)?;

        let Some(tip) = head_tip(&repo)? else {
            return Ok(None);
        };

        let mut walk = repo.revwalk().map_err(GitError::Read)?;
        walk.push(tip).map_err(GitError::Read)?;
        // A ordem não importa para indexação — cada linha entra sozinha na tabela, sem
        // depender da vizinha. `TIME` é mais barato que `TOPOLOGICAL`, que só existe para as
        // lanes do log terem sentido.
        walk.set_sorting(Sort::TIME).map_err(GitError::Read)?;

        for oid in walk {
            let oid = oid.map_err(GitError::Read)?;
            let commit = repo.find_commit(oid).map_err(GitError::Read)?;
            let author = commit.author();

            let entry = IndexEntry {
                oid: commit.id().to_string(),
                author: String::from_utf8_lossy(author.name_bytes()).into_owned(),
                email: String::from_utf8_lossy(author.email_bytes()).into_owned(),
                time: author.when().seconds(),
                summary: String::from_utf8_lossy(commit.summary_bytes().unwrap_or_default())
                    .into_owned(),
            };

            if !on_commit(entry) {
                break;
            }
        }

        Ok(Some(tip.to_string()))
    }

    fn list_paths(&self, prefix: &str, limit: usize) -> Result<Vec<String>, GitError> {
        let repo = Repository::open(&self.path).map_err(GitError::Read)?;

        let Some(tip) = head_tip(&repo)? else {
            return Ok(Vec::new());
        };
        let root_tree = repo
            .find_commit(tip)
            .map_err(GitError::Read)?
            .tree()
            .map_err(GitError::Read)?;

        // `dir` é tudo antes da última `/` (a pasta já fechada); `partial` é o que ainda está
        // sendo digitado dentro dela. Sem `/` nenhuma, `dir` é a raiz.
        let (dir, partial) = match prefix.rfind('/') {
            Some(at) => (&prefix[..at], &prefix[at + 1..]),
            None => ("", prefix),
        };

        let tree = if dir.is_empty() {
            root_tree
        } else {
            // Prefixo aponta para uma pasta que não existe (mais provável: o usuário ainda
            // está digitando, ou apagou uma letra e o caminho ficou momentaneamente inválido)
            // ou para um arquivo, não pasta: os dois casos são "nada para completar", não erro.
            match root_tree
                .get_path(Path::new(dir))
                .ok()
                .and_then(|entry| entry.to_object(&repo).ok())
                .and_then(|object| object.into_tree().ok())
            {
                Some(tree) => tree,
                None => return Ok(Vec::new()),
            }
        };

        let mut paths = Vec::with_capacity(limit.min(MAX_PREALLOC));
        for entry in tree.iter() {
            if paths.len() >= limit {
                break;
            }

            let Ok(name) = entry.name() else { continue };
            if !name.starts_with(partial) {
                continue;
            }

            let full = if dir.is_empty() {
                name.to_owned()
            } else {
                format!("{dir}/{name}")
            };

            // Diretório ganha a `/` no fim: é o sinal para a UI de que dá para continuar
            // completando ali dentro, em vez de oferecer como resultado final.
            paths.push(if entry.kind() == Some(git2::ObjectType::Tree) {
                format!("{full}/")
            } else {
                full
            });
        }

        Ok(paths)
    }

    fn state(&self) -> Result<RepoState, GitError> {
        let repo = Repository::open(&self.path).map_err(GitError::Read)?;

        Ok(match repo.state() {
            git2::RepositoryState::Clean => RepoState::Clean,
            git2::RepositoryState::Merge => RepoState::Merge,
            git2::RepositoryState::Revert | git2::RepositoryState::RevertSequence => {
                RepoState::Revert
            }
            git2::RepositoryState::CherryPick | git2::RepositoryState::CherryPickSequence => {
                RepoState::CherryPick
            }
            git2::RepositoryState::Bisect => RepoState::Bisect,
            git2::RepositoryState::Rebase
            | git2::RepositoryState::RebaseInteractive
            | git2::RepositoryState::RebaseMerge => RepoState::Rebase,
            // `git am` em andamento não é nenhum dos cinco estados que o Bloco G trata — mais
            // perto de "nada bloqueando um commit normal" do que fingir ser um rebase.
            git2::RepositoryState::ApplyMailbox | git2::RepositoryState::ApplyMailboxOrRebase => {
                RepoState::Clean
            }
        })
    }
}

fn to_signature(sig: &git2::Signature<'_>) -> Signature {
    Signature {
        name: String::from_utf8_lossy(sig.name_bytes()).into_owned(),
        email: String::from_utf8_lossy(sig.email_bytes()).into_owned(),
        time: sig.when().seconds(),
        offset: sig.when().offset_minutes(),
    }
}

/// Primeiro slot livre em `lanes`, alocando uma coluna nova no final se não houver nenhum.
fn free_lane(lanes: &mut Vec<Option<Oid>>) -> usize {
    match lanes.iter().position(Option::is_none) {
        Some(index) => index,
        None => {
            lanes.push(None);
            lanes.len() - 1
        }
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

/// Estado das lanes guardado no cursor, conferido contra este repositório.
///
/// Cursor de outro repositório, ou de um objeto que sumiu num `gc`, é pedido inválido — não
/// falha de leitura. A diferença aparece na rota: 400, não 500.
fn decode_lanes(repo: &Repository, cursor: &str) -> Result<Vec<Option<Oid>>, GitError> {
    let slots = model::decode_cursor(cursor).ok_or(GitError::InvalidCursor)?;

    slots
        .into_iter()
        .map(|slot| match slot {
            model::LaneSlot::Free => Ok(None),
            model::LaneSlot::Waiting(hex) => {
                let oid = Oid::from_str(&hex).map_err(|_| GitError::InvalidCursor)?;
                repo.find_commit(oid).map_err(|_| GitError::InvalidCursor)?;
                Ok(Some(oid))
            }
        })
        .collect()
}

fn to_commit(commit: &git2::Commit<'_>, lane: usize, parent_lanes: Vec<Option<usize>>) -> Commit {
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
        lane,
        parent_lanes,
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
            "v2.",
            "lixo",
            "v2.naoehumoid",
            // Formato certo, objeto que não existe neste repositório.
            "v2.0000000000000000000000000000000000000001",
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

    /// Repositório sintético com um fork e um merge:
    ///
    /// ```text
    /// root -- a -- b ------ merge -- c   (linha principal)
    ///          \            /
    ///           f1 ------- f2            (feature)
    /// ```
    ///
    /// `a` é o ponto em que duas lanes precisam convergir de volta numa só (`b` e `f1` esperam
    /// o mesmo pai); `merge` é o ponto em que uma lane nova nasce (o segundo pai, `f2`, ainda
    /// não tem coluna). É o menor histórico que exercita as duas pontas do algoritmo de lanes.
    fn build_merge_repo(dir: &Path) -> Repository {
        let repo = Repository::init(dir).unwrap();

        // Bloco à parte: `tree`, o fecho `commit` e cada `git2::Commit` devolvido tomam
        // emprestado `repo`, e o empréstimo precisa acabar antes de devolvê-lo por valor.
        {
            let blob = repo.blob(b"conteudo").unwrap();
            let tree_id = {
                let mut builder = repo.treebuilder(None).unwrap();
                builder.insert("arquivo.txt", blob, 0o100644).unwrap();
                builder.write().unwrap()
            };
            let tree = repo.find_tree(tree_id).unwrap();

            // Relógio crescente por commit: a ordem de criação vira a ordem cronológica, o que
            // deixa o desempate do `Sort::TIME` previsível — mas nenhuma das duas asserções
            // abaixo depende disso, só da estrutura.
            let mut clock = 1_700_000_000i64;
            let mut commit = |msg: &str, parents: &[&git2::Commit], on_head: bool| {
                clock += 1;
                let sig =
                    git2::Signature::new("Teste", "teste@example.com", &git2::Time::new(clock, 0))
                        .unwrap();
                let update_ref = on_head.then_some("HEAD");
                let oid = repo
                    .commit(update_ref, &sig, &sig, msg, &tree, parents)
                    .unwrap();
                repo.find_commit(oid).unwrap()
            };

            let root = commit("root", &[], true);
            let a = commit("a", &[&root], true);
            let b = commit("b", &[&a], true);
            let f1 = commit("f1", &[&a], false);
            let f2 = commit("f2", &[&f1], false);
            let merge = commit("merge", &[&b, &f2], true);
            commit("c", &[&merge], true);
        }

        repo
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("porc-log-lanes-{name}"));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn lanes_convergem_no_fork_e_nascem_no_merge() {
        let dir = temp("estrutura");
        build_merge_repo(&dir);
        let repo = Git2Repo::open(&dir).unwrap();

        let page = repo.log(&query(100, None)).unwrap();
        assert!(
            page.next_cursor.is_none(),
            "7 commits cabem numa página de 100"
        );
        assert_eq!(page.commits.len(), 7);

        let by_summary: std::collections::HashMap<&str, &Commit> = page
            .commits
            .iter()
            .map(|c| (c.summary.as_str(), c))
            .collect();

        let merge = by_summary["merge"];
        // O merge tem dois pais, em duas colunas diferentes: uma continua a lane dele, a
        // outra é onde a lane da feature nasce.
        assert_eq!(merge.parent_lanes.len(), 2);
        assert_ne!(merge.parent_lanes[0], merge.parent_lanes[1]);
        assert!(merge.parent_lanes.iter().all(Option::is_some));

        let a = by_summary["a"];
        // `b` e `f1` esperavam o mesmo pai (`a`); a convergência deixa `a` na lane mais à
        // esquerda entre as duas, que é a lane principal (0) porque `b` é o primeiro pai do
        // merge.
        assert_eq!(a.lane, 0);

        let root = by_summary["root"];
        assert!(root.parents.is_empty());
        assert!(root.parent_lanes.is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn duas_paginas_encaixam_sem_descontinuidade_nas_lanes() {
        let dir = temp("paginado");
        build_merge_repo(&dir);
        let repo = Git2Repo::open(&dir).unwrap();

        let inteiro = repo.log(&query(100, None)).unwrap();
        let esperado: Vec<_> = inteiro
            .commits
            .iter()
            .map(|c| (c.oid.clone(), c.lane, c.parent_lanes.clone()))
            .collect();

        // De um em um: se o cursor perder informação de lane na emenda entre páginas, a coluna
        // de algum commit (ou de alguma aresta) diverge da versão calculada de uma vez só.
        let mut aos_pedacos = Vec::new();
        let mut cursor = None;
        loop {
            let page = repo.log(&query(1, cursor.as_deref())).unwrap();
            aos_pedacos.extend(
                page.commits
                    .iter()
                    .map(|c| (c.oid.clone(), c.lane, c.parent_lanes.clone())),
            );

            match page.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }

        assert_eq!(aos_pedacos, esperado);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn refs_do_projeto_marca_a_branch_atual() {
        let repo = Git2Repo::open(&project_root()).unwrap();
        let markers = repo.refs().unwrap();

        let main = markers
            .iter()
            .find(|marker| marker.kind == RefKind::Branch && marker.name == "main")
            .expect("este checkout tem a branch main");
        assert!(
            main.is_head,
            "main é a branch para a qual o HEAD deste checkout aponta"
        );

        assert!(
            markers
                .iter()
                .any(|marker| marker.kind == RefKind::Remote && marker.name == "origin/main"),
            "o remote origin/main deveria aparecer entre as pontas"
        );
    }

    #[test]
    fn tag_e_head_destacado_ganham_marcador() {
        let dir = temp("refs");
        let repo = Repository::init(&dir).unwrap();

        let commit_id = {
            let blob = repo.blob(b"conteudo").unwrap();
            let tree_id = {
                let mut builder = repo.treebuilder(None).unwrap();
                builder.insert("arquivo.txt", blob, 0o100644).unwrap();
                builder.write().unwrap()
            };
            let tree = repo.find_tree(tree_id).unwrap();
            let sig = git2::Signature::new(
                "Teste",
                "teste@example.com",
                &git2::Time::new(1_700_000_000, 0),
            )
            .unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "c", &tree, &[])
                .unwrap()
        };

        {
            let object = repo.find_object(commit_id, None).unwrap();
            repo.tag_lightweight("v1", &object, false).unwrap();
        }
        // Destacado depois do commit: sem isso o `HEAD` continuaria sendo a branch que o
        // `init` criou, e o marcador `Head` nunca nasceria.
        repo.set_head_detached(commit_id).unwrap();
        drop(repo);

        let repo = Git2Repo::open(&dir).unwrap();
        let markers = repo.refs().unwrap();

        assert!(markers
            .iter()
            .any(|marker| marker.kind == RefKind::Tag && marker.name == "v1"));

        let head = markers
            .iter()
            .find(|marker| marker.kind == RefKind::Head)
            .expect("HEAD destacado deveria ganhar um marcador próprio");
        assert!(head.is_head);
        assert_eq!(head.commit, commit_id.to_string());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn commit_invalido_e_recusado() {
        let repo = Git2Repo::open(&project_root()).unwrap();

        for oid in ["", "lixo", "0000000000000000000000000000000000000001"] {
            assert!(
                matches!(repo.commit_detail(oid), Err(GitError::InvalidCommit)),
                "oid {oid:?} deveria ser recusado"
            );
        }
    }

    #[test]
    fn commit_raiz_do_projeto_e_tudo_adicionado() {
        let repo = Git2Repo::open(&project_root()).unwrap();

        // O primeiro commit deste próprio repositório: sem pai, `git show --stat` confirma 15
        // arquivos, todos novos, 1385 inserções e zero remoções.
        let detail = repo
            .commit_detail("2cea64adc1a40be6a9388b50800daea31c850d84")
            .unwrap();

        assert!(detail.parents.is_empty());
        assert_eq!(detail.files.len(), 15);
        assert!(detail
            .files
            .iter()
            .all(|file| file.kind == FileChangeKind::Added));
        assert_eq!(detail.insertions, 1385);
        assert_eq!(detail.deletions, 0);

        let summed: usize = detail.files.iter().map(|file| file.insertions).sum();
        assert_eq!(
            summed, detail.insertions,
            "soma por arquivo bate com o agregado"
        );
    }

    #[test]
    fn commit_normal_soma_por_arquivo_bate_com_o_agregado() {
        let repo = Git2Repo::open(&project_root()).unwrap();

        // `git show --stat 0b61c6f`: 50 arquivos, 5763 inserções, 9 remoções.
        let detail = repo
            .commit_detail("0b61c6f62675d70dec6e486da71f89b9cd6a6561")
            .unwrap();

        assert_eq!(detail.parents.len(), 1);
        assert_eq!(detail.files.len(), 50);
        assert_eq!(detail.insertions, 5763);
        assert_eq!(detail.deletions, 9);

        let (insertions, deletions) = detail.files.iter().fold((0, 0), |(ins, del), file| {
            (ins + file.insertions, del + file.deletions)
        });
        assert_eq!(insertions, detail.insertions);
        assert_eq!(deletions, detail.deletions);
    }

    #[test]
    fn diff_de_arquivo_de_texto_traz_hunks_com_a_mesma_soma_do_diffstat() {
        let repo = Git2Repo::open(&project_root()).unwrap();

        // `git diff-tree --numstat 9db77b8 -- web/src/app/Shell.tsx`: 146 inserções, 79
        // remoções — o arquivo com mais deleção deste próprio histórico, bom teste para as
        // duas direções ao mesmo tempo.
        let diff = repo
            .commit_diff(
                "9db77b8e291c73877e0052d085d5c2236967b062",
                "web/src/app/Shell.tsx",
            )
            .unwrap();

        let FileDiff::Text { hunks } = diff else {
            panic!("Shell.tsx é texto, não deveria virar Binary nem NotUtf8");
        };

        assert!(!hunks.is_empty());
        assert!(hunks
            .iter()
            .all(|hunk| hunk.header.starts_with("@@ ") && !hunk.lines.is_empty()));

        let (insertions, deletions) =
            hunks
                .iter()
                .flat_map(|hunk| &hunk.lines)
                .fold((0, 0), |(ins, del), line| match line.kind {
                    DiffLineKind::Addition => (ins + 1, del),
                    DiffLineKind::Deletion => (ins, del + 1),
                    DiffLineKind::Context => (ins, del),
                });
        assert_eq!(insertions, 146);
        assert_eq!(deletions, 79);

        // Toda linha de contexto ou remoção tem número no arquivo antigo; toda linha de
        // contexto ou adição tem número no novo. É o par que falta em cada uma que diz de
        // qual lado ela é.
        for hunk in &hunks {
            for line in &hunk.lines {
                match line.kind {
                    DiffLineKind::Addition => {
                        assert!(line.old_lineno.is_none() && line.new_lineno.is_some())
                    }
                    DiffLineKind::Deletion => {
                        assert!(line.old_lineno.is_some() && line.new_lineno.is_none())
                    }
                    DiffLineKind::Context => {
                        assert!(line.old_lineno.is_some() && line.new_lineno.is_some())
                    }
                }
            }
        }
    }

    #[test]
    fn diff_de_arquivo_binario_nao_traz_hunks() {
        let repo = Git2Repo::open(&project_root()).unwrap();

        // A própria fonte que o Passo 15 trouxe: o git a reconhece como binário de cara.
        let diff = repo
            .commit_diff(
                "0b61c6f62675d70dec6e486da71f89b9cd6a6561",
                "assets/fonts/Inter-latin.woff2",
            )
            .unwrap();

        assert!(matches!(diff, FileDiff::Binary));
    }

    #[test]
    fn arquivo_que_o_commit_nao_tocou_e_recusado() {
        let repo = Git2Repo::open(&project_root()).unwrap();

        assert!(matches!(
            repo.commit_diff(
                "2cea64adc1a40be6a9388b50800daea31c850d84",
                "arquivo/que/nao/existe.txt",
            ),
            Err(GitError::FileNotInCommit)
        ));
    }

    #[test]
    fn walk_for_index_visita_todo_commit_alcancavel_do_head() {
        let repo = Git2Repo::open(&project_root()).unwrap();

        let mut entries = Vec::new();
        let tip = repo
            .walk_for_index(&mut |entry| {
                entries.push(entry);
                true
            })
            .unwrap();

        if let Head::Branch { commit, .. } | Head::Detached { commit } = repo.info().unwrap().head {
            assert_eq!(tip.as_deref(), Some(commit.as_str()));
        }

        // Contagem de referência tirada do git2 direto, sem passar pela função sob teste — não
        // hardcoda o tamanho do histórico deste repositório, que cresce a cada commit real.
        let raw = Repository::open(project_root()).unwrap();
        let mut raw_walk = raw.revwalk().unwrap();
        raw_walk.push_head().unwrap();
        let expected = raw_walk.count();

        assert_eq!(entries.len(), expected);
        assert!(entries
            .iter()
            .all(|entry| !entry.oid.is_empty() && !entry.author.is_empty()));
    }

    #[test]
    fn walk_for_index_para_cedo_quando_on_commit_devolve_false() {
        let dir = temp("walk-index");
        build_merge_repo(&dir);
        let repo = Git2Repo::open(&dir).unwrap();

        let mut count = 0;
        repo.walk_for_index(&mut |_entry| {
            count += 1;
            count < 3
        })
        .unwrap();

        assert_eq!(count, 3, "parou assim que on_commit devolveu false");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn list_paths_na_raiz_marca_pasta_com_barra_e_arquivo_sem() {
        let repo = Git2Repo::open(&project_root()).unwrap();

        let paths = repo.list_paths("", 100).unwrap();

        assert!(paths.contains(&"crates/".to_owned()), "{paths:?}");
        assert!(paths.contains(&"web/".to_owned()), "{paths:?}");
        assert!(paths.contains(&"Cargo.toml".to_owned()), "{paths:?}");
        assert!(paths.contains(&"CLAUDE.md".to_owned()), "{paths:?}");
        assert!(
            !paths.contains(&"Cargo.toml/".to_owned()),
            "arquivo não pode levar barra"
        );
    }

    #[test]
    fn list_paths_completa_dentro_de_uma_subpasta_pelo_prefixo() {
        let repo = Git2Repo::open(&project_root()).unwrap();

        // As quatro crates do workspace começam todas com "porc-".
        let all = repo.list_paths("crates/porc-", 100).unwrap();
        assert_eq!(all.len(), 4, "{all:?}");
        assert!(all.contains(&"crates/porc-git/".to_owned()), "{all:?}");

        // Prefixo específico o bastante para achar só uma.
        let narrow = repo.list_paths("crates/porc-g", 100).unwrap();
        assert_eq!(narrow, ["crates/porc-git/"]);
    }

    #[test]
    fn list_paths_respeita_o_limite() {
        let repo = Git2Repo::open(&project_root()).unwrap();
        assert_eq!(repo.list_paths("", 2).unwrap().len(), 2);
    }

    #[test]
    fn list_paths_em_pasta_inexistente_ou_dentro_de_arquivo_nao_falha() {
        let repo = Git2Repo::open(&project_root()).unwrap();

        assert!(repo.list_paths("nao/existe/", 10).unwrap().is_empty());
        // "Cargo.toml" é arquivo, não pasta: não há como descer "dentro" dele.
        assert!(repo.list_paths("Cargo.toml/x", 10).unwrap().is_empty());
    }

    #[test]
    fn repositorio_normal_esta_limpo() {
        let repo = Git2Repo::open(&project_root()).unwrap();
        assert_eq!(repo.state().unwrap(), model::RepoState::Clean);
    }

    #[test]
    fn merge_head_presente_vira_estado_merge() {
        let dir = temp("state-merge");
        let git2_repo = Repository::init(&dir).unwrap();

        // `git2::Repository::state()` decide só pela presença destes marcadores em disco — o
        // mesmo arquivo que um `git merge` com conflito deixaria para trás. Escrever à mão
        // evita ter que produzir um conflito de verdade só para o teste.
        std::fs::write(
            git2_repo.path().join("MERGE_HEAD"),
            "0000000000000000000000000000000000000001\n",
        )
        .unwrap();
        drop(git2_repo);

        let repo = Git2Repo::open(&dir).unwrap();
        assert_eq!(repo.state().unwrap(), model::RepoState::Merge);

        std::fs::remove_dir_all(&dir).ok();
    }
}
