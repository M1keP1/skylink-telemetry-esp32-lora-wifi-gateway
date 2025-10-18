use kiwi_store::{Key, Store, Value};
fn main() {
    let mut store = Store::new();
    println!("Putting Key:5 and Value:6");
    store.put(Key::Int(5), Value::Int(6));
    println!("Checking if Key:2 exists -> {:?}", store.contains_key(&Key::Int(2)));
    println!("Checking if Key:5 exists -> {:?}", store.contains_key(&Key::Int(5)));
    println!("Getting Key:5 -> {:?}", store.get(&Key::Int(5)));
}