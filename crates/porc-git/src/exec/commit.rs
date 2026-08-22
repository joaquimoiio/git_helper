//! `git commit`, com a mensagem entregue pelo stdin.
//!
//! Shell-out, e não `git2`: um commit precisa disparar os hooks do usuário (`pre-commit`,
//! `prepare-commit-msg`, `commit-msg`, `post-commit`), respeitar `commit.gpgSign`,
//! `user.signingkey` e o `core.hooksPath` dele. Um commit escrito por dentro do libgit2
//! nasceria sem nada disso, e o repositório é dele.
//!
//! `-F -`: a mensagem vai por pipe. Nunca em `argv` (que qualquer `ps` da máquina mostra) e
//! nunca em arquivo temporário (uma janela a mais para outro processo, e mais um arquivo para
//! limpar quando o comando morre no meio).

use std::path::Path;

use crate::exec::{self, ExecError, LOCAL_TIMEOUT};

#[derive(Debug, thiserror::Error)]
pub enum CommitError {
    #[error("a mensagem do commit está vazia")]
    EmptyMessage,
    #[error("não há nada preparado para commitar")]
    NothingToCommit,
    #[error("o git não sabe quem você é — configure `user.name` e `user.email`")]
    IdentityMissing,
    /// O git ou um **hook** recusou. O texto aqui é a saída do hook, que é escrita pelo time do
    /// usuário e costuma ser a única coisa útil que existe sobre a recusa — diferente do stderr
    /// de rede do Passo 34, que é jargão do git e por isso vira frase nossa.
    #[error("o commit foi recusado: {0}")]
    Refused(String),
    /// A assinatura GPG não saiu. Quase sempre é o pinentry: o `gpg-agent` precisa perguntar a
    /// passphrase e não tem onde — não herdamos terminal, e o askpass do Passo 33 é do git e do
    /// ssh, não do gpg.
    #[error("não consegui assinar o commit — o gpg não conseguiu pedir a senha da chave")]
    SigningFailed,
    #[error("não há commit anterior para emendar")]
    NothingToAmend,
    #[error("o commit a corrigir não é um hash válido")]
    InvalidFixupTarget,
    #[error(transparent)]
    Exec(#[from] ExecError),
}

#[derive(Debug, Clone, Default)]
pub struct CommitOptions {
    /// Assunto e corpo já juntos, como o usuário escreveu. A limpeza (espaço no fim, linhas em
    /// branco sobrando, linhas de comentário do template) é do `--cleanup=strip`.
    pub message: String,
    /// Reescreve o commit anterior em vez de criar um novo (`--amend`).
    pub amend: bool,
    /// Acrescenta a linha `Signed-off-by:` com a identidade configurada (`--signoff`).
    pub signoff: bool,
    /// `None` respeita a config do usuário (`commit.gpgSign`); `Some` força ligado ou desligado
    /// só para **este** commit, via flag — nunca escrevendo na config dele.
    pub gpg_sign: Option<bool>,
    /// Oid do commit que este corrige (`--fixup=<oid>`). Quando presente, **a mensagem é do
    /// git**: ele monta `fixup! <assunto do alvo>` sozinho, e é essa mensagem exata que o
    /// `rebase --autosquash` sabe reconhecer depois. Escrever uma mensagem à mão aqui seria a
    /// forma mais fácil de produzir um fixup que o autosquash ignora.
    pub fixup: Option<String>,
}

/// Faz o commit. Devolve o oid novo do `HEAD`.
pub async fn commit(repo: &Path, options: &CommitOptions) -> Result<String, CommitError> {
    // Num fixup a mensagem é do git, então a caixa vazia é o normal — a checagem só vale para
    // um commit em que o texto é do usuário.
    //
    // Checado aqui e não deixado para o git: "aborting due to empty commit message" no stderr
    // seria uma volta inteira ao processo para dizer o que dá para saber antes de sair daqui.
    if options.fixup.is_none() && options.message.trim().is_empty() {
        return Err(CommitError::EmptyMessage);
    }

    let mut git = exec::command(Some(repo));
    git.arg("commit");

    match &options.fixup {
        // `--fixup` e `-F -` são mutuamente exclusivos: os dois dizem qual é a mensagem, e o git
        // recusa os dois juntos. Aqui quem manda é o `--fixup`.
        Some(target) => {
            validate_oid(target)?;
            git.arg(format!("--fixup={target}"));
        }
        // `--cleanup=strip` é o que o git aplica depois de uma sessão de editor — e a caixa de
        // mensagem da interface **é** o editor. Tira espaço no fim, linhas em branco sobrando e
        // as linhas de comentário que um `commit.template` costuma trazer. A consequência
        // conhecida é a mesma do terminal: uma linha que começa com `#` é comentário.
        None => {
            git.args(["--cleanup=strip", "-F", "-"]);
        }
    }

    if options.amend {
        git.arg("--amend");
    }
    if options.signoff {
        git.arg("--signoff");
    }
    // Flag, e nunca `git config`: forçar assinatura para **este** commit não pode virar uma
    // mudança permanente na configuração do usuário.
    match options.gpg_sign {
        Some(true) => git.arg("--gpg-sign"),
        Some(false) => git.arg("--no-gpg-sign"),
        // Sem flag nenhuma: vale o `commit.gpgSign` dele, que é o certo por padrão.
        None => &mut git,
    };

    // Num fixup não há stdin para alimentar: a mensagem é do git. Mandar um pipe vazio ali
    // faria o git ler uma mensagem vazia de um `-F -` que nem está na linha de comando.
    let result = match options.fixup {
        Some(_) => exec::run(git, LOCAL_TIMEOUT).await,
        None => {
            exec::run_with_input(git, LOCAL_TIMEOUT, options.message.clone().into_bytes()).await
        }
    };

    if let Err(ExecError::Failed(failure)) = &result {
        // Os dois canais: "nothing to commit" o git escreve no **stdout**, e o resto (recusa de
        // hook, identidade faltando) no stderr.
        return Err(classify(&failure.stderr, &failure.stdout));
    }
    result?;

    head_oid(repo).await
}

/// O stderr de um `git commit` que falhou vira o erro certo.
///
/// Ordem importa, como no `exec::error`: o específico antes do genérico.
fn classify(stderr: &str, stdout: &str) -> CommitError {
    // Antes de tudo: uma falha de assinatura é o que mais confunde, porque o git a reporta como
    // se o commit tivesse sido recusado por outro motivo qualquer.
    if stderr.contains("gpg failed to sign the data")
        || stderr.contains("failed to write commit object")
        || stderr.contains("Inappropriate ioctl for device")
    {
        return CommitError::SigningFailed;
    }

    if stderr.contains("You have nothing to amend")
        || stderr.contains("does not have any commits yet")
    {
        return CommitError::NothingToAmend;
    }

    // `LC_ALL=C` na `exec::command` garante que estas frases chegam em inglês, sempre.
    if stderr.contains("Please tell me who you are")
        || stderr.contains("unable to auto-detect email address")
        || stderr.contains("empty ident name")
    {
        return CommitError::IdentityMissing;
    }

    let ambos = format!("{stderr}\n{stdout}");
    if ambos.contains("nothing to commit")
        || ambos.contains("no changes added to commit")
        || ambos.contains("nothing added to commit")
    {
        return CommitError::NothingToCommit;
    }

    // O que sobra é, na prática, hook recusando. As primeiras linhas não vazias são a mensagem
    // que o hook escreveu; um teto existe para um hook falador não virar uma parede de texto.
    let details: Vec<&str> = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(3)
        .collect();

    if details.is_empty() {
        return CommitError::Refused("o git não disse por quê".to_owned());
    }

    CommitError::Refused(details.join(" · "))
}

/// O alvo do `--fixup` tem que ser um oid, não um revisão qualquer.
///
/// Confinamento do mesmo tipo que o do `init`: o valor vem do cliente e vira parte de um
/// argumento (`--fixup=<x>`). Aceitar revisão livre daria a ele `HEAD@{…}`, `:/texto` e
/// companhia — sintaxes que o git resolve e que ninguém aqui está preparado para explicar.
/// A UI sempre tem o oid completo do commit que o usuário selecionou no log.
fn validate_oid(oid: &str) -> Result<(), CommitError> {
    let valido = (7..=40).contains(&oid.len()) && oid.chars().all(|c| c.is_ascii_hexdigit());

    if valido {
        Ok(())
    } else {
        Err(CommitError::InvalidFixupTarget)
    }
}

async fn head_oid(repo: &Path) -> Result<String, CommitError> {
    let mut git = exec::command(Some(repo));
    git.args(["rev-parse", "HEAD"]);

    let output = exec::run(git, LOCAL_TIMEOUT).await?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// O conteúdo do `commit.template` do usuário, se ele tiver um.
///
/// Duas chamadas: o git resolve a **chave** (que pode vir do `.git/config`, do global ou do
/// sistema) e nós lemos o arquivo. Não configurado, ilegível ou apontando para o nada devolve
/// `None` — um template ausente nunca pode impedir alguém de commitar.
pub async fn template(repo: &Path) -> Option<String> {
    let mut git = exec::command(Some(repo));
    git.args(["config", "--get", "commit.template"]);

    // Chave ausente é status 1, não erro de execução: `Err` aqui é o caso normal de quem não
    // configurou template nenhum.
    let output = exec::run(git, LOCAL_TIMEOUT).await.ok()?;
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_owned();

    if raw.is_empty() {
        return None;
    }

    // `~` é expandido pelo shell, e aqui não há shell. O git faz essa expansão sozinho no valor
    // de `commit.template`, mas só quando ele mesmo vai ler o arquivo.
    let path = match raw.strip_prefix("~/") {
        Some(rest) => dirs_home()?.join(rest),
        None => std::path::PathBuf::from(&raw),
    };

    match std::fs::read_to_string(&path) {
        Ok(content) => Some(content),
        Err(err) => {
            tracing::warn!(%err, template = %path.display(), "commit.template não pôde ser lido");
            None
        }
    }
}

fn dirs_home() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(std::path::PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::LOCAL_TIMEOUT;

    async fn repo(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("porc-commit-{name}"));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();

        let mut init = exec::command(None);
        init.arg("init").arg(&dir);
        exec::run(init, LOCAL_TIMEOUT).await.unwrap();

        // Identidade no config **local** do repositório de teste: nunca `--global`, que é a
        // máquina do usuário. É o mesmo motivo pelo qual o app usa `-c` efêmero.
        for (key, value) in [("user.email", "teste@example.com"), ("user.name", "Teste")] {
            let mut config = exec::command(Some(&dir));
            config.args(["config", key, value]);
            exec::run(config, LOCAL_TIMEOUT).await.unwrap();
        }

        dir
    }

    async fn stage(dir: &Path, name: &str, content: &str) {
        std::fs::write(dir.join(name), content).unwrap();
        crate::exec::stage::add(dir, &[name.to_owned()])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn commita_o_que_esta_no_indice_e_devolve_o_oid() {
        let dir = repo("normal").await;
        stage(&dir, "a.txt", "conteudo\n").await;

        let oid = commit(
            &dir,
            &CommitOptions {
                message: "assunto\n\ncorpo do commit\n".to_owned(),
                ..CommitOptions::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(oid.len(), 40, "oid completo, veio {oid:?}");

        let mut log = exec::command(Some(&dir));
        log.args(["log", "-1", "--format=%s%n%b"]);
        let saida = exec::run(log, LOCAL_TIMEOUT).await.unwrap();
        let texto = String::from_utf8_lossy(&saida.stdout);

        assert!(texto.starts_with("assunto\n"), "{texto}");
        assert!(texto.contains("corpo do commit"), "{texto}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn mensagem_vazia_nao_chega_a_chamar_o_git() {
        let dir = repo("vazia").await;
        stage(&dir, "a.txt", "conteudo\n").await;

        let err = commit(
            &dir,
            &CommitOptions {
                message: "   \n\n".to_owned(),
                ..CommitOptions::default()
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(err, CommitError::EmptyMessage), "veio {err:?}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn indice_vazio_vira_erro_proprio() {
        let dir = repo("nada").await;

        let err = commit(
            &dir,
            &CommitOptions {
                message: "assunto".to_owned(),
                ..CommitOptions::default()
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(err, CommitError::NothingToCommit), "veio {err:?}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn hook_que_recusa_traz_a_mensagem_do_hook() {
        let dir = repo("hook").await;
        stage(&dir, "a.txt", "conteudo\n").await;

        let hook = dir.join(".git/hooks/pre-commit");
        std::fs::write(
            &hook,
            "#!/bin/sh\necho 'o lint falhou em 3 arquivos' >&2\nexit 1\n",
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let err = commit(
            &dir,
            &CommitOptions {
                message: "assunto".to_owned(),
                ..CommitOptions::default()
            },
        )
        .await
        .unwrap_err();

        match err {
            CommitError::Refused(details) => {
                assert!(details.contains("o lint falhou em 3 arquivos"), "{details}")
            }
            other => panic!("esperava Refused, veio {other:?}"),
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn amend_reescreve_o_commit_anterior_em_vez_de_criar_outro() {
        let dir = repo("amend").await;
        stage(&dir, "a.txt", "conteudo\n").await;

        let primeiro = commit(
            &dir,
            &CommitOptions {
                message: "assunto errado".to_owned(),
                ..CommitOptions::default()
            },
        )
        .await
        .unwrap();

        let emendado = commit(
            &dir,
            &CommitOptions {
                message: "assunto certo".to_owned(),
                amend: true,
                ..CommitOptions::default()
            },
        )
        .await
        .unwrap();

        // Commit reescrito é outro objeto — o oid muda mesmo com a mesma árvore.
        assert_ne!(primeiro, emendado);

        let mut log = exec::command(Some(&dir));
        log.args(["log", "--format=%s"]);
        let saida = exec::run(log, LOCAL_TIMEOUT).await.unwrap();
        let texto = String::from_utf8_lossy(&saida.stdout);

        // E continua sendo **um** commit, não dois.
        assert_eq!(
            texto.lines().collect::<Vec<_>>(),
            ["assunto certo"],
            "{texto}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn amend_sem_commit_anterior_tem_erro_proprio() {
        let dir = repo("amend-vazio").await;
        stage(&dir, "a.txt", "conteudo\n").await;

        let err = commit(
            &dir,
            &CommitOptions {
                message: "assunto".to_owned(),
                amend: true,
                ..CommitOptions::default()
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(err, CommitError::NothingToAmend), "veio {err:?}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn signoff_acrescenta_o_trailer_com_a_identidade_configurada() {
        let dir = repo("signoff").await;
        stage(&dir, "a.txt", "conteudo\n").await;

        commit(
            &dir,
            &CommitOptions {
                message: "assunto".to_owned(),
                signoff: true,
                ..CommitOptions::default()
            },
        )
        .await
        .unwrap();

        let mut log = exec::command(Some(&dir));
        log.args(["log", "-1", "--format=%b"]);
        let saida = exec::run(log, LOCAL_TIMEOUT).await.unwrap();
        let texto = String::from_utf8_lossy(&saida.stdout);

        assert!(
            texto.contains("Signed-off-by: Teste <teste@example.com>"),
            "{texto}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn no_gpg_sign_desliga_a_assinatura_mesmo_com_a_config_ligada() {
        let dir = repo("sem-assinatura").await;

        // Config ligada de propósito, com uma chave que não existe: sem o `--no-gpg-sign` o
        // commit falharia ao tentar assinar.
        for (key, value) in [
            ("commit.gpgSign", "true"),
            ("user.signingkey", "0xNAOEXISTE"),
        ] {
            let mut config = exec::command(Some(&dir));
            config.args(["config", key, value]);
            exec::run(config, LOCAL_TIMEOUT).await.unwrap();
        }

        stage(&dir, "a.txt", "conteudo\n").await;

        commit(
            &dir,
            &CommitOptions {
                message: "assunto".to_owned(),
                gpg_sign: Some(false),
                ..CommitOptions::default()
            },
        )
        .await
        .unwrap();

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn fixup_monta_a_mensagem_que_o_autosquash_reconhece() {
        let dir = repo("fixup").await;
        stage(&dir, "a.txt", "primeira versao\n").await;
        let alvo = commit(
            &dir,
            &CommitOptions {
                message: "arruma o parser".to_owned(),
                ..CommitOptions::default()
            },
        )
        .await
        .unwrap();

        stage(&dir, "a.txt", "segunda versao\n").await;
        commit(
            &dir,
            &CommitOptions {
                fixup: Some(alvo.clone()),
                ..CommitOptions::default()
            },
        )
        .await
        .unwrap();

        let mut log = exec::command(Some(&dir));
        log.args(["log", "-1", "--format=%s"]);
        let saida = exec::run(log, LOCAL_TIMEOUT).await.unwrap();
        let assunto = String::from_utf8_lossy(&saida.stdout).trim().to_owned();

        // A mensagem exata que o `rebase --autosquash` procura: `fixup! ` mais o assunto do alvo.
        assert_eq!(assunto, "fixup! arruma o parser");

        // E o autosquash de verdade a reconhece: um rebase não interativo com `--autosquash`
        // funde o fixup no alvo e o histórico volta a ter um commit só.
        let mut rebase = exec::command(Some(&dir));
        rebase.args([
            "-c",
            "sequence.editor=true",
            "rebase",
            "--autosquash",
            "--root",
        ]);
        exec::run(rebase, LOCAL_TIMEOUT).await.unwrap();

        let mut depois = exec::command(Some(&dir));
        depois.args(["log", "--format=%s"]);
        let saida = exec::run(depois, LOCAL_TIMEOUT).await.unwrap();
        let texto = String::from_utf8_lossy(&saida.stdout);
        assert_eq!(
            texto.lines().collect::<Vec<_>>(),
            ["arruma o parser"],
            "{texto}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn alvo_de_fixup_que_nao_e_hash_e_recusado_antes_do_git() {
        let dir = repo("fixup-invalido").await;

        for alvo in ["HEAD", ":/assunto", "../etc", "zzzz", ""] {
            let err = commit(
                &dir,
                &CommitOptions {
                    fixup: Some(alvo.to_owned()),
                    ..CommitOptions::default()
                },
            )
            .await
            .unwrap_err();

            assert!(
                matches!(err, CommitError::InvalidFixupTarget),
                "alvo {alvo:?} veio {err:?}"
            );
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn falha_de_assinatura_tem_frase_propria_e_nao_vira_recusa_generica() {
        // stderr real de um `git commit -S` sem agente para pedir a passphrase.
        let stderr = "error: gpg failed to sign the data\n\
                      fatal: failed to write commit object\n";

        assert!(
            matches!(classify(stderr, ""), CommitError::SigningFailed),
            "veio {:?}",
            classify(stderr, "")
        );
    }

    #[tokio::test]
    async fn template_ausente_e_none_e_presente_vem_inteiro() {
        let dir = repo("template").await;

        assert!(template(&dir).await.is_none());

        let arquivo = dir.join("modelo.txt");
        std::fs::write(&arquivo, "# escreva o assunto aqui\n\n# corpo\n").unwrap();

        let mut config = exec::command(Some(&dir));
        config.args(["config", "commit.template", &arquivo.to_string_lossy()]);
        exec::run(config, LOCAL_TIMEOUT).await.unwrap();

        let lido = template(&dir).await.unwrap();
        assert!(lido.contains("escreva o assunto aqui"), "{lido}");

        std::fs::remove_dir_all(&dir).ok();
    }
}
