use kiwi_store::{Store, Key, Value, StoreError};
use std::io::{self, Write};

fn main() -> Result<(), StoreError> {
    println!("=== KV Store Interactive Menu ===\n");
    
    // Ask if user wants to load existing store or create new one
    println!("1. Create new store");
    println!("2. Load existing store");
    print!("\nChoice: ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let mut store = match input.trim() {
        "1" => Store::new(),
        "2" => {
            print!("Enter store path: ");
            io::stdout().flush().unwrap();
            let mut path = String::new();
            io::stdin().read_line(&mut path).unwrap();
            Store::with_path(path.trim())?
        }
        _ => {
            println!("Invalid choice, creating new store");
            Store::new()
        }
    };
    
    // Set path for auto-save if not already set
    if input.trim() == "1" {
        print!("Enter path to save store (or press Enter to skip): ");
        io::stdout().flush().unwrap();
        let mut path = String::new();
        io::stdin().read_line(&mut path).unwrap();
        let path = path.trim();
        if !path.is_empty() {
            store = Store::with_path(path)?;
        }
    }
    
    loop {
        println!("\n=== Main Menu ===");
        println!("1. Put (insert/update key-value)");
        println!("2. Get (retrieve value by key)");
        println!("3. Delete (remove key)");
        println!("4. Display all entries");
        println!("5. List all keys");
        println!("6. Compact store");
        println!("7. Show fragmentation ratio");
        println!("8. Save store");
        println!("9. Exit");
        print!("\nChoice: ");
        io::stdout().flush().unwrap();
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        
        match choice.trim() {
            "1" => put_value(&mut store)?,
            "2" => get_value(&store)?,
            "3" => delete_value(&mut store)?,
            "4" => display_all(&store)?,
            "5" => list_keys(&store),
            "6" => compact_store(&mut store)?,
            "7" => show_fragmentation(&store),
            "8" => save_store(&mut store)?,
            "9" => {
                println!("\nExiting... (store will auto-save if path is set)");
                break;
            }
            _ => println!("Invalid choice, please try again"),
        }
    }
    
    Ok(())
}

fn put_value(store: &mut Store) -> Result<(), StoreError> {
    println!("\n--- Put Value ---");
    println!("Key type: 1=String, 2=Int");
    print!("Choice: ");
    io::stdout().flush().unwrap();
    
    let mut key_type = String::new();
    io::stdin().read_line(&mut key_type).unwrap();
    
    let key = match key_type.trim() {
        "1" => {
            print!("Enter key (string): ");
            io::stdout().flush().unwrap();
            let mut k = String::new();
            io::stdin().read_line(&mut k).unwrap();
            Key::String(k.trim().to_string())
        }
        "2" => {
            print!("Enter key (integer): ");
            io::stdout().flush().unwrap();
            let mut k = String::new();
            io::stdin().read_line(&mut k).unwrap();
            let num: i64 = k.trim().parse().unwrap_or(0);
            Key::Int(num)
        }
        _ => {
            println!("Invalid key type");
            return Ok(());
        }
    };
    
    println!("Value type: 1=String, 2=Int");
    print!("Choice: ");
    io::stdout().flush().unwrap();
    
    let mut val_type = String::new();
    io::stdin().read_line(&mut val_type).unwrap();
    
    let value = match val_type.trim() {
        "1" => {
            print!("Enter value (string): ");
            io::stdout().flush().unwrap();
            let mut v = String::new();
            io::stdin().read_line(&mut v).unwrap();
            Value::String(v.trim().to_string())
        }
        "2" => {
            print!("Enter value (integer): ");
            io::stdout().flush().unwrap();
            let mut v = String::new();
            io::stdin().read_line(&mut v).unwrap();
            let num: i64 = v.trim().parse().unwrap_or(0);
            Value::Int(num)
        }
        _ => {
            println!("Invalid value type");
            return Ok(());
        }
    };
    
    store.put(key.clone(), value);
    println!("✓ Stored: {:?}", key);
    
    Ok(())
}

fn get_value(store: &Store) -> Result<(), StoreError> {
    println!("\n--- Get Value ---");
    println!("Key type: 1=String, 2=Int");
    print!("Choice: ");
    io::stdout().flush().unwrap();
    
    let mut key_type = String::new();
    io::stdin().read_line(&mut key_type).unwrap();
    
    let key = match key_type.trim() {
        "1" => {
            print!("Enter key (string): ");
            io::stdout().flush().unwrap();
            let mut k = String::new();
            io::stdin().read_line(&mut k).unwrap();
            Key::String(k.trim().to_string())
        }
        "2" => {
            print!("Enter key (integer): ");
            io::stdout().flush().unwrap();
            let mut k = String::new();
            io::stdin().read_line(&mut k).unwrap();
            let num: i64 = k.trim().parse().unwrap_or(0);
            Key::Int(num)
        }
        _ => {
            println!("Invalid key type");
            return Ok(());
        }
    };
    
    match store.get(&key) {
        Ok(value) => println!("✓ Value: {:?}", value),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    Ok(())
}

fn delete_value(store: &mut Store) -> Result<(), StoreError> {
    println!("\n--- Delete Value ---");
    println!("Key type: 1=String, 2=Int");
    print!("Choice: ");
    io::stdout().flush().unwrap();
    
    let mut key_type = String::new();
    io::stdin().read_line(&mut key_type).unwrap();
    
    let key = match key_type.trim() {
        "1" => {
            print!("Enter key (string): ");
            io::stdout().flush().unwrap();
            let mut k = String::new();
            io::stdin().read_line(&mut k).unwrap();
            Key::String(k.trim().to_string())
        }
        "2" => {
            print!("Enter key (integer): ");
            io::stdout().flush().unwrap();
            let mut k = String::new();
            io::stdin().read_line(&mut k).unwrap();
            let num: i64 = k.trim().parse().unwrap_or(0);
            Key::Int(num)
        }
        _ => {
            println!("Invalid key type");
            return Ok(());
        }
    };
    
    match store.delete(&key) {
        Ok(_) => println!("✓ Deleted: {:?}", key),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    Ok(())
}

fn display_all(store: &Store) -> Result<(), StoreError> {
    println!("\n--- All Entries ---");
    store.display_all()?;
    Ok(())
}

fn list_keys(store: &Store) {
    println!("\n--- All Keys ---");
    let keys: Vec<_> = store.keys().collect();
    if keys.is_empty() {
        println!("(no keys)");
    } else {
        for (i, key) in keys.iter().enumerate() {
            println!("{}. {:?}", i + 1, key);
        }
        println!("\nTotal: {} keys", keys.len());
    }
}

fn compact_store(store: &mut Store) -> Result<(), StoreError> {
    println!("\n--- Compacting Store ---");
    let bytes_reclaimed = store.compact()?;
    println!("✓ Compaction complete");
    println!("  Bytes reclaimed: {}", bytes_reclaimed);
    Ok(())
}

fn show_fragmentation(store: &Store) {
    println!("\n--- Fragmentation Info ---");
    let ratio = store.fragmentation_ratio();
    println!("Fragmentation ratio: {:.2}%", ratio * 100.0);
    
    if ratio > 0.35 {
        println!("⚠ High fragmentation - consider compacting");
    } else if ratio > 0.15 {
        println!("ℹ Moderate fragmentation");
    } else {
        println!("✓ Low fragmentation");
    }
}

fn save_store(store: &mut Store) -> Result<(), StoreError> {
    println!("\n--- Saving Store ---");
    store.save()?;
    println!("✓ Store saved successfully");
    Ok(())
}
