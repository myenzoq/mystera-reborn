use axum::{
    extract::{
        ws::{self, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use futures_util::{stream::StreamExt, SinkExt};
use std::sync::Arc;
use tokio_tungstenite::connect_async;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use url::Url;

#[derive(Clone)]
struct AppState {
    game_server_url_base: Url,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let game_server_url_base = Url::parse("ws://mysteralegacy.com").unwrap();

    let state = Arc::new(AppState {
        game_server_url_base,
    });

    let app = Router::new()
        .route("/ws/:server_id", get(ws_proxy_handler))
        .fallback_service(ServeDir::new("../../public"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn ws_proxy_handler(
    ws: WebSocketUpgrade,
    Path(server_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, server_id, state))
}

async fn handle_socket(mut client_socket: WebSocket, server_id: String, state: Arc<AppState>) {
    let mut url = state.game_server_url_base.clone();

    let host = format!("{}.{}", server_id, url.host_str().unwrap());
    url.set_host(Some(&host)).unwrap();

    tracing::debug!("Connecting to upstream server: {}", url);

    let (upstream_socket, _) = match connect_async(url.to_string()).await {
        Ok((socket, response)) => (socket, response),
        Err(e) => {
            tracing::error!("Could not connect to upstream server: {}", e);
            let _ = client_socket
                .send(ws::Message::Close(Some(ws::CloseFrame {
                    code: ws::close_code::ERROR,
                    reason: "Upstream connection failed".into(),
                })))
                .await;
            return;
        }
    };

    let (mut client_sender, mut client_receiver) = client_socket.split();
    let (mut upstream_sender, mut upstream_receiver) = upstream_socket.split();

    // Task to forward messages from client to upstream
    let client_to_upstream = async {
        while let Some(Ok(msg)) = client_receiver.next().await {
            let msg = match msg {
                ws::Message::Text(t) => tungstenite::Message::Text(t),
                ws::Message::Binary(b) => tungstenite::Message::Binary(b),
                ws::Message::Ping(p) => tungstenite::Message::Ping(p),
                ws::Message::Pong(p) => tungstenite::Message::Pong(p),
                ws::Message::Close(c) => {
                    let code = c.as_ref().map(|c| c.code.into());
                    let reason = c.as_ref().map(|c| c.reason.clone()).unwrap_or_default();
                    tungstenite::Message::Close(Some(tungstenite::protocol::CloseFrame {
                        code: tungstenite::protocol::frame::coding::CloseCode::from(code.unwrap_or(1000)),
                        reason,
                    }))
                }
            };
            if upstream_sender.send(msg).await.is_err() {
                break;
            }
        }
    };

    // Task to forward messages from upstream to client
    let upstream_to_client = async {
        while let Some(Ok(msg)) = upstream_receiver.next().await {
            let msg = match msg {
                tungstenite::Message::Text(t) => ws::Message::Text(t),
                tungstenite::Message::Binary(b) => ws::Message::Binary(b),
                tungstenite::Message::Ping(p) => ws::Message::Ping(p),
                tungstenite::Message::Pong(p) => ws::Message::Pong(p),
                tungstenite::Message::Close(c) => {
                    let code = c.as_ref().map(|c| c.code.into()).unwrap_or(1000);
                    let reason = c.as_ref().map(|c| c.reason.clone()).unwrap_or_default();
                    ws::Message::Close(Some(ws::CloseFrame { code, reason }))
                }
                tungstenite::Message::Frame(_) => continue, // Raw frames not supported by axum
            };
            if client_sender.send(msg).await.is_err() {
                break;
            }
        }
    };

    tokio::select! {
        _ = client_to_upstream => { tracing::debug!("Client disconnected."); },
        _ = upstream_to_client => { tracing::debug!("Upstream server disconnected."); },
    }
}
