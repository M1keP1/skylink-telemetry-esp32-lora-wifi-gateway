use kiwi_store::{Store, Key, Value, StoreError};

fn main() -> Result<(), StoreError> {
    example_1_pattern_matching();
    example_2_different_error_types();
    example_3_iterator_error_handling()?;
    example_4_io_error_handling()?;
    Ok(())
}

fn example_1_pattern_matching() {
    println!("=== Example 1: Pattern Matching on Error Types ===");
    let store = Store::new();

    match store.get(&Key::String("nonexistent".into())) {
        Ok(value) => println!("Value: {:?}", value),
        Err(StoreError::KeyNotFound(k)) => {
            println!("✓ Key {:?} not found (expected)", k);
        }
        Err(StoreError::DataCorruption { .. }) => {
            println!("Data corruption");
        }
        Err(StoreError::InvalidData { .. }) => {
            println!("Invalid data format");
        }
        Err(e) => println!("Other error: {}", e),
    }
    println!();
}

fn example_2_different_error_types() {
    println!("=== Example 2: Handling Different Error Types Differently ===");
    let mut store = Store::new();

    store.put(Key::String("valid".into()), Value::Int(42));

    let keys_to_try = vec![
        Key::String("valid".into()),
        Key::String("missing".into()),
        Key::Int(999),
    ];

    for key in keys_to_try {
        match store.get(&key) {
            Ok(value) => {
                println!("✓ Successfully got {:?} = {:?}", key, value);
            }
            Err(StoreError::KeyNotFound(k)) => {
                println!("✗ Key {:?} not found - could use default value", k);
            }
            Err(StoreError::DataCorruption { .. }) => {
                println!("✗ CRITICAL: Data corruption detected!");
            }
            Err(StoreError::InvalidData { .. }) => {
                println!("✗ Invalid data format");
            }
            Err(e) => println!("✗ Other error: {}", e),
        }
    }
    println!();
}

fn example_3_iterator_error_handling() -> Result<(), StoreError> {
    println!("=== Example 3: Iterator with Type-Safe Error Handling ===");
    let mut store = Store::new();

    store.put(Key::String("key1".into()), Value::Int(100));
    store.put(Key::String("key2".into()), Value::String("hello".into()));
    store.put(Key::String("key3".into()), Value::Int(200));

    println!("Iterating with pattern matching on errors:");
    for (key, value_result) in store.iter() {
        match value_result {
            Ok(value) => println!("  {:?} => {:?}", key, value),
            Err(StoreError::DataCorruption { .. }) => {
                println!("  ALERT: Corrupted entry at {:?}", key);
            }
            Err(e) => println!("  Error reading {:?}: {}", key, e),
        }
    }
    println!();

    Ok(())
}

fn example_4_io_error_handling() -> Result<(), StoreError> {
    println!("=== Example 4: File I/O Error Handling ===");

    let store_path = "/tmp/ex3_error_demo";

    {
        let mut store = Store::with_path(store_path)?;
        store.put(Key::String("test".into()), Value::Int(123));
        println!("✓ Created store and added data");
    }

    println!("✓ Store auto-saved on drop");

    match Store::with_path(store_path) {
        Ok(loaded) => {
            println!("✓ Successfully loaded store from disk");
            match loaded.get(&Key::String("test".into())) {
                Ok(value) => println!("✓ Retrieved value: {:?}", value),
                Err(e) => println!("✗ Error retrieving value: {}", e),
            }
        }
        Err(StoreError::IoError(e)) => {
            println!("✗ I/O error loading store: {}", e);
        }
        Err(StoreError::FileCorrupted) => {
            println!("✗ Files are corrupted!");
        }
        Err(StoreError::UnsupportedVersion(v)) => {
            println!("✗ Unsupported file version: {}", v);
        }
        Err(e) => {
            println!("✗ Other error: {}", e);
        }
    }

    println!("\nTesting missing file (should create new store):");
    match Store::with_path("/tmp/ex3_nonexistent") {
        Ok(_) => println!("✓ Created new store for non-existent path"),
        Err(e) => println!("✗ Unexpected error: {}", e),
    }

    std::fs::remove_file(format!("{}.keys", store_path)).ok();
    std::fs::remove_file(format!("{}.data", store_path)).ok();
    std::fs::remove_file(format!("{}.meta", store_path)).ok();

    println!();
    Ok(())
}