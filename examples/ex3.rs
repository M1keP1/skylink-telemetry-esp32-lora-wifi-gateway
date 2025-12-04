// Showcase demonstrating error handling with anyhow:
// 1. Key not found error
// 2. Buffer iterator showing graceful error handling

use kiwi_store::{Store, Key, Value};
use anyhow::Result;

fn main() -> Result<()> {
    example_1_key_not_found();
    example_2_iterator_error_handling()?;
    Ok(())
}

fn example_1_key_not_found() {
    println!("=== Example 1: Key Not Found ===");
    let store = Store::new();

    match store.get(&Key::String("nonexistent".into())) {
        Ok(value) => println!("Value: {:?}", value),
        Err(e) => println!("Error: {}", e),
    }
    println!();
}

fn example_2_iterator_error_handling() -> Result<()> {
    println!("=== Example 2: Iterator Error Handling ===");
    let mut store = Store::new();

    store.put(Key::String("key1".into()), Value::Int(100));
    store.put(Key::String("key2".into()), Value::String("hello".into()));
    store.put(Key::String("key3".into()), Value::Int(200));

    println!("Iterating with error handling:");
    for (key, value_result) in store.iter() {
        match value_result {
            Ok(value) => println!("  {:?} => {:?}", key, value),
            Err(e) => println!("  Error reading {:?}: {}", key, e),
        }
    }

    Ok(())
}