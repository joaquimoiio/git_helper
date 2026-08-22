//! `git status --porcelain=v2 -z --branch --untracked-files=all`.
//!
//! Shell-out, não `git2`: libgit2 não usa fsmonitor nem untracked-cache do git do usuário, e
//! numa worktree grande a diferença é de segundos para dezenas de ms — "segundos por refresh"
//! mataria a premissa do dia inteiro sem terminal (`CLAUDE.md`/`BLOCO-E.md`). `--porcelain=v2`
//! é o formato estável e documentado; `-z` troca todo `\n` por NUL (inclusive no cabeçalho) e
//! caminho nunca vem entre aspas nem escapado — quem interpreta é [`crate::parse::status_v2`].
//!
//! É um comando local e rápido, não streaming: roda até o fim como `init`, sob o
//! [`crate::exec::LOCAL_TIMEOUT`] de trinta segundos.

use std::path::Path;

use crate::exec::{self, ExecError, Output, LOCAL_TIMEOUT};

/// Roda o comando e devolve o stdout cru. Quem interpreta é `parse::status_v2::parse`.
pub async fn run(repo: &Path) -> Result<Output, ExecError> {
    let mut git = exec::command(Some(repo));
    git.args([
        "status",
        "--porcelain=v2",
        "-z",
        "--branch",
        "--untracked-files=all",
    ]);

    exec::run(git, LOCAL_TIMEOUT).await
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
    async fn roda_contra_este_repositorio_e_traz_o_cabecalho() {
        let output = run(&project_root()).await.unwrap();
        let text = String::from_utf8_lossy(&output.stdout);

        assert!(text.contains("# branch.oid"), "saiu: {text}");
        assert!(text.contains("# branch.head"), "saiu: {text}");
    }

    #[tokio::test]
    async fn worktree_limpa_de_verdade_nao_falha() {
        let dir = std::env::temp_dir().join("porc-status-exec-limpa");
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();

        let mut init = exec::command(None);
        init.arg("init").arg(&dir);
        exec::run(init, LOCAL_TIMEOUT).await.unwrap();

        let output = run(&dir).await.unwrap();
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains("# branch.oid (initial)"), "saiu: {text}");

        std::fs::remove_dir_all(&dir).ok();
    }
}
