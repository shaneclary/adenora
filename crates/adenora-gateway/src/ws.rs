use crate::state::AppState;
use axum::{
    extract::{
        State,
        WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum WsCommand {
    /// Subscribe to a market's order book updates
    Subscribe {
        market_id: Uuid,
        // Interface scaffolding: retained for planned people/bot book selection.
        #[allow(dead_code)]
        mode: Option<String>, // "people", "bot", or "both" (default)
    },
    /// Unsubscribe from a market
    Unsubscribe {
        market_id: Uuid,
    },
    /// Request current order book snapshot
    Snapshot {
        market_id: Uuid,
    },
    /// Ping
    Ping,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // Send welcome message
    let welcome = json!({
        "type": "welcome",
        "service": "adenora",
        "message": "connected — send {\"cmd\":\"subscribe\",\"market_id\":\"...\"} to start"
    });
    if socket.send(Message::Text(welcome.to_string())).await.is_err() {
        return;
    }

    let mut subscribed_markets: Vec<Uuid> = Vec::new();
    let mut snapshot_interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

    loop {
        tokio::select! {
            // Handle incoming messages
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<WsCommand>(&text) {
                            Ok(WsCommand::Subscribe { market_id, mode: _ }) => {
                                if !subscribed_markets.contains(&market_id) {
                                    subscribed_markets.push(market_id);
                                }
                                let resp = json!({
                                    "type": "subscribed",
                                    "market_id": market_id
                                });
                                if socket.send(Message::Text(resp.to_string())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(WsCommand::Unsubscribe { market_id }) => {
                                subscribed_markets.retain(|id| *id != market_id);
                                let resp = json!({
                                    "type": "unsubscribed",
                                    "market_id": market_id
                                });
                                if socket.send(Message::Text(resp.to_string())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(WsCommand::Snapshot { market_id }) => {
                                let engine = state.get_engine(market_id).await;
                                let snaps = engine.snapshots().await;
                                let resp = json!({
                                    "type": "snapshot",
                                    "market_id": market_id,
                                    "people": snaps.people,
                                    "bot": snaps.bot
                                });
                                if socket.send(Message::Text(resp.to_string())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(WsCommand::Ping) => {
                                let resp = json!({"type": "pong"});
                                if socket.send(Message::Text(resp.to_string())).await.is_err() {
                                    break;
                                }
                            }
                            Err(_) => {
                                let resp = json!({"type": "error", "message": "invalid command"});
                                if socket.send(Message::Text(resp.to_string())).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            // Push order book updates for subscribed markets
            _ = snapshot_interval.tick() => {
                for market_id in &subscribed_markets {
                    let engine = state.get_engine(*market_id).await;
                    let snaps = engine.snapshots().await;
                    let update = json!({
                        "type": "book_update",
                        "market_id": market_id,
                        "people": {
                            "best_bid": snaps.people.best_bid,
                            "best_ask": snaps.people.best_ask,
                            "spread": snaps.people.spread
                        },
                        "bot": {
                            "best_bid": snaps.bot.best_bid,
                            "best_ask": snaps.bot.best_ask,
                            "spread": snaps.bot.spread
                        }
                    });
                    if socket.send(Message::Text(update.to_string())).await.is_err() {
                        return;
                    }
                }
            }
        }
    }
}
