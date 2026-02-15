use kiwi_store::telemetry::{FlightDetector, TelemetryConfig, TelemetryPacket};
use kiwi_store::{Key, Store, Value, api::start_server};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    tracing::info!("=== SkyLink Wet Run (Hardware Connected) ===");

    // Load configuration
    let config = TelemetryConfig::load()?;
    tracing::info!("Configuration loaded");
    tracing::info!(esp32_address = %config.esp32_address(), "ESP32 connection target");
    tracing::info!("API Server will start on http://127.0.0.1:3000");

    // Create shared store
    let store = Arc::new(Mutex::new(Store::new()));
    tracing::debug!("Shared store initialized");

    // Create broadcast channel for live streaming (capacity: 100 packets)
    let (broadcast_tx, _) = tokio::sync::broadcast::channel(100);
    tracing::debug!("Broadcast channel created for WebSocket streaming");

    // Clone store references for both tasks
    let receiver_store = Arc::clone(&store);
    let api_store = Arc::clone(&store);
    let api_broadcast = broadcast_tx.clone();
    let receiver_broadcast = broadcast_tx.clone();

    // Spawn API server task
    let api_task = tokio::spawn(async move {
        tracing::info!("Starting API server on http://127.0.0.1:3000");

        if let Err(e) = start_server(api_store, api_broadcast, "127.0.0.1", 3000).await {
            tracing::error!(error = %e, "API server error");
        }
    });

    // Small delay to let API server start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Spawn receiver task
    let receiver_task = tokio::spawn(async move {
        if let Err(e) = run_receiver(receiver_store, receiver_broadcast, config).await {
            tracing::error!(error = %e, "Receiver error");
        }
    });

    tracing::info!("Both tasks running! Waiting for hardware connection...");
    tracing::info!("Test the APIs:");
    tracing::info!("  REST: curl http://localhost:3000/api/stats");
    tracing::info!("  WS:   websocat ws://localhost:3000/ws/telemetry");
    tracing::info!("═══════════════════════════════════════════════════════════");

    // Wait for either task to complete (or Ctrl+C)
    tokio::select! {
        _ = receiver_task => tracing::info!("Receiver task completed"),
        _ = api_task => tracing::info!("API server task completed"),
    }

    Ok(())
}

async fn run_receiver(
    store: Arc<Mutex<Store>>,
    broadcast_tx: tokio::sync::broadcast::Sender<TelemetryPacket>,
    config: TelemetryConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut detector = FlightDetector::new(config.flight_detection.clone());
    let mut packet_count = 0;

    loop {
        tracing::info!(address = %config.esp32_address(), "Connecting to ESP32...");
        match TcpStream::connect(config.esp32_address()).await {
            Ok(stream) => {
                tracing::info!("Connected to ESP32");
                let reader = BufReader::new(stream);
                let mut lines = reader.lines();

                // Inner loop: Read packets until disconnect/error
                while let Some(line) = lines.next_line().await.unwrap_or_else(|e| {
                    tracing::error!(error = %e, "Error reading line");
                    None
                }) {
                    let packet: TelemetryPacket = match serde_json::from_str(&line) {
                        Ok(p) => p,
                        Err(e) => {
                            tracing::error!(error = %e, line = %line, "JSON parse error");
                            continue;
                        }
                    };

                    let key = format!("telem:{}", packet.timestamp);
                    let value = match serde_json::to_string(&packet) {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::error!(error = %e, "Serialization error");
                            continue;
                        }
                    };

                    // Lock is acquired here and released at end of block
                    {
                        let mut store = store.lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        store.put(Key::String(key.clone()), Value::String(value));
                    } // Lock is released here - API can now access the store

                    // Broadcast packet to WebSocket clients
                    let _ = broadcast_tx.send(packet.clone());

                    print!(".");
                    packet_count += 1;

                    // Process Flight Detection
                    if let Some(metadata) = detector.process_packet(&packet)
                        && metadata.max_altitude >= config.flight_detection.min_takeoff_altitude_m
                    {
                        let flight_key = format!("flight:{}", metadata.flight_id);
                        let flight_val = match serde_json::to_string(&metadata) {
                            Ok(v) => v,
                            Err(e) => {
                                tracing::error!(error = %e, "Metadata serialization error");
                                continue;
                            }
                        };

                        let mut store = store.lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        store.put(Key::String(flight_key), Value::String(flight_val));

                        tracing::info!(
                            flight_id = %metadata.flight_id,
                            max_altitude = metadata.max_altitude,
                            "Flight detected and stored"
                        );
                    }

                    // Status update every 10 packets
                    if packet_count % 10 == 0 {
                        tracing::debug!(
                            timestamp = packet.timestamp,
                            packet_count = packet_count,
                            seq = packet.seq,
                            altitude = packet.baro.alt,
                            phase = %packet.phase,
                            "Telemetry packet stored"
                        );
                    }
                }
                tracing::warn!("Disconnected from ESP32");
            }
            Err(e) => {
                tracing::error!(error = %e, "Connection failed");
            }
        }

        tracing::info!("Retrying in 5 seconds...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
