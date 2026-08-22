//! Descartar mudanças do working tree — arquivo inteiro ou trecho.
//!
//! **É a única operação do bloco que destrói trabalho sem reflog para socorrer.** Um commit
//! errado se emenda, um stage errado se desfaz, um push errado se reverte; um arquivo
//! sobrescrito pelo conteúdo do índice não existe em lugar nenhum depois. Por isso o
//! `BLOCO-E.md` exige confirmação explícita nomeando o que será perdido — e por isso este
//! módulo nunca recebe "tudo": só listas de caminhos que a interface mostrou, uma a uma.
//!
//! Três comandos, porque são três coisas diferentes:
//!
//! - rastreado e modificado → `git checkout -- <paths>` (volta ao que está no índice);
//! - **não rastreado** → apagar o arquivo, porque o git não tem versão nenhuma dele para
//!   restaurar (`git clean` faria o mesmo, com mais flags perigosas por perto);
//! - trecho → `git apply --reverse` **sem** `--cached`, que desfaz o hunk no arquivo em disco.

use std::path::Path;

use crate::exec::{self, ExecError, Output, LOCAL_TIMEOUT};

/// `git checkout -- <paths…>`: o arquivo volta ao conteúdo que está no índice.
///
/// Note que **não** volta ao do `HEAD`: se algo daquele arquivo estava preparado, o preparado
/// continua ali. É a mesma semântica do terminal, e é a que não surpreende quem stageou de
/// propósito antes de descartar o resto.
pub async fn checkout(repo: &Path, paths: &[String]) -> Result<Output, ExecError> {
    let mut git = exec::command(Some(repo));
    // `--` antes dos caminhos, sempre: sem ela um arquivo chamado `-f` viraria flag — e aqui a
    // flag errada apaga coisa.
    git.arg("checkout").arg("--").args(paths);

    exec::run(git, LOCAL_TIMEOUT).await
}

/// Apaga arquivos não rastreados, um a um, pelos caminhos dados.
///
/// `std::fs::remove_file` e não `git clean`: o `clean` opera por pathspec e tem vizinhos
/// (`-x`, `-d`, `-f`) cuja diferença entre "apaga o que você pediu" e "apaga a pasta inteira" é
/// uma letra. Aqui o que se apaga é exatamente o que a interface nomeou na confirmação.
///
/// Cada caminho é resolvido dentro do repositório e conferido contra ele: um `..` no meio ou um
/// symlink apontando para fora não podem virar um `remove_file` em outro lugar do disco.
pub fn remove_untracked(repo: &Path, paths: &[String]) -> Result<(), ExecError> {
    for path in paths {
        let target = repo.join(path);

        let Ok(canonical) = target.canonicalize() else {
            // Já não existe: o objetivo era exatamente esse.
            continue;
        };
        let Ok(root) = repo.canonicalize() else {
            return Err(ExecError::Spawn(std::io::Error::other(
                "o repositório sumiu do disco",
            )));
        };

        if !canonical.starts_with(&root) {
            tracing::warn!(path, "caminho de descarte sai do repositório — ignorado");
            continue;
        }

        // Só arquivo: descartar "uma pasta não rastreada" seria uma remoção recursiva pedida
        // por um clique, e não é o que a confirmação nomeou.
        if canonical.is_dir() {
            tracing::warn!(path, "descarte de diretório não rastreado não é suportado");
            continue;
        }

        if let Err(err) = std::fs::remove_file(&canonical) {
            return Err(ExecError::Spawn(err));
        }
    }

    Ok(())
}

/// `git apply --reverse` **sem** `--cached`: desfaz o trecho no arquivo em disco.
///
/// O patch vem do mesmo recorte do Passo 50/51, montado a partir do diff unstaged. É a versão
/// destrutiva do que o `apply::cached` faz — ali a origem continua intacta no worktree, aqui é
/// o worktree que muda.
pub async fn revert_worktree(repo: &Path, patch: &str) -> Result<Output, ExecError> {
    let mut git = exec::command(Some(repo));
    git.args(["apply", "--reverse", "--whitespace=nowarn", "-"]);

    exec::run_with_input(git, LOCAL_TIMEOUT, patch.as_bytes().to_vec()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        exec::status,
        parse::status_v2,
        patch,
        read::{DiffSide, Git2Repo, RepoRead},
    };

    async fn repo(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("porc-discard-{name}"));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();

        let mut init = exec::command(None);
        init.arg("init").arg(&dir);
        exec::run(init, LOCAL_TIMEOUT).await.unwrap();

        let original: Vec<String> = (1..=30).map(|n| format!("linha {n}")).collect();
        std::fs::write(dir.join("a.txt"), original.join("\n") + "\n").unwrap();

        let mut add = exec::command(Some(&dir));
        add.args(["add", "--", "a.txt"]);
        exec::run(add, LOCAL_TIMEOUT).await.unwrap();

        let mut commit = exec::command(Some(&dir));
        commit.args([
            "-c",
            "user.email=t@e",
            "-c",
            "user.name=T",
            "commit",
            "-m",
            "inicial",
        ]);
        exec::run(commit, LOCAL_TIMEOUT).await.unwrap();

        dir
    }

    async fn worktree_status(dir: &Path) -> crate::model::WorktreeStatus {
        let output = status::run(dir).await.unwrap();
        status_v2::parse(&output.stdout)
    }

    #[tokio::test]
    async fn checkout_devolve_o_arquivo_ao_que_esta_no_indice() {
        let dir = repo("checkout").await;
        let antes = std::fs::read_to_string(dir.join("a.txt")).unwrap();

        std::fs::write(dir.join("a.txt"), "estragado\n").unwrap();
        checkout(&dir, &["a.txt".to_owned()]).await.unwrap();

        assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), antes);
        assert!(worktree_status(&dir).await.unstaged.is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn checkout_preserva_o_que_ja_estava_preparado() {
        let dir = repo("checkout-staged").await;

        // Uma mudança preparada…
        std::fs::write(dir.join("a.txt"), "preparado\n").unwrap();
        crate::exec::stage::add(&dir, &["a.txt".to_owned()])
            .await
            .unwrap();
        // …e outra por cima, só no disco.
        std::fs::write(dir.join("a.txt"), "preparado\ne mais isto\n").unwrap();

        checkout(&dir, &["a.txt".to_owned()]).await.unwrap();

        // Volta ao índice, não ao HEAD: o que estava preparado continua ali.
        assert_eq!(
            std::fs::read_to_string(dir.join("a.txt")).unwrap(),
            "preparado\n"
        );
        let status = worktree_status(&dir).await;
        assert_eq!(status.staged.len(), 1);
        assert!(status.unstaged.is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn arquivo_nao_rastreado_e_apagado() {
        let dir = repo("untracked").await;
        std::fs::write(dir.join("novo.txt"), "lixo\n").unwrap();

        remove_untracked(&dir, &["novo.txt".to_owned()]).unwrap();

        assert!(!dir.join("novo.txt").exists());
        assert!(worktree_status(&dir).await.untracked.is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn caminho_que_sai_do_repositorio_nao_apaga_nada() {
        let dir = repo("confinamento").await;
        let fora = dir.parent().unwrap().join("porc-discard-nao-me-apague.txt");
        std::fs::write(&fora, "importante\n").unwrap();

        remove_untracked(&dir, &["../porc-discard-nao-me-apague.txt".to_owned()]).unwrap();

        assert!(fora.exists(), "o arquivo de fora tem que continuar lá");

        std::fs::remove_file(&fora).ok();
        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn reverter_um_hunk_desfaz_so_ele_no_disco() {
        let dir = repo("hunk").await;

        let mut conteudo: Vec<String> = std::fs::read_to_string(dir.join("a.txt"))
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect();
        conteudo[1] = "MUDOU-A".to_owned();
        conteudo[28] = "MUDOU-C".to_owned();
        std::fs::write(dir.join("a.txt"), conteudo.join("\n") + "\n").unwrap();

        let git2 = Git2Repo::open(&dir).unwrap();
        let raw = git2.worktree_patch(DiffSide::Unstaged, "a.txt").unwrap();
        let parsed = patch::parse(&raw).unwrap();
        assert_eq!(parsed.hunks.len(), 2);

        revert_worktree(&dir, &parsed.select_hunks(&[0]).unwrap())
            .await
            .unwrap();

        let depois = std::fs::read_to_string(dir.join("a.txt")).unwrap();
        assert!(!depois.contains("MUDOU-A"), "o primeiro hunk voltou");
        assert!(depois.contains("MUDOU-C"), "o segundo continua: {depois}");

        std::fs::remove_dir_all(&dir).ok();
    }
}
