//! Camada HTTP: router axum, middlewares (origin, rate limit, auth, csrf), rotas,
//! WebSocket e embed do frontend. Nunca `use git2` — fala com `porc-git`.

#[cfg(unix)]
pub mod askpass;
pub mod config;
/// Proxy para o Vite. Existe só em debug e só com a feature ligada — ver `dev_proxy.rs`.
#[cfg(all(feature = "dev-proxy", debug_assertions))]
pub mod dev_proxy;
/// Frontend embutido. Exclui o proxy: os dois nunca são compilados juntos.
#[cfg(all(
    feature = "embed-web",
    not(all(feature = "dev-proxy", debug_assertions))
))]
pub mod embed;
pub mod jobs;
pub mod middleware;
pub mod repos;
pub mod routes;
pub mod session;
pub mod ws;

#[cfg(not(any(
    all(feature = "dev-proxy", debug_assertions),
    all(
        feature = "embed-web",
        not(all(feature = "dev-proxy", debug_assertions))
    )
)))]
compile_error!(
    "porc-server precisa de `dev-proxy` (em debug) ou `embed-web`: \
     sem uma das duas não há frontend para servir"
);

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::State,
    http::{
        header::{CONTENT_SECURITY_POLICY, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS},
        HeaderValue,
    },
    middleware::from_fn_with_state,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::{set_header::SetResponseHeaderLayer, trace::TraceLayer};

use session::Secret;

/// Política de conteúdo do binário de release.
///
/// `default-src 'self'` é viável de verdade aqui porque tudo — script, estilo, fonte,
/// ícone — sai do próprio processo. Não há CDN, não há analytics, não há `<iframe>`.
/// Junto vem o fecho contra as saídas laterais: sem `<base>` reescrito, sem plugin, sem
/// ser embutido por terceiro, sem POST para fora.
#[cfg(not(all(feature = "dev-proxy", debug_assertions)))]
const CSP: &str = "default-src 'self'; base-uri 'none'; object-src 'none'; \
                   frame-ancestors 'none'; form-action 'none'";

/// Em dev a política é **mais frouxa**, e não há como não ser: o Vite injeta um `<script>`
/// inline no HTML e reescreve estilo em tempo de execução, e o cliente de HMR abre um
/// WebSocket para a porta 5173. O binário que chega ao usuário nunca usa esta versão — o
/// `cfg` acima garante isso.
#[cfg(all(feature = "dev-proxy", debug_assertions))]
const CSP: &str = "default-src 'self'; script-src 'self' 'unsafe-inline'; \
                   style-src 'self' 'unsafe-inline'; \
                   connect-src 'self' ws://127.0.0.1:5173; \
                   base-uri 'none'; object-src 'none'; frame-ancestors 'none'";

/// Porta preferida. Se estiver ocupada, o Passo 4 cai para porta efêmera.
const PREFERRED_PORT: u16 = 7867;

const APP_NAME: &str = "porcelain";
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 16 hex = 64 bits. Não é segredo, só precisa não colidir entre boots.
const INSTANCE_ID_LEN: usize = 16;

#[derive(Debug, thiserror::Error)]
pub enum ServeError {
    #[error("não consegui abrir {addr}: {source}")]
    Bind {
        addr: SocketAddr,
        #[source]
        source: std::io::Error,
    },
    #[error("o servidor parou de forma inesperada: {0}")]
    Serve(#[source] std::io::Error),
    #[error(transparent)]
    Token(#[from] session::TokenError),
}

/// Estado compartilhado por todo request. Clonado a cada um, então tudo aqui dentro é
/// barato de clonar.
#[derive(Clone)]
pub struct AppState {
    pub session: Secret,
    pub csrf: Secret,
    /// Identifica este boot para o lockfile. Público por definição.
    pub instance_id: String,
    /// Porta real em que estamos servindo. O guard de `Host` compara contra ela.
    pub port: u16,
    /// Config resolvida: raiz do navegador de pastas (já canônica) e limites da varredura.
    /// Nenhum caminho vindo do cliente escapa da raiz. `Arc` porque o estado é clonado por
    /// request.
    pub settings: Arc<config::Settings>,
    /// Repositórios abertos neste boot. Só o usuário o preenche, e some quando o processo sai.
    pub repos: Arc<repos::Registry>,
    /// Índice em SQLite. Descartável: se o arquivo não abrir, ele cai para memória e o app sobe
    /// do mesmo jeito — o que se perde é a memória entre boots, não o acesso ao git.
    pub index: Arc<porc_index::Index>,
    /// Jobs em andamento e o canal de eventos que o WebSocket entrega.
    pub jobs: Arc<jobs::Jobs>,
    /// Pedidos de senha esperando resposta do usuário. A passphrase passa por aqui em memória e
    /// segue para o socket; não é guardada em lugar nenhum.
    #[cfg(unix)]
    pub prompts: Arc<askpass::Prompts>,
    /// Avisado uma vez, por quem pedir o encerramento.
    shutdown: Arc<tokio::sync::Notify>,
}

/// Assinatura que a segunda instância usa para confirmar que quem atende na porta do
/// lockfile é mesmo este `porc`, e não outro processo que herdou a porta.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    pub name: String,
    pub version: String,
    pub pid: u32,
    /// Prefixo do identificador de instância — **não** é derivado do token de sessão, para
    /// que a rota pública `/health` não vaze nem um bit do segredo.
    pub instance_id: String,
}

async fn health(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        name: APP_NAME.to_owned(),
        version: VERSION.to_owned(),
        pid: std::process::id(),
        instance_id: state.instance_id.clone(),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WhoAmI {
    authenticated: bool,
    pid: u32,
    version: &'static str,
}

/// Rota mínima atrás da autenticação, para provar que a camada funciona. Provisória:
/// vira informação de repositório quando o Bloco C chegar.
async fn whoami() -> Json<WhoAmI> {
    Json(WhoAmI {
        authenticated: true,
        pid: std::process::id(),
        version: VERSION,
    })
}

/// POST mínimo que muda nada, só para exercitar a camada de CSRF de ponta a ponta.
/// Provisório — sai quando houver mutação de verdade no Bloco E.
async fn ping() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

/// `POST /api/v1/shutdown` — pede o encerramento e responde antes de ele acontecer.
///
/// Só avisa: quem de fato para é o `with_graceful_shutdown`, que deixa esta resposta e as
/// outras em andamento terminarem. Fechar o socket aqui deixaria o navegador com um erro
/// de rede em vez de um "encerrado".
async fn shutdown(State(state): State<AppState>) -> Json<serde_json::Value> {
    tracing::info!("encerramento pedido pela interface");
    // `notify_one` e não `notify_waiters`: se por algum motivo ninguém estiver esperando
    // ainda, o aviso fica guardado em vez de se perder.
    state.shutdown.notify_one();

    Json(serde_json::json!({ "ok": true }))
}

/// Une os dois caminhos de encerramento: `Ctrl+C` e a rota.
async fn shutdown_signal(notify: Arc<tokio::sync::Notify>) {
    let notified = notify.notified();

    tokio::select! {
        _ = notified => {}
        result = tokio::signal::ctrl_c() => {
            match result {
                Ok(()) => tracing::info!("Ctrl+C recebido"),
                // Sem handler de sinal não há como parar limpo; seguir servindo é pior.
                Err(err) => tracing::error!(%err, "não consegui escutar Ctrl+C"),
            }
        }
    }
}

/// Monta o router. A ordem das camadas é fixa e cresce nos passos seguintes:
/// `trace → origin_guard → rate_limit → auth → csrf → rotas`.
///
/// No `ServiceBuilder` a primeira camada é a mais externa, então a lista abaixo se lê na
/// mesma ordem em que o request atravessa.
pub fn router(state: AppState) -> Router {
    let layers = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::overriding(
            CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(CSP),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(from_fn_with_state(state.clone(), middleware::origin::guard))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::auth::require_session,
        ))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::csrf::require_double_submit,
        ));

    let router = Router::new()
        .route("/health", get(health))
        .route("/api/v1/session", post(session::create))
        .route("/api/v1/whoami", get(whoami))
        .route("/api/v1/fs/list", get(routes::fs::list))
        .route("/api/v1/fs/scan", get(routes::fs::scan))
        .route(
            "/api/v1/repos",
            get(routes::repos::list).post(routes::repos::open),
        )
        // Estática antes da paramétrica: o matchit do axum dá prioridade ao segmento literal,
        // então `init` nunca é confundido com um `repo_id`.
        .route("/api/v1/repos/init", post(routes::repos::init))
        // Chaves e não `:id`: é a sintaxe de parâmetro do axum 0.8.
        .route("/api/v1/repos/{repo_id}", get(routes::repos::get))
        .route("/api/v1/repos/{repo_id}/log", get(routes::repos::log))
        .route("/api/v1/recents", get(routes::recents::list))
        .route(
            "/api/v1/recents/{repo_id}",
            axum::routing::delete(routes::recents::forget),
        )
        .route("/api/v1/jobs", get(routes::jobs::list))
        // O tipo é segmento estático, não parâmetro: cada tipo tem corpo próprio.
        .route("/api/v1/jobs/test", post(routes::jobs::create_test))
        .route("/api/v1/jobs/clone", post(routes::jobs::create_clone))
        .route(
            "/api/v1/jobs/{job_id}",
            get(routes::jobs::get).delete(routes::jobs::cancel),
        );

    #[cfg(unix)]
    let router = router.route(
        "/api/v1/jobs/{job_id}/askpass",
        post(routes::jobs::answer_askpass),
    );

    let router = router
        // O upgrade atravessa as mesmas camadas do resto: `/api/` significa `origin::guard` e
        // `auth::require_session` antes do handshake. WebSocket não sofre CORS e escapa de
        // `SameSite`, então um socket fora dessas camadas seria a porta dos fundos do app.
        .route("/api/v1/ws", get(ws::upgrade))
        .route("/api/v1/ping", post(ping))
        .route("/api/v1/shutdown", post(shutdown));

    // Quem serve o frontend: em debug, o Vite via proxy; em release, os bytes embutidos.
    // Nos dois casos é `fallback`, porque a UI é uma SPA — qualquer caminho que não seja
    // rota do servidor pertence a ela.
    #[cfg(all(feature = "dev-proxy", debug_assertions))]
    let router = router.fallback(dev_proxy::proxy);
    #[cfg(all(
        feature = "embed-web",
        not(all(feature = "dev-proxy", debug_assertions))
    ))]
    let router = router.fallback(embed::serve);

    router.layer(layers).with_state(state)
}

/// Socket já aberto, com a porta real conhecida.
///
/// Separar `bind` de `serve` é o que permite ao CLI imprimir a URL, gravar o lockfile e
/// abrir o navegador **antes** de entregar a thread ao axum — e com a porta que o SO
/// realmente deu, não com a que pedimos.
pub struct Bound {
    listener: tokio::net::TcpListener,
    addr: SocketAddr,
    state: AppState,
}

impl Bound {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    /// Token de boot, para o CLI montar a URL de handshake (`?t=…`).
    pub fn token(&self) -> &str {
        self.state.session.as_str()
    }

    /// Identificador desta instância, o que vai para o lockfile.
    pub fn instance_id(&self) -> &str {
        &self.state.instance_id
    }

    /// URL que o navegador abre no boot. O `?t=` é trocado por cookie no primeiro request
    /// e some da barra de endereços — não fica no histórico nem no `Referer`.
    pub fn handshake_url(&self) -> String {
        format!("{}/?t={}", self.url(), self.token())
    }

    /// URL de origem do app. Sempre `127.0.0.1` literal — é o mesmo host que o guard de
    /// `Host`/`Origin` vai exigir.
    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Serve até alguém pedir para parar — pela rota de shutdown ou por `Ctrl+C`.
    pub async fn serve(self) -> Result<(), ServeError> {
        tracing::info!(addr = %self.addr, "porcelain no ar");

        let jobs = self.state.jobs.clone();
        let notify = self.state.shutdown.clone();

        // O aviso de parada e a faxina, nesta ordem: o axum só para de aceitar depois que este
        // futuro resolve, então cancelar aqui dentro garante que nenhum clone fica pela metade
        // com a limpeza por rodar.
        let signal = async move {
            shutdown_signal(notify).await;
            jobs.shutdown().await;
        };

        axum::serve(self.listener, router(self.state))
            .with_graceful_shutdown(signal)
            .await
            .map_err(ServeError::Serve)?;

        tracing::info!("servidor encerrado");
        Ok(())
    }
}

/// Pergunta a `/health` de quem estiver na porta quem ele é.
///
/// Mora aqui, e não no `porc-cli`, porque quem define o formato de `/health` é o servidor
/// — o CLI só decide o que fazer com a resposta. `None` significa "não é um `porc` vivo":
/// porta fechada, timeout, ou outro programa qualquer atendendo ali.
pub async fn probe(port: u16) -> Option<Health> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    // Timeout curto: isto roda no caminho do boot, que tem orçamento de 1s inteiro.
    let probe = async {
        let mut stream = tokio::net::TcpStream::connect(addr).await.ok()?;

        let request =
            format!("GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).await.ok()?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response).await.ok()?;

        // Corpo é o que vem depois da linha em branco que fecha os headers. `Connection:
        // close` evita ter que interpretar `Content-Length` ou chunked.
        let body = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|at| &response[at + 4..])?;

        serde_json::from_slice::<Health>(body).ok()
    };

    match tokio::time::timeout(std::time::Duration::from_millis(500), probe).await {
        Ok(health) => health.filter(|health| health.name == APP_NAME),
        Err(_) => {
            tracing::debug!(%addr, "health não respondeu a tempo");
            None
        }
    }
}

/// Abre o socket em `127.0.0.1`: nunca `0.0.0.0`, nunca `[::]`, sem flag para mudar.
///
/// Porta ocupada não é erro fatal — cai para porta efêmera (`:0`), que é o caso normal de
/// já haver outra instância ou outro serviço na 7867.
pub async fn bind(preferred: Option<u16>) -> Result<Bound, ServeError> {
    let preferred = SocketAddr::from((Ipv4Addr::LOCALHOST, preferred.unwrap_or(PREFERRED_PORT)));

    let listener = match tokio::net::TcpListener::bind(preferred).await {
        Ok(listener) => listener,
        Err(source) if source.kind() == std::io::ErrorKind::AddrInUse => {
            tracing::debug!(addr = %preferred, "porta ocupada, caindo para porta efêmera");

            let ephemeral = SocketAddr::from((Ipv4Addr::LOCALHOST, 0));
            tokio::net::TcpListener::bind(ephemeral)
                .await
                .map_err(|source| ServeError::Bind {
                    addr: ephemeral,
                    source,
                })?
        }
        Err(source) => {
            return Err(ServeError::Bind {
                addr: preferred,
                source,
            })
        }
    };

    // A porta real vem do listener, não do que pedimos: com `:0` só o SO sabe qual é.
    let addr = listener.local_addr().map_err(|source| ServeError::Bind {
        addr: preferred,
        source,
    })?;

    let state = AppState {
        session: Secret::generate()?,
        csrf: Secret::generate()?,
        instance_id: Secret::generate()?.as_str()[..INSTANCE_ID_LEN].to_owned(),
        port: addr.port(),
        settings: Arc::new(config::Settings::load()),
        repos: Arc::new(repos::Registry::default()),
        index: Arc::new(porc_index::Index::open_or_memory(
            porc_index::Index::default_path().as_deref(),
        )),
        jobs: Arc::new(jobs::Jobs::default()),
        #[cfg(unix)]
        prompts: Arc::new(askpass::Prompts::default()),
        shutdown: Arc::new(tokio::sync::Notify::new()),
    };

    Ok(Bound {
        listener,
        addr,
        state,
    })
}
