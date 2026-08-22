//! Parser do `git status --porcelain=v2 -z`.
//!
//! Formato documentado e estável (ao contrário do `git status` sem `--porcelain`): um
//! cabeçalho `# branch.*`, depois uma linha por entrada — tipo `1` (mudança comum), `2`
//! (rename/copy), `u` (conflito) ou `?` (não rastreado). Com `-z`, **tudo** termina em NUL em
//! vez de `\n`, inclusive o cabeçalho, e o caminho nunca vem entre aspas — é por isso que basta
//! cortar em NUL para separar os registros.
//!
//! A única armadilha é a entrada de rename/copy (tipo `2`): ela carrega **dois** caminhos —
//! `path` e `origPath` — separados por um NUL a mais, então "encontrei um NUL" nem sempre quer
//! dizer "fim de uma entrada"; para o tipo `2`, o registro seguinte inteiro é o caminho antigo,
//! não uma entrada nova.
//!
//! Não é streaming como `parse::records`: `status` não usa `exec::stream` (é rápido e local, o
//! processo já terminou quando isto roda), então opera no buffer inteiro de uma vez.

use crate::model::{BranchStatus, RepoState, StatusEntry, StatusKind, WorktreeStatus};

/// Divide o stdout em registros por NUL. O NUL final produz uma fatia vazia à direita
/// (`"a\0b\0"` → `["a", "b", ""]`), descartada.
fn split_records(stdout: &[u8]) -> Vec<String> {
    stdout
        .split(|&byte| byte == 0)
        .filter(|slice| !slice.is_empty())
        // Um nome de arquivo fora de UTF-8 não pode derrubar o status inteiro — mesma escolha
        // do resto do `porc-git` (`parse::records`, os campos de commit no `read.rs`).
        .map(|slice| String::from_utf8_lossy(slice).into_owned())
        .collect()
}

/// `kind` de um caractere de status (`X` ou `Y`). `None` para `.` — sem mudança naquele lado.
fn kind_of(code: char) -> Option<StatusKind> {
    match code {
        'M' => Some(StatusKind::Modified),
        'A' => Some(StatusKind::Added),
        'D' => Some(StatusKind::Deleted),
        'R' => Some(StatusKind::Renamed),
        'C' => Some(StatusKind::Copied),
        'T' => Some(StatusKind::Typechange),
        'U' => Some(StatusKind::Unmerged),
        _ => None,
    }
}

/// `# branch.oid <oid>|(initial)`, `# branch.head <nome>|(detached)`, `# branch.upstream
/// <nome>`, `# branch.ab +<ahead> -<behind>` — os quatro cabeçalhos que `--branch` acrescenta,
/// cada um numa linha própria.
fn apply_header(branch: &mut BranchStatus, header: &str) {
    let mut parts = header.splitn(2, ' ');
    let Some(key) = parts.next() else { return };
    let value = parts.next().unwrap_or_default();

    match key {
        "branch.oid" => branch.oid = (value != "(initial)").then(|| value.to_owned()),
        "branch.head" => {
            if value == "(detached)" {
                branch.detached = true;
            } else {
                branch.head = Some(value.to_owned());
            }
        }
        "branch.upstream" => branch.upstream = Some(value.to_owned()),
        "branch.ab" => {
            let mut numbers = value.split_whitespace();
            branch.ahead = numbers
                .next()
                .and_then(|n| n.strip_prefix('+'))
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
            branch.behind = numbers
                .next()
                .and_then(|n| n.strip_prefix('-'))
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
        }
        _ => {}
    }
}

/// `1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>` — oito campos, o último é o caminho (pode
/// conter espaço; só ele, por isso `splitn`).
fn ordinary(rest: &str) -> Option<(&str, &str)> {
    let mut fields = rest.splitn(8, ' ');
    let xy = fields.next()?;
    // sub, mH, mI, mW, hH, hI: seis campos entre `xy` e `path`.
    for _ in 0..6 {
        fields.next()?;
    }
    let path = fields.next()?;
    Some((xy, path))
}

/// `2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path>` — nove campos; `origPath` não
/// está aqui, é o próximo registro NUL inteiro.
fn renamed(rest: &str) -> Option<(&str, &str)> {
    let mut fields = rest.splitn(9, ' ');
    let xy = fields.next()?;
    // sub, mH, mI, mW, hH, hI, Xscore: sete campos entre `xy` e `path`.
    for _ in 0..7 {
        fields.next()?;
    }
    let path = fields.next()?;
    Some((xy, path))
}

/// `u <XY> <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>` — dez campos.
fn unmerged(rest: &str) -> Option<&str> {
    let mut fields = rest.splitn(10, ' ');
    for _ in 0..9 {
        fields.next()?;
    }
    fields.next()
}

/// Um par `XY` vira até duas entradas: `X` (índice) alimenta `staged`, `Y` (worktree) alimenta
/// `unstaged`. Um arquivo com as duas letras diferentes de `.` (parcialmente stageado) aparece
/// nos dois grupos — é exatamente o que `git status` de terminal também mostra.
fn push_xy(
    staged: &mut Vec<StatusEntry>,
    unstaged: &mut Vec<StatusEntry>,
    xy: &str,
    path: &str,
    old_path: Option<String>,
) {
    let mut chars = xy.chars();
    let x = chars.next().unwrap_or('.');
    let y = chars.next().unwrap_or('.');

    if let Some(kind) = kind_of(x) {
        staged.push(StatusEntry {
            path: path.to_owned(),
            old_path: old_path.clone(),
            kind,
        });
    }
    if let Some(kind) = kind_of(y) {
        unstaged.push(StatusEntry {
            path: path.to_owned(),
            old_path,
            kind,
        });
    }
}

/// Interpreta o stdout cru de [`crate::exec::status::run`].
///
/// `state` sai sempre `Clean`: quem sabe de merge/rebase em andamento é o
/// `git2::Repository::state()` (`read.rs`), uma fonte diferente — o chamador (a rota) sobrescreve
/// depois de chamar as duas.
///
/// Linha que não bate nenhum formato conhecido é ignorada, não derruba o status inteiro: uma
/// versão futura do git pode acrescentar um tipo novo (`!` de ignorado, por exemplo, que esta
/// rota nunca pede mas apareceria se o argumento mudasse), e perder uma linha é melhor do que
/// a UI inteira ficar sem status por causa dela.
pub fn parse(stdout: &[u8]) -> WorktreeStatus {
    let mut branch = BranchStatus {
        oid: None,
        head: None,
        detached: false,
        upstream: None,
        ahead: 0,
        behind: 0,
    };
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();

    let mut records = split_records(stdout).into_iter();

    while let Some(record) = records.next() {
        if let Some(header) = record.strip_prefix("# ") {
            apply_header(&mut branch, header);
            continue;
        }

        let mut fields = record.splitn(2, ' ');
        let kind_char = fields.next().unwrap_or_default();
        let rest = fields.next().unwrap_or_default();

        match kind_char {
            "1" => {
                if let Some((xy, path)) = ordinary(rest) {
                    push_xy(&mut staged, &mut unstaged, xy, path, None);
                }
            }
            "2" => {
                if let Some((xy, path)) = renamed(rest) {
                    // `origPath` é o registro NUL seguinte inteiro — sem parsing nenhum, é só
                    // o caminho, cru.
                    let old_path = records.next();
                    push_xy(&mut staged, &mut unstaged, xy, path, old_path);
                }
            }
            "u" => {
                if let Some(path) = unmerged(rest) {
                    unstaged.push(StatusEntry {
                        path: path.to_owned(),
                        old_path: None,
                        kind: StatusKind::Unmerged,
                    });
                }
            }
            "?" => untracked.push(StatusEntry {
                path: rest.to_owned(),
                old_path: None,
                kind: StatusKind::Untracked,
            }),
            // "!" (ignorado) só apareceria com `--ignored`, que esta rota não pede.
            _ => {}
        }
    }

    WorktreeStatus {
        branch,
        state: RepoState::Clean,
        staged,
        unstaged,
        untracked,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(records: &[&str]) -> Vec<u8> {
        records
            .iter()
            .flat_map(|record| record.bytes().chain(std::iter::once(0)))
            .collect()
    }

    #[test]
    fn cabecalho_completo_e_lido() {
        let stdout = build(&[
            "# branch.oid 9db77b8e291c73877e0052d085d5c2236967b062",
            "# branch.head main",
            "# branch.upstream origin/main",
            "# branch.ab +2 -1",
        ]);

        let status = parse(&stdout);

        assert_eq!(
            status.branch.oid.as_deref(),
            Some("9db77b8e291c73877e0052d085d5c2236967b062")
        );
        assert_eq!(status.branch.head.as_deref(), Some("main"));
        assert!(!status.branch.detached);
        assert_eq!(status.branch.upstream.as_deref(), Some("origin/main"));
        assert_eq!(status.branch.ahead, 2);
        assert_eq!(status.branch.behind, 1);
    }

    #[test]
    fn repositorio_sem_commit_nao_tem_oid() {
        let status = parse(&build(&["# branch.oid (initial)", "# branch.head main"]));
        assert_eq!(status.branch.oid, None);
    }

    #[test]
    fn head_destacado_nao_tem_nome_de_branch() {
        let status = parse(&build(&["# branch.oid aaaa", "# branch.head (detached)"]));
        assert!(status.branch.detached);
        assert_eq!(status.branch.head, None);
    }

    #[test]
    fn entrada_ordinaria_separa_staged_e_unstaged() {
        let stdout = build(&[
            "# branch.oid aaaa",
            "# branch.head main",
            "1 M. N... 100644 100644 100644 0000000000000000000000000000000000000000 0000000000000000000000000000000000000000 so-staged.txt",
            "1 .M N... 100644 100644 100644 0000000000000000000000000000000000000000 0000000000000000000000000000000000000000 so-unstaged.txt",
            "1 MM N... 100644 100644 100644 0000000000000000000000000000000000000000 0000000000000000000000000000000000000000 os-dois.txt",
        ]);

        let status = parse(&stdout);

        assert_eq!(status.staged.len(), 2, "{:?}", status.staged);
        assert_eq!(status.unstaged.len(), 2, "{:?}", status.unstaged);
        assert!(status
            .staged
            .iter()
            .any(|e| e.path == "so-staged.txt" && e.kind == StatusKind::Modified));
        assert!(status
            .unstaged
            .iter()
            .any(|e| e.path == "so-unstaged.txt" && e.kind == StatusKind::Modified));
        assert!(status.staged.iter().any(|e| e.path == "os-dois.txt"));
        assert!(status.unstaged.iter().any(|e| e.path == "os-dois.txt"));
    }

    #[test]
    fn caminho_com_espaco_nao_quebra_o_parser() {
        let stdout = build(&[
            "# branch.oid aaaa",
            "# branch.head main",
            "1 .M N... 100644 100644 100644 0000000000000000000000000000000000000000 0000000000000000000000000000000000000000 pasta com espaco/arquivo.txt",
        ]);

        let status = parse(&stdout);
        assert_eq!(status.unstaged[0].path, "pasta com espaco/arquivo.txt");
    }

    #[test]
    fn arquivo_novo_vira_untracked() {
        let stdout = build(&["# branch.oid aaaa", "# branch.head main", "? novo.txt"]);
        let status = parse(&stdout);

        assert_eq!(status.untracked.len(), 1);
        assert_eq!(status.untracked[0].kind, StatusKind::Untracked);
        assert_eq!(status.untracked[0].path, "novo.txt");
    }

    #[test]
    fn rename_traz_o_caminho_antigo_do_registro_seguinte() {
        let stdout = build(&[
            "# branch.oid aaaa",
            "# branch.head main",
            "2 R. N... 100644 100644 100644 0000000000000000000000000000000000000000 0000000000000000000000000000000000000000 R100 novo.txt",
            "velho.txt",
            // Uma entrada normal logo depois: confirma que o parser voltou a ler registros
            // como entradas novas, e não continuou "grudado" no rename.
            "? sobrou.txt",
        ]);

        let status = parse(&stdout);

        assert_eq!(status.staged.len(), 1);
        assert_eq!(status.staged[0].path, "novo.txt");
        assert_eq!(status.staged[0].old_path.as_deref(), Some("velho.txt"));
        assert_eq!(status.staged[0].kind, StatusKind::Renamed);
        assert_eq!(status.untracked.len(), 1);
        assert_eq!(status.untracked[0].path, "sobrou.txt");
    }

    #[test]
    fn conflito_vira_entrada_unmerged() {
        let stdout = build(&[
            "# branch.oid aaaa",
            "# branch.head main",
            "u UU N... 100644 100644 100644 100644 0000000000000000000000000000000000000000 1111111111111111111111111111111111111111 2222222222222222222222222222222222222222 conflito.txt",
        ]);

        let status = parse(&stdout);

        assert_eq!(status.unstaged.len(), 1);
        assert_eq!(status.unstaged[0].kind, StatusKind::Unmerged);
        assert_eq!(status.unstaged[0].path, "conflito.txt");
        assert!(status.staged.is_empty());
    }

    #[test]
    fn worktree_limpa_nao_traz_entrada_nenhuma() {
        let stdout = build(&[
            "# branch.oid aaaa",
            "# branch.head main",
            "# branch.upstream origin/main",
            "# branch.ab +0 -0",
        ]);

        let status = parse(&stdout);

        assert!(status.staged.is_empty());
        assert!(status.unstaged.is_empty());
        assert!(status.untracked.is_empty());
    }

    #[test]
    fn linha_de_formato_desconhecido_e_ignorada_sem_derrubar_o_resto() {
        let stdout = build(&[
            "# branch.oid aaaa",
            "# branch.head main",
            "! ignorado.txt",
            "? real.txt",
        ]);

        let status = parse(&stdout);

        assert_eq!(status.untracked.len(), 1);
        assert_eq!(status.untracked[0].path, "real.txt");
    }
}
