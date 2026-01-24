use kiwi_store::telemetry::TelemetryConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("==='Kiwi Store - Config Test === \n");

    let config = TelemetryConfig::load()?;

    println!("ESP32 Configuration");
    println!("  Address: {}",config.esp32_address());

    println!("Storage Configuration");
    println!("  Persist: {}",config.storage.persists);
    println!("  Path: {}",config.storage.path);
    print!("    Auto Compact : {}",config.storage.auto_compact);

    println!(" Config Loadded Successfully");

    Ok(())

}