use kiwi_store::{Store, Key, Value};
use kiwi_store::telemetry::{TelemetryPacket, TelemetryConfig, FlightDetector};
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SkyLink : Telemetry Test ===");

    let config = TelemetryConfig::load()?;
    println!("Config Loaded");
    println!("  Address: {}",config.esp32_address());

    let mut detector = FlightDetector::new(config.flight_detection.clone());
    let store = Arc::new(Mutex::new(Store::new()));
    println!("SkyLink Store initialized");

    let stream = TcpStream::connect(config.esp32_address()).await?;
    println!("Connected to Esp32");

    let reader = BufReader::new(stream);
    let mut lines = reader.lines();
    let mut packet_count = 0;

    println!("Receiving Telemetry Packets...");

    while let Some(line) = lines.next_line().await? {
        let packet: TelemetryPacket = match serde_json::from_str(&line) {
            Ok(p) => p,
            Err(e) => {
                println!("JSON parse error: {}", e);
                continue;
            }
        };

        let key = format!("telem:{}", packet.timestamp);
        let value = serde_json::to_string(&packet)?;

        {
            let mut store = store.lock().unwrap();
            store.put(
                Key::String(key.clone()),
                Value::String(value)
            );
        }
        print!("X");
        packet_count += 1;

        if let Some(metadata) = detector.process_packet(&packet) {
            if metadata.max_altitude >= config.flight_detection.min_takeoff_altitude_m {
                let flight_key = format!("flight:{}", metadata.flight_id);
                let flight_value = serde_json::to_string(&metadata)?;

                let mut store = store.lock().unwrap();
                store.put(
                    Key::String(flight_key),
                    Value::String(flight_value)
                );

                println!("\n Flight Stamp Added for id {}", metadata.flight_id);
            }
        }
        if packet_count % 10 == 0 {
            println!("[{}] Stored {} Packets | seq = {}, alt = {:.1}m, speed = {:.1}m/s, Amp = {:.1}A, Phase {} ",
                     packet.timestamp,
                     packet_count,
                     packet.seq,
                packet.baro.alt,
                packet.gps.speed,
                packet.battery.current,
                packet.phase
            );
        }

        if packet_count == 1000 {
            println!("SkyLink Telemetry Receiver Test PASS");
            let mut store = store.lock().unwrap();
            store.clear();
            println!("SkyLink Store Cleared");
            return Ok(());
        }
    }
    Ok(())

}