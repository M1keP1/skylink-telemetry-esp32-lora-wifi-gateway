use kiwi_store::{Key, Store, Value, BorrowedEntry, StoreError};

fn main() -> Result<(), StoreError> {
    println!("=== Example 1: Basic Operations with Persistence ===\n");

    let store_path = "/tmp/example1_store";

    println!("Creating store with path: {}", store_path);
    let mut store = Store::with_path(store_path)?;

    println!("\nPutting values...");
    store.put(Key::Int(5), Value::Int(6));
    store.put(Key::Int(6), Value::Int(7));
    store.put(Key::Int(7), Value::Int(8));
    store.put(Key::String("name".into()), Value::String("Alice".into()));
    store.put(Key::String("age".into()), Value::Int(25));

    let key_to_get = Key::Int(5);
    match store.get(&key_to_get) {
        Ok(value) => println!("Got value for key 5: {:?}", value),
        Err(e) => println!("Error getting key 5: {}", e),
    }

    println!("\n=== Demonstrating Error Pattern Matching ===");
    let missing_key = Key::Int(999);
    match store.get(&missing_key) {
        Ok(value) => println!("Got value for key 999: {:?}", value),
        Err(StoreError::KeyNotFound(k)) => {
            println!("✓ Key {:?} not found (expected)", k);
        }
        Err(StoreError::DataCorruption { .. }) => {
            println!("Data corruption detected!");
        }
        Err(StoreError::InvalidData { .. }) => {
            println!("Invalid data format!");
        }
        Err(e) => println!("Other error: {}", e),
    }

    println!("\n=== Store Contents ===");
    store.display_all()?;

    println!("\n=== Buffer-based iteration (values only) ===");
    for value_result in store.buffer_iter() {
        match value_result {
            Ok(value) => println!("{:?}", value),
            Err(e) => println!("Error during iteration: {}", e),
        }
    }

    println!("\n✓ Store will auto-save on drop!");
    println!("  Files created:");
    println!("    - {}.meta", store_path);
    println!("    - {}.keys", store_path);
    println!("    - {}.data", store_path);

    Ok(())
}