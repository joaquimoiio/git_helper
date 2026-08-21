//! Traduz o stderr do git para uma frase que uma pessoa entende.
//!
//! A regra da casa é dura e vale aqui mais que em qualquer lugar: **nunca despejar stderr cru na
//! tela**. "fatal: Could not read from remote repository." não diz a ninguém o que fazer, e
//! quatro linhas de inglês com um link para o troubleshooting do GitLab dizem menos ainda.
//!
//! O cru não some — ele continua no log do job, atrás do "ver detalhes". O que muda é o que a
//! interface diz **primeiro**: uma frase e, quando existe, a próxima coisa a fazer.
//!
//! A ordem das checagens importa. O git empilha as mensagens: um `Permission denied (publickey)`
//! vem seguido do genérico `Could not read from remote repository`, e quem casasse o genérico
//! primeiro perderia a informação boa. Por isso o específico vem sempre antes.
//!
//! As amostras dos testes são stderr **real**, capturado dos casos correspondentes.

/// Categoria do erro. A UI usa para decidir tratamento; o teste, para não depender do texto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    /// DNS não resolveu.
    UnknownHost,
    /// A identidade do servidor SSH não confere (ou nunca foi aceita).
    HostKey,
    /// Chave SSH ou senha recusada.
    AuthDenied,
    /// Precisava de credencial e não havia como pedir.
    CredentialsNeeded,
    /// O repositório não existe — ou existe e não podemos vê-lo, que para o git é o mesmo 404.
    NotFound,
    /// Autenticou, mas não tem acesso.
    Forbidden,
    /// Rede indisponível, conexão caída no meio, timeout.
    Network,
    /// A pasta de destino já tem coisa dentro.
    DestinationNotEmpty,
    /// A branch pedida não existe no remoto.
    BranchNotFound,
    /// Repositório com LFS e o `git-lfs` não está instalado.
    LfsMissing,
    /// Não reconhecemos. A frase vira genérica e o cru continua disponível.
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Diagnosis {
    pub kind: FailureKind,
    /// Frase curta, em português, sem jargão.
    pub message: String,
    /// A próxima coisa a fazer. `None` quando não há conselho honesto a dar — inventar um é
    /// pior do que admitir que não sabemos.
    pub action: Option<String>,
}

impl Diagnosis {
    fn new(kind: FailureKind, message: &str, action: Option<&str>) -> Self {
        Self {
            kind,
            message: message.to_owned(),
            action: action.map(str::to_owned),
        }
    }
}

/// Lê o stderr acumulado e decide o que dizer.
pub fn diagnose(stderr: &str) -> Diagnosis {
    // O git mistura maiúsculas conforme a fonte da mensagem (curl, ssh, ele mesmo).
    let text = stderr.to_lowercase();
    let has = |needle: &str| text.contains(needle);

    // --- host e rede ---------------------------------------------------------------------
    if has("could not resolve host") || has("name or service not known") {
        return Diagnosis::new(
            FailureKind::UnknownHost,
            "não encontrei esse servidor",
            Some("confira o endereço do repositório e se você está conectado à internet"),
        );
    }

    if has("remote host identification has changed") {
        return Diagnosis::new(
            FailureKind::HostKey,
            "a identidade do servidor mudou desde a última vez",
            // Deliberadamente sem oferecer um botão de "confiar assim mesmo": esta é
            // exatamente a mensagem que um ataque de intermediário produz.
            Some("isto também é o que um ataque de intermediário parece. confirme com quem administra o servidor antes de continuar"),
        );
    }

    if has("host key verification failed") {
        return Diagnosis::new(
            FailureKind::HostKey,
            "não confio na identidade desse servidor ainda",
            Some("conecte-se uma vez pelo terminal (`ssh -T git@servidor`) para conferir e aceitar a chave do host"),
        );
    }

    if has("connection timed out")
        || has("operation timed out")
        || has("connection refused")
        || has("network is unreachable")
        || has("failed to connect")
        || has("could not connect")
        || has("early eof")
        || has("rpc failed")
    {
        return Diagnosis::new(
            FailureKind::Network,
            "a conexão com o servidor falhou",
            Some("verifique a rede, o proxy e a VPN, e tente de novo"),
        );
    }

    // --- credencial ----------------------------------------------------------------------
    if has("permission denied (publickey") || has("permission denied (publickey,") {
        return Diagnosis::new(
            FailureKind::AuthDenied,
            "o servidor recusou sua chave SSH",
            Some("confira se a chave certa está no ssh-agent e se a chave pública está cadastrada no servidor"),
        );
    }

    if has("authentication failed")
        || has("http basic: access denied")
        || has("invalid username or password")
        || has("password authentication failed")
    {
        return Diagnosis::new(
            FailureKind::AuthDenied,
            "usuário ou senha recusados",
            Some("muitos servidores já não aceitam senha: use um token de acesso pessoal no lugar dela"),
        );
    }

    if has("terminal prompts disabled") || has("could not read username") {
        return Diagnosis::new(
            FailureKind::CredentialsNeeded,
            "o repositório exige credencial e não consegui pedi-la",
            Some("tente de novo; se o pedido de senha não aparecer, configure um credential helper do git"),
        );
    }

    // --- do outro lado -------------------------------------------------------------------
    if has("repository not found") || (has("not found") && has("fatal: repository")) {
        return Diagnosis::new(
            FailureKind::NotFound,
            "esse repositório não existe (ou você não tem acesso a ele)",
            Some("confira a URL. em repositório privado, o servidor responde a mesma coisa para quem não tem acesso"),
        );
    }

    if has("403 forbidden") || has("you don't have permission") || has("access denied") {
        return Diagnosis::new(
            FailureKind::Forbidden,
            "você não tem permissão nesse repositório",
            Some("peça acesso a quem administra o repositório"),
        );
    }

    if has("remote branch") && has("not found in upstream") {
        return Diagnosis::new(
            FailureKind::BranchNotFound,
            "essa branch não existe no repositório remoto",
            Some("deixe o campo de branch vazio para clonar a branch padrão"),
        );
    }

    if has("already exists and is not an empty directory") {
        return Diagnosis::new(
            FailureKind::DestinationNotEmpty,
            "a pasta de destino já existe e não está vazia",
            Some("escolha outro nome de pasta"),
        );
    }

    if has("git-lfs") && (has("not found") || has("is not a git command")) {
        return Diagnosis::new(
            FailureKind::LfsMissing,
            "esse repositório usa git-lfs, que não está instalado",
            Some("instale o git-lfs e tente de novo"),
        );
    }

    // Genérico do ssh: vem depois de todos os específicos justamente porque o git o empilha
    // atrás deles. Chegar aqui significa que o ssh não disse mais nada — e as duas causas
    // possíveis são estas duas mesmo.
    if has("could not read from remote repository") {
        return Diagnosis::new(
            FailureKind::NotFound,
            "não consegui ler esse repositório",
            Some("ou o endereço está errado, ou sua chave não tem acesso a ele"),
        );
    }

    Diagnosis::new(
        FailureKind::Unknown,
        "o git não conseguiu concluir a operação",
        // Nenhum conselho: não sabemos o que aconteceu, e chutar mandaria o usuário para o
        // lugar errado. O stderr cru está a um clique.
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cada amostra abaixo é stderr real, capturado do caso que ela nomeia.
    fn kind(stderr: &str) -> FailureKind {
        diagnose(stderr).kind
    }

    #[test]
    fn host_que_nao_resolve() {
        assert_eq!(
            kind("fatal: unable to access 'https://host-que-nao-existe.invalid/r.git/': Could not resolve host: host-que-nao-existe.invalid"),
            FailureKind::UnknownHost
        );
    }

    #[test]
    fn repositorio_inexistente() {
        assert_eq!(
            kind("remote: Repository not found.\nfatal: repository 'https://github.com/nao-existe/nada.git/' not found"),
            FailureKind::NotFound
        );
    }

    #[test]
    fn chave_rejeitada_vence_o_generico_do_ssh() {
        // O git empilha: a linha útil vem antes das quatro genéricas. Casar o genérico primeiro
        // jogaria fora a única informação que serve.
        let real = "git@github.com: Permission denied (publickey).\n\
                    fatal: Could not read from remote repository.\n\n\
                    Please make sure you have the correct access rights\n\
                    and the repository exists.";

        assert_eq!(kind(real), FailureKind::AuthDenied);
    }

    #[test]
    fn host_key_nao_confiavel() {
        let real = "Host key verification failed.\n\
                    fatal: Could not read from remote repository.\n\n\
                    Please make sure you have the correct access rights\n\
                    and the repository exists.";

        assert_eq!(kind(real), FailureKind::HostKey);
    }

    #[test]
    fn host_key_trocada_nao_ganha_conselho_de_confiar() {
        let diagnosis = diagnose(
            "@@@@@ WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED! @@@@@\nHost key verification failed.",
        );

        assert_eq!(diagnosis.kind, FailureKind::HostKey);
        assert!(
            diagnosis
                .action
                .as_deref()
                .is_some_and(|action| action.contains("intermediário")),
            "trocar de chave é o que um ataque parece; a ação não pode ser 'confie assim mesmo'"
        );
    }

    #[test]
    fn senha_recusada() {
        assert_eq!(
            kind("remote: HTTP Basic: Access denied. If a password was provided for Git authentication, the password was incorrect\nfatal: Authentication failed for 'https://gitlab.com/x/y.git/'"),
            FailureKind::AuthDenied
        );
    }

    #[test]
    fn destino_nao_vazio_e_branch_inexistente() {
        assert_eq!(
            kind("fatal: destination path '/tmp/e3' already exists and is not an empty directory."),
            FailureKind::DestinationNotEmpty
        );
        assert_eq!(
            kind("fatal: Remote branch nao-existe not found in upstream origin"),
            FailureKind::BranchNotFound
        );
    }

    #[test]
    fn rede_caida() {
        assert_eq!(
            kind("fatal: unable to access 'https://github.com/x/y.git/': Failed to connect to github.com port 443 after 75000 ms: Couldn't connect to server"),
            FailureKind::Network
        );
        assert_eq!(
            kind("error: RPC failed; curl 18 transfer closed with outstanding read data remaining\nfatal: early EOF"),
            FailureKind::Network
        );
    }

    #[test]
    fn o_que_nao_reconhecemos_nao_ganha_conselho_inventado() {
        let diagnosis = diagnose("fatal: alguma coisa que nunca vimos antes");

        assert_eq!(diagnosis.kind, FailureKind::Unknown);
        assert_eq!(diagnosis.action, None);
        assert!(
            !diagnosis.message.contains("fatal"),
            "a frase é nossa, não do git"
        );
    }

    #[test]
    fn stderr_vazio_nao_entra_em_panico() {
        assert_eq!(kind(""), FailureKind::Unknown);
    }
}
