use kiwi_store::telemetry::{FlightDetector, TelemetryConfig, TelemetryPacket};
use kiwi_store::{Key, Store, Value, api::start_server};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SkyLink Wet Run (Hardware Connected) ===\n");

    // Load configuration
    let config = TelemetryConfig::load()?;
    println!("✓ Config Loaded");
    println!("  ESP32 Address: {}", config.esp32_address());
    println!("  API Server: http://127.0.0.1:3000\n");

    // Create shared store
    let store = Arc::new(Mutex::new(Store::new()));
    println!("Shared Store initialized");

    // Create broadcast channel for live streaming (capacity: 100 packets)
    let (broadcast_tx, _) = tokio::sync::broadcast::channel(100);
    println!("Broadcast channel created for WebSocket streaming\n");

    // Clone store references for both tasks
    let receiver_store = Arc::clone(&store);
    let api_store = Arc::clone(&store);
    let api_broadcast = broadcast_tx.clone();
    let receiver_broadcast = broadcast_tx.clone();

    // Spawn API server task
    let api_task = tokio::spawn(async move {
        println!("Starting API Server on http://127.0.0.1:3000");
        println!("   Available endpoints:");
        println!("   - GET /api/telemetry?start=X&end=Y&limit=Z");
        println!("   - GET /api/telemetry/:timestamp");
        println!("   - GET /api/flights");
        println!("   - GET /api/flights/:id");
        println!("   - GET /api/stats");
        println!("   - WS  /ws/telemetry (Live stream)\n");

        if let Err(e) = start_server(api_store, api_broadcast, "127.0.0.1", 3000).await {
            eprintln!("API Server error: {}", e);
        }
    });

    // Small delay to let API server start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Spawn receiver task
    let receiver_task = tokio::spawn(async move {
        if let Err(e) = run_receiver(receiver_store, receiver_broadcast, config).await {
            eprintln!("Receiver error: {}", e);
        }
    });

    println!("Both tasks running! Waiting for hardware connection...");
    println!("\nTest the APIs:");
    println!("   REST: Invoke-RestMethod http://localhost:3000/api/stats");
    println!("   WS:   websocat ws://localhost:3000/ws/telemetry");
    println!("\n═══════════════════════════════════════════════════════════\n");

    // Wait for either task to complete (or Ctrl+C)
    tokio::select! {
        _ = receiver_task => println!("Receiver task completed"),
        _ = api_task => println!("API server task completed"),
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
        println!("Connecting to ESP32 at {}...", config.esp32_address());
        match TcpStream::connect(config.esp32_address()).await {
            Ok(stream) => {
                println!("Connected to ESP32\n");
                let reader = BufReader::new(stream);
                let mut lines = reader.lines();

                // Inner loop: Read packets until disconnect/error
                while let Some(line) = lines.next_line().await.unwrap_or_else(|e| {
                    eprintln!("Error reading line: {}", e);
                    None
                }) {
                    let packet: TelemetryPacket = match serde_json::from_str(&line) {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("JSON parse error: {}", e);
                            continue;
                        }
                    };

                    let key = format!("telem:{}", packet.timestamp);
                    let value = match serde_json::to_string(&packet) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("Serialization error: {}", e);
                            continue;
                        }
                    };

                    // Lock is acquired here and released at end of block
                    {
                        let mut store = store.lock().unwrap();
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
                                eprintln!("Metadata Serialization error: {}", e);
                                continue;
                            }
                        };

                        let mut store = store.lock().unwrap();
                        store.put(Key::String(flight_key), Value::String(flight_val));

                        println!("\nFlight detected! ID: {}", metadata.flight_id);
                    }

                    // Status update every 10 packets
                    if packet_count % 10 == 0 {
                        println!(
                            "\n[{}] Stored {} packets | seq={}, alt={:.1}m, phase={}",
                            packet.timestamp,
                            packet_count,
                            packet.seq,
                            packet.baro.alt,
                            packet.phase
                        );
                    }
                }
                println!("\nDisconnected from ESP32.");
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }

        println!("Retrying in 5 seconds...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
