use kiwi_store::{Store, Key, Value, api::start_server};
use kiwi_store::telemetry::{TelemetryPacket, TelemetryConfig, FlightDetector};
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SkyLink Combined Receiver + API Server ===\n");

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

    println!("Both tasks running!");
    println!("\nTest the APIs:");
    println!("   REST: .\\test_api.ps1");
    println!("   REST: Invoke-RestMethod http://localhost:3000/api/telemetry?limit=10");
    println!("\nTest WebSocket live stream:");
    println!("   PowerShell: Install websocat: cargo install websocat");
    println!("   Then run: websocat ws://localhost:3000/ws/telemetry");
    println!("\n   JavaScript (browser console):");
    println!("   const ws = new WebSocket('ws://localhost:3000/ws/telemetry');");
    println!("   ws.onmessage = (e) => console.log(JSON.parse(e.data));");
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

    println!("Connecting to ESP32 at {}...", config.esp32_address());
    let stream = TcpStream::connect(config.esp32_address()).await?;
    println!("Connected to ESP32\n");

    let reader = BufReader::new(stream);
    let mut lines = reader.lines();
    let mut packet_count = 0;

    println!("Receiving telemetry packets...\n");

    while let Some(line) = lines.next_line().await? {
        let packet: TelemetryPacket = match serde_json::from_str(&line) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("JSON parse error: {}", e);
                continue;
            }
        };

        let key = format!("telem:{}", packet.timestamp);
        let value = serde_json::to_string(&packet)?;

        // Lock is acquired here and released at end of block
        {
            let mut store = store.lock().unwrap();
            store.put(
                Key::String(key.clone()),
                Value::String(value)
            );
        } // Lock is released here - API can now access the store

        // Broadcast packet to WebSocket clients (non-blocking)
        // If no clients are connected, this is a no-op
        let _ = broadcast_tx.send(packet.clone());

        print!(".");
        packet_count += 1;

        // Process flight detection
        if let Some(metadata) = detector.process_packet(&packet) {
            if metadata.max_altitude >= config.flight_detection.min_takeoff_altitude_m {
                let flight_key = format!("flight:{}", metadata.flight_id);
                let flight_value = serde_json::to_string(&metadata)?;

                let mut store = store.lock().unwrap();
                store.put(
                    Key::String(flight_key),
                    Value::String(flight_value)
                );

                println!("\nFlight detected! ID: {}", metadata.flight_id);
            }
        }

        // Status update every 10 packets
        if packet_count % 10 == 0 {
            println!(
                "\n[{}] Stored {} packets | seq={}, alt={:.1}m, speed={:.1}m/s, current={:.1}A, phase={}",
                packet.timestamp,
                packet_count,
                packet.seq,
                packet.baro.alt,
                packet.gps.speed,
                packet.battery.current,
                packet.phase
            );
        }
    }

    Ok(())
}
