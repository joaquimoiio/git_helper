//! `git add` (stage) e `git reset` (unstage), sempre por lista explícita de caminhos.
//!
//! Nunca `-A`, nunca pathspec vazio: cada chamada nasce de uma seleção que a UI já fez —
//! uma linha, várias marcadas, ou "selecionar tudo" resolvido no cliente para a lista
//! completa (Passo 48). Local e rápido como `status`/`init`: roda até o fim sob o
//! [`crate::exec::LOCAL_TIMEOUT`], não é streaming.

use std::path::Path;

use crate::exec::{self, ExecError, Output, LOCAL_TIMEOUT};

/// `git add -- <paths…>`. Cobre modificado, novo (untracked) e deletado no mesmo comando —
/// `add` stagea a deleção também, não só conteúdo novo.
pub async fn add(repo: &Path, paths: &[String]) -> Result<Output, ExecError> {
    run(repo, "add", paths).await
}

/// `git reset -- <paths…>`. Funciona mesmo com `HEAD` unborn (repositório sem commit
/// nenhum): sem árvore para comparar, o efeito é só tirar a entrada do índice.
pub async fn reset(repo: &Path, paths: &[String]) -> Result<Output, ExecError> {
    run(repo, "reset", paths).await
}

async fn run(repo: &Path, subcommand: &str, paths: &[String]) -> Result<Output, ExecError> {
    let mut git = exec::command(Some(repo));
    // `--` antes dos caminhos: sem ela, um arquivo chamado `-f` viraria flag.
    git.arg(subcommand).arg("--").args(paths);

    exec::run(git, LOCAL_TIMEOUT).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::status_v2;

    async fn status_of(repo: &Path) -> crate::model::WorktreeStatus {
        let output = crate::exec::status::run(repo).await.unwrap();
        status_v2::parse(&output.stdout)
    }

    async fn temp_repo(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("porc-stage-exec-{name}"));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();

        let mut init = exec::command(None);
        init.arg("init").arg(&dir);
        exec::run(init, LOCAL_TIMEOUT).await.unwrap();

        // Autor de commit não importa aqui (não commitamos), mas `status` não precisa dele.
        dir
    }

    #[tokio::test]
    async fn add_stagea_um_arquivo_novo() {
        let dir = temp_repo("add-novo").await;
        std::fs::write(dir.join("arquivo.txt"), "conteudo").unwrap();

        add(&dir, &["arquivo.txt".to_owned()]).await.unwrap();

        let status = status_of(&dir).await;
        assert!(status.untracked.is_empty(), "{:?}", status.untracked);
        assert_eq!(status.staged.len(), 1);
        assert_eq!(status.staged[0].path, "arquivo.txt");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn reset_desfaz_o_add() {
        let dir = temp_repo("reset-desfaz").await;
        std::fs::write(dir.join("arquivo.txt"), "conteudo").unwrap();
        add(&dir, &["arquivo.txt".to_owned()]).await.unwrap();

        reset(&dir, &["arquivo.txt".to_owned()]).await.unwrap();

        let status = status_of(&dir).await;
        assert!(status.staged.is_empty(), "{:?}", status.staged);
        assert_eq!(status.untracked.len(), 1);
        assert_eq!(status.untracked[0].path, "arquivo.txt");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn add_aceita_varios_caminhos_de_uma_vez() {
        let dir = temp_repo("add-lote").await;
        std::fs::write(dir.join("um.txt"), "a").unwrap();
        std::fs::write(dir.join("dois.txt"), "b").unwrap();

        add(&dir, &["um.txt".to_owned(), "dois.txt".to_owned()])
            .await
            .unwrap();

        let status = status_of(&dir).await;
        assert_eq!(status.staged.len(), 2, "{:?}", status.staged);

        std::fs::remove_dir_all(&dir).ok();
    }
}
