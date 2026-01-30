use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use tokio::sync::broadcast;
use crate::telemetry::TelemetryPacket;
use super::handlers::AppState;

/// WebSocket handler for live telemetry streaming
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state.broadcast_tx))
}

/// Handle individual WebSocket connection
async fn handle_socket(mut socket: WebSocket, tx: broadcast::Sender<TelemetryPacket>) {
    let mut rx = tx.subscribe();

    println!("WebSocket client connected");

    // Send packets to the client
    loop {
        tokio::select! {
            // Receive packet from broadcast channel
            packet = rx.recv() => {
                match packet {
                    Ok(p) => {
                        // Serialize packet to JSON
                        let json = match serde_json::to_string(&p) {
                            Ok(j) => j,
                            Err(e) => {
                                eprintln!("Failed to serialize packet: {}", e);
                                continue;
                            }
                        };

                        // Send to WebSocket client
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break; // Client disconnected
                        }
                    }
                    Err(_) => break, // Channel closed
                }
            }
            // Receive messages from client (for close detection)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(_))) => {
                        // Pong is automatically handled by axum
                    }
                    Some(Err(_)) => break,
                    _ => {} // Ignore other messages
                }
            }
        }
    }

    println!("WebSocket client disconnected");
}
