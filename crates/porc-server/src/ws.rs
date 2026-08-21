//! O WebSocket. **Um** só, multiplexado por `topic`.
//!
//! Um socket por assunto (job, repo.changed, search, term) seria quatro pontos para autenticar,
//! quatro para validar `Origin` e quatro para reconectar quando a máquina acorda do sleep. Aqui
//! é um: o cliente assina os assuntos que lhe interessam e o servidor filtra.
//!
//! **Segurança.** O upgrade passa pelas mesmas camadas de todo request — o `origin::guard` e o
//! `auth::require_session` envolvem o router inteiro, e o caminho começa com `/api/`. Isso não é
//! detalhe: WebSocket não sofre CORS e escapa de `SameSite`, então um socket que pulasse o guard
//! seria a porta dos fundos de todo o resto.
//!
//! **Perder evento é previsto.** O canal tem fôlego finito; um cliente lento recebe `resync` e
//! recarrega o estado por HTTP. É por isso que `GET /api/v1/jobs/{id}` devolve o estado
//! completo: o socket é o caminho rápido, não a fonte da verdade.

use std::collections::HashSet;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;

use crate::{jobs::ServerMessage, AppState};

/// Assinado sem o cliente pedir: são as respostas do próprio protocolo (`ready`, `pong`,
/// `resync`), e um cliente que não as recebesse não teria como perceber que ficou para trás.
const CONTROL: &str = "control";

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ClientMessage {
    Subscribe { topics: Vec<String> },
    Unsubscribe { topics: Vec<String> },
    Ping,
}

pub async fn upgrade(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| serve(socket, state))
}

async fn serve(mut socket: WebSocket, state: AppState) {
    let mut events = state.jobs.subscribe();
    let mut topics: HashSet<String> = HashSet::from([CONTROL.to_owned()]);

    if send(&mut socket, &ServerMessage::Ready).await.is_err() {
        return;
    }

    loop {
        tokio::select! {
            // O cliente. `None` é socket fechado; `Err` é socket quebrado — nos dois casos
            // acabou, e o `Drop` cuida do resto.
            incoming = socket.recv() => {
                let Some(Ok(message)) = incoming else { break };

                match message {
                    Message::Text(text) => {
                        let Some(reply) = handle_client(&text, &mut topics) else { continue };
                        if send(&mut socket, &reply).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => break,
                    // Ping/Pong do protocolo o axum responde sozinho; binário não usamos.
                    _ => {}
                }
            }

            event = events.recv() => {
                match event {
                    Ok(event) => {
                        if !topics.contains(event.topic()) {
                            continue;
                        }
                        if send(&mut socket, &event).await.is_err() {
                            break;
                        }
                    }
                    // Ficamos para trás. Não tentamos remontar o que se perdeu: mandamos o
                    // cliente reconsultar por HTTP, que é a única forma de ele voltar a um
                    // estado que sabemos ser verdadeiro.
                    Err(RecvError::Lagged(perdidos)) => {
                        tracing::debug!(perdidos, "cliente de WebSocket ficou para trás");
                        if send(&mut socket, &ServerMessage::Resync).await.is_err() {
                            break;
                        }
                    }
                    Err(RecvError::Closed) => break,
                }
            }
        }
    }

    tracing::debug!("WebSocket encerrado");
}

/// Devolve `Some` quando há resposta imediata a mandar.
fn handle_client(text: &str, topics: &mut HashSet<String>) -> Option<ServerMessage> {
    match serde_json::from_str::<ClientMessage>(text) {
        Ok(ClientMessage::Subscribe { topics: wanted }) => {
            topics.extend(wanted);
            None
        }
        Ok(ClientMessage::Unsubscribe { topics: unwanted }) => {
            for topic in unwanted {
                // `control` não se cancela: sem ele o cliente não recebe nem o `resync`.
                if topic != CONTROL {
                    topics.remove(&topic);
                }
            }
            None
        }
        Ok(ClientMessage::Ping) => Some(ServerMessage::Pong),
        Err(err) => {
            // Mensagem malformada não derruba a conexão: é quase sempre uma aba com o bundle
            // antigo depois de um deploy, e derrubar só a deixaria reconectando em laço.
            tracing::debug!(%err, "mensagem de WebSocket ignorada");
            None
        }
    }
}

async fn send(socket: &mut WebSocket, message: &ServerMessage) -> Result<(), ()> {
    let text = serde_json::to_string(message).map_err(|err| {
        tracing::error!(%err, "não consegui serializar mensagem de WebSocket");
    })?;

    socket
        .send(Message::Text(text.into()))
        .await
        .map_err(|err| {
            tracing::debug!(%err, "WebSocket fechou durante o envio");
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assina_e_cancela_assinatura() {
        let mut topics = HashSet::from([CONTROL.to_owned()]);

        handle_client(r#"{"type":"subscribe","topics":["job"]}"#, &mut topics);
        assert!(topics.contains("job"));

        handle_client(r#"{"type":"unsubscribe","topics":["job"]}"#, &mut topics);
        assert!(!topics.contains("job"));
    }

    #[test]
    fn control_nao_se_cancela() {
        let mut topics = HashSet::from([CONTROL.to_owned()]);

        handle_client(
            r#"{"type":"unsubscribe","topics":["control"]}"#,
            &mut topics,
        );
        assert!(topics.contains(CONTROL), "sem control não há resync");
    }

    #[test]
    fn ping_responde_pong_e_lixo_e_ignorado() {
        let mut topics = HashSet::new();

        assert!(matches!(
            handle_client(r#"{"type":"ping"}"#, &mut topics),
            Some(ServerMessage::Pong)
        ));
        assert!(handle_client("nada disso é JSON", &mut topics).is_none());
    }
}
