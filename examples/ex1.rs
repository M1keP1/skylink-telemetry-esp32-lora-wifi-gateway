use kiwi_store::{Key, Store, Value, BorrowedEntry};
use anyhow::Result;

fn main() -> Result<()> {
    let mut store = Store::new();

    println!("Putting Key: 5 and Value: 6");
    store.put(Key::Int(5), Value::Int(6));
    store.put(Key::Int(6), Value::Int(7));
    store.put(Key::Int(7), Value::Int(8));
    
    let key_to_get = Key::Int(5);
    match store.get(&key_to_get) {
        Ok(value) => println!("Got value for key 5: {:?}", value),
        Err(e) => println!("Error getting key 5: {}", e),
    }
    
    let missing_key = Key::Int(2);
    match store.get(&missing_key) {
        Ok(value) => println!("Got value for key 2: {:?}", value),
        Err(e) => println!("Getting Key: 2 -> Error: {}", e),
    }

    store.display_all()?;

    println!("=== Buffer-based iteration (values only) ===");
    for value_result in store.buffer_iter() {
        match value_result {
            Ok(value) => println!("{:?}", value),
            Err(e) => println!("Error during iteration: {}", e),
        }
    }

    Ok(())
}