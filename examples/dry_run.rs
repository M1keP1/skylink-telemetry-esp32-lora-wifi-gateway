use futures_util::StreamExt;
use kiwi_store::{Key, Store, Value, api::start_server, telemetry::*};
use std::sync::{Arc, Mutex};
use tokio::time::Duration;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SkyLink Dry Run (Simulation + Verification) ===\n");

    // 1. Setup Store and Broadcast
    let store = Arc::new(Mutex::new(Store::new()));
    let (broadcast_tx, _) = tokio::sync::broadcast::channel(100);

    let api_store = Arc::clone(&store);
    let api_broadcast = broadcast_tx.clone();

    // 2. Start API Server
    let server_port = 3001; // Use different port for dry run
    tokio::spawn(async move {
        if let Err(e) = start_server(api_store, api_broadcast, "127.0.0.1", server_port).await {
            eprintln!("API Server error: {}", e);
        }
    });

    // Wait for server to start
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("✓ API Server started on port {}", server_port);

    // 3. Populate Store with Simulated Flight
    // 3. Populate Store with Simulated Flight using FlightDetector
    println!("\n[Phase 1] Populating Store with Simulated Flight...");
    let start_ts = 1700000000;

    // Create Config for Detection
    let config = FlightDetectionConfig {
        start_altitude_m: 10.0,
        start_speed_ms: 2.0,
        end_altitude_m: 5.0,
        min_takeoff_altitude_m: 20.0,
        end_speed_ms: 1.0,
        ground_stable_duration_ms: 5000,
        timeout_duration_ms: 30000,
    };
    let mut detector = FlightDetector::new(config.clone());
    let simulator = simulator::FlightSimulator::new(start_ts);

    // Generate telemetry packets (Ascent -> Cruise -> Descent)
    for i in 0..120 {
        // Increased packet count to ensure flight ends correctly
        let packet = simulator.generate_packet(i);
        let ts = packet.timestamp;

        let key = format!("telem:{}", ts);
        let val = serde_json::to_string(&packet)?;

        {
            let mut s = store.lock().unwrap();
            s.put(Key::String(key), Value::String(val));
        }

        // Process Flight Detection
        // println!("DEBUG: TS={} Speed={} Alt={}", packet.timestamp, packet.gps.speed, packet.baro.alt);
        if let Some(metadata) = detector.process_packet(&packet) {
            println!("Flight Detected/Ended: {}", metadata.flight_id);
            let flight_key = format!("flight:{}", metadata.flight_id);
            let flight_val = serde_json::to_string(&metadata)?;
            {
                let mut s = store.lock().unwrap();
                s.put(Key::String(flight_key), Value::String(flight_val));
            }
        }

        // Also broadcast some to verify channel works
        let _ = broadcast_tx.send(packet);
    }
    println!("✓ Processed 100 packets via FlightDetector");

    // 4. Run Automated verifications
    println!("\n[Phase 2] Verifying Endpoints...");

    // Check Stats
    let client = reqwest::Client::new();
    let stats_url = format!("http://127.0.0.1:{}/api/stats", server_port);
    let stats: serde_json::Value = client.get(&stats_url).send().await?.json().await?;

    println!("Stats: {}", stats);
    assert_eq!(stats["total_flights"], 1, "Expected 1 flight");
    assert_eq!(stats["total_packets"], 120, "Expected 120 packets");
    println!("✓ /api/stats verification passed");

    // Check Flight List
    let flights_url = format!("http://127.0.0.1:{}/api/flights", server_port);
    let flights: serde_json::Value = client.get(&flights_url).send().await?.json().await?;
    let flight_list = flights["flights"].as_array().unwrap();
    assert_eq!(flight_list.len(), 1);
    // Flight ID is dynamic (timestamp based), just check it exists
    println!("✓ Flight detected with ID: {}", flight_list[0]["flight_id"]);
    println!("✓ /api/flights verification passed");

    // 5. WebSocket Verification
    println!("\n[Phase 3] Verifying WebSocket Streaming...");

    let ws_url = format!("ws://127.0.0.1:{}/ws/telemetry", server_port);
    let (mut ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");
    println!("✓ WebSocket connected");

    println!("Simulating live data...");
    // Simulate live data sending
    let live_tx = broadcast_tx.clone();
    tokio::spawn(async move {
        let sim_live = simulator::FlightSimulator::new(1700000000);
        for i in 60..65 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let packet = sim_live.generate_packet(i);
            let _ = live_tx.send(packet);
        }
    });

    // Receive from WebSocket
    let mut received_count = 0;
    while received_count < 5 {
        if let Some(msg) = ws_stream.next().await {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let _packet: TelemetryPacket = serde_json::from_str(&text)?;
                received_count += 1;
                print!(".");
            }
        }
    }
    println!("\n✓ Received {} live packets via WebSocket", received_count);

    println!("\n=== SUCCESS: Dry Run Verification Complete! ===");
    Ok(())
}
