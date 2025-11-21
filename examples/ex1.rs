use kiwi_store::{Key, Store, Value, BorrowedEntry};

fn main() {
    let mut store = Store::new();

    println!("Putting Key: 5 and Value: 6");
    store.put(Key::Int(5), Value::Int(6));

    // Getting the value for Key: 5
    let key_to_get = Key::Int(5);
    match store.get(&key_to_get) {
        Some(value) => println!("Got value for key 5: {:?}", value),
        None => println!("Key 5 not found"),
    }

    // Trying to get a non-existent key 2
    let missing_key = Key::Int(2);
    println!("Getting Key: 2 -> {:?}", store.get(&missing_key));
    
    store.display_all();
}