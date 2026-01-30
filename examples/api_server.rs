use kiwi_store::{Store, api::start_server};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Skylink store server");

    let store = Arc::new(Mutex::new(Store::new()));

    {
        let store_guard = store.lock().unwrap();
        let key_count = store_guard.keys().count();
        println!("Skylink Store Loaded");
        println!("Packets in store: {}", key_count);
    }

    // Create broadcast channel
    let (broadcast_tx, _) = tokio::sync::broadcast::channel(100);

    let host = "127.0.0.1";
    let port = 3000;

    start_server(store, broadcast_tx, host, port).await?;
    Ok(())

}