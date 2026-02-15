use super::handlers::AppState;
use crate::telemetry::TelemetryPacket;
use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use tokio::sync::broadcast;

/// WebSocket handler for live telemetry streaming
pub async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
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

#[cfg(test)]
mod tests {

    use crate::Store;
    use crate::api::server::create_router;
    use crate::telemetry::{
        AccelData, Barodata, BatteryData, GpsData, GyroData, ImuData, LinkQuality, TelemetryPacket,
    };
    use futures_util::StreamExt;
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;
    use tokio::sync::broadcast;
    use tokio_tungstenite::tungstenite::Message;

    #[tokio::test]
    async fn test_websocket_broadcast() {
        // 1. Setup Store and Broadcast Channel
        let store = Arc::new(Mutex::new(Store::new()));
        let (tx, _rx) = broadcast::channel(100);
        let app = create_router(store, tx.clone());

        // 2. Start Test Server on random port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // 3. Connect WebSocket Client
        let ws_url = format!("ws://{}/ws/telemetry", addr);
        let (mut socket, _response) = tokio_tungstenite::connect_async(ws_url)
            .await
            .expect("Failed to connect");

        // 4. Send Telemetry Packet
        let packet = TelemetryPacket {
            seq: 1,
            timestamp: 1234567890,
            phase: "TEST".to_string(),
            gps: GpsData {
                lat: 0.0,
                lon: 0.0,
                alt: 0.0,
                speed: 0.0,
                heading: 0.0,
                sats: 0,
                fix: 0,
            },
            baro: Barodata {
                alt: 0.0,
                vspeed: 0.0,
                temp: 0.0,
            },
            imu: ImuData {
                roll: 0.0,
                pitch: 0.0,
                yaw: 0.0,
                gyro: GyroData {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                accel: AccelData {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            battery: BatteryData {
                voltage: 0.0,
                current: 0.0,
                power: 0.0,
                mah_used: 0.0,
            },
            link: LinkQuality {
                rssi: 0.0,
                snr: 0.0,
            },
            status: 0,
        };

        // Send to broadcast channel
        tx.send(packet.clone()).unwrap();

        // 5. Verify Receipt
        if let Some(msg) = socket.next().await {
            let msg = msg.expect("Error reading message");
            if let Message::Text(text) = msg {
                let received: TelemetryPacket = serde_json::from_str(&text).unwrap();
                assert_eq!(received.timestamp, 1234567890);
                assert_eq!(received.phase, "TEST");
            } else {
                panic!("Expected text message");
            }
        } else {
            panic!("Stream ended without message");
        }
    }
}
