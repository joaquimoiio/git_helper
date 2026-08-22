//! `git apply --cached`, alimentado por um patch parcial pelo stdin.
//!
//! É o caminho que o `BLOCO-E.md` fixa para stage e unstage por hunk (e por linha, no passo
//! seguinte): **o git é quem aplica**. Casar contexto, respeitar `.gitattributes`, lidar com
//! CRLF e com `\ No newline at end of file` é trabalho dele; reimplementar isso aqui seria
//! fonte garantida de corrupção silenciosa num lugar onde o erro só aparece depois do commit.
//!
//! `--cached` mexe **só no índice** — o arquivo do usuário no disco não é tocado em nenhum dos
//! dois sentidos. Stagear um hunk não muda o que está no editor dele; desfazer também não.

use std::path::Path;

use crate::exec::{self, ExecError, Output, LOCAL_TIMEOUT};

/// Aplica o patch ao índice. `reverse` desfaz em vez de fazer — é o unstage por hunk.
///
/// O patch vem do [`crate::patch::Patch::select`], montado a partir do diff do lado certo:
/// stagear é aplicar o recorte do diff **unstaged**, desfazer é aplicar o recorte do diff
/// **staged** ao contrário.
pub async fn cached(repo: &Path, patch: &str, reverse: bool) -> Result<Output, ExecError> {
    let mut git = exec::command(Some(repo));
    git.arg("apply").arg("--cached");

    if reverse {
        git.arg("--reverse");
    }

    // O patch saiu do arquivo do próprio usuário: espaço em branco "errado" nele é o espaço em
    // branco que ele escreveu, e recusar por causa disso (o que `apply.whitespace = error` na
    // config dele faria) transformaria a config de um `git apply` de terminal numa parede aqui.
    git.arg("--whitespace=nowarn");
    // `-` explícito: sem ele, `git apply` sem arquivo também lê stdin, mas dizer é melhor que
    // depender do padrão.
    git.arg("-");

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

    /// Repositório com um arquivo de trinta linhas commitado e três mudanças espalhadas — longe
    /// o bastante umas das outras para o git as separar em três hunks de verdade.
    async fn repo_com_tres_hunks(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("porc-apply-{name}"));
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

        let mut mudado = original.clone();
        mudado[1] = "MUDOU-A".to_owned();
        mudado[14] = "MUDOU-B".to_owned();
        mudado[28] = "MUDOU-C".to_owned();
        std::fs::write(dir.join("a.txt"), mudado.join("\n") + "\n").unwrap();

        dir
    }

    async fn staged_do_arquivo(dir: &Path) -> Vec<String> {
        let output = status::run(dir).await.unwrap();
        status_v2::parse(&output.stdout)
            .staged
            .into_iter()
            .map(|entry| entry.path)
            .collect()
    }

    /// O que o `git diff --cached` de verdade diz — a referência para "só um hunk entrou".
    async fn diff_staged(dir: &Path) -> String {
        let mut git = exec::command(Some(dir));
        git.args(["diff", "--cached"]);
        let output = exec::run(git, LOCAL_TIMEOUT).await.unwrap();
        String::from_utf8(output.stdout).unwrap()
    }

    #[tokio::test]
    async fn um_hunk_de_tres_entra_no_indice_e_os_outros_ficam_de_fora() {
        let dir = repo_com_tres_hunks("um-de-tres").await;

        let repo = Git2Repo::open(&dir).unwrap();
        let raw = repo.worktree_patch(DiffSide::Unstaged, "a.txt").unwrap();
        let parsed = patch::parse(&raw).unwrap();
        assert_eq!(parsed.hunks.len(), 3, "o repositório de teste tem 3 hunks");

        cached(&dir, &parsed.select_hunks(&[1]).unwrap(), false)
            .await
            .unwrap();

        assert_eq!(staged_do_arquivo(&dir).await, ["a.txt"]);

        let staged = diff_staged(&dir).await;
        assert!(staged.contains("+MUDOU-B"), "{staged}");
        assert!(!staged.contains("+MUDOU-A"), "{staged}");
        assert!(!staged.contains("+MUDOU-C"), "{staged}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn o_ultimo_hunk_sozinho_tambem_aplica() {
        // O caso que quebra se a linha inicial do lado novo não for recalculada: sozinho, o
        // terceiro hunk não tem os deslocamentos dos dois anteriores.
        let dir = repo_com_tres_hunks("ultimo-sozinho").await;

        let repo = Git2Repo::open(&dir).unwrap();
        let raw = repo.worktree_patch(DiffSide::Unstaged, "a.txt").unwrap();
        let parsed = patch::parse(&raw).unwrap();

        cached(&dir, &parsed.select_hunks(&[2]).unwrap(), false)
            .await
            .unwrap();

        let staged = diff_staged(&dir).await;
        assert!(staged.contains("+MUDOU-C"), "{staged}");
        assert!(!staged.contains("+MUDOU-A"), "{staged}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn reverse_tira_do_indice_o_que_foi_stageado() {
        let dir = repo_com_tres_hunks("reverse").await;

        // Stagea o arquivo inteiro pelo caminho normal…
        let mut add = exec::command(Some(&dir));
        add.args(["add", "--", "a.txt"]);
        exec::run(add, LOCAL_TIMEOUT).await.unwrap();

        // …e depois tira **um** hunk de volta, aplicando o diff staged ao contrário.
        let repo = Git2Repo::open(&dir).unwrap();
        let raw = repo.worktree_patch(DiffSide::Staged, "a.txt").unwrap();
        let parsed = patch::parse(&raw).unwrap();
        assert_eq!(parsed.hunks.len(), 3);

        cached(&dir, &parsed.select_hunks(&[0]).unwrap(), true)
            .await
            .unwrap();

        let staged = diff_staged(&dir).await;
        assert!(!staged.contains("+MUDOU-A"), "saiu do índice: {staged}");
        assert!(staged.contains("+MUDOU-B"), "continua no índice: {staged}");
        assert!(staged.contains("+MUDOU-C"), "continua no índice: {staged}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn o_disco_do_usuario_nao_e_tocado() {
        let dir = repo_com_tres_hunks("disco-intacto").await;
        let antes = std::fs::read_to_string(dir.join("a.txt")).unwrap();

        let repo = Git2Repo::open(&dir).unwrap();
        let raw = repo.worktree_patch(DiffSide::Unstaged, "a.txt").unwrap();
        let parsed = patch::parse(&raw).unwrap();
        cached(&dir, &parsed.select_hunks(&[0]).unwrap(), false)
            .await
            .unwrap();

        assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), antes);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn duas_linhas_de_um_hunk_entram_e_o_resto_fica_de_fora() {
        // O aceite do Passo 51: escolher linhas dentro de um hunk e conferir com o
        // `git diff --cached` de verdade.
        let dir = repo_com_tres_hunks("linhas").await;

        // Duas mudanças **vizinhas**, que o git junta num hunk só.
        let mut conteudo: Vec<String> = std::fs::read_to_string(dir.join("a.txt"))
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect();
        conteudo[5] = "VIZINHA-1".to_owned();
        conteudo[6] = "VIZINHA-2".to_owned();
        std::fs::write(dir.join("a.txt"), conteudo.join("\n") + "\n").unwrap();

        let repo = Git2Repo::open(&dir).unwrap();
        let raw = repo.worktree_patch(DiffSide::Unstaged, "a.txt").unwrap();
        let parsed = patch::parse(&raw).unwrap();

        // O hunk que contém as duas vizinhas é o primeiro (as linhas 6 e 7 do arquivo).
        let alvo = parsed
            .hunks
            .iter()
            .position(|hunk| hunk.lines.iter().any(|line| line.contains("VIZINHA-1")))
            .unwrap();

        // Dentro dele, as linhas de mudança são `-linha 6`, `-linha 7`, `+VIZINHA-1`,
        // `+VIZINHA-2` (o git agrupa as remoções antes das adições), mais o par da MUDOU-A.
        // Escolhemos exatamente o par que troca a linha 6.
        let mudancas: Vec<String> = parsed.hunks[alvo]
            .lines
            .iter()
            .filter(|line| line.starts_with('+') || line.starts_with('-'))
            .cloned()
            .collect();
        let remove_6 = mudancas.iter().position(|l| l == "-linha 6").unwrap();
        let poe_vizinha_1 = mudancas.iter().position(|l| l == "+VIZINHA-1").unwrap();

        let recorte = parsed
            .select(&[patch::HunkSelection {
                hunk: alvo,
                lines: Some(vec![remove_6, poe_vizinha_1]),
            }])
            .unwrap();

        cached(&dir, &recorte, false).await.unwrap();

        let staged = diff_staged(&dir).await;
        assert!(staged.contains("+VIZINHA-1"), "{staged}");
        assert!(!staged.contains("+VIZINHA-2"), "{staged}");
        assert!(!staged.contains("+MUDOU-A"), "{staged}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn patch_que_nao_casa_falha_em_vez_de_aplicar_torto() {
        let dir = repo_com_tres_hunks("nao-casa").await;

        let mentira = "diff --git a/a.txt b/a.txt\n--- a/a.txt\n+++ b/a.txt\n\
                       @@ -1,3 +1,3 @@\n conteudo que nunca existiu\n-nem este\n+nem este outro\n conteudo\n";

        let err = cached(&dir, mentira, false).await.unwrap_err();
        assert!(matches!(err, ExecError::Failed(_)), "veio {err:?}");

        std::fs::remove_dir_all(&dir).ok();
    }
}
