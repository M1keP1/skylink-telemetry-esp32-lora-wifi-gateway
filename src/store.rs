use crate::{borrowed_to_owned, deserialize_value, serialize_value, BorrowedEntry, Key, OwnedEntry, Value};
use std::collections::HashMap;

pub struct Store {
    index: HashMap<Key, usize>,
    data: Vec<u8>,
}

impl Store {
    pub fn new() -> Store {
        Store {
            index: HashMap::new(),
            data: Vec::new(),
        }
    }

    pub fn put(&mut self, key: Key, value: Value) {
        let position = self.data.len();
        let serialized = serialize_value(&value);
        self.data.extend_from_slice(&serialized);
        self.index.insert(key, position);
    }

    pub fn get<'a>(&'a self, search_key: &Key) -> Option<BorrowedEntry<'a>> {
        let position = self.index.get(search_key)?;

        if *position >= self.data.len() {
            return None;
        }

        let (value, _) = deserialize_value(&self.data[*position..])?;
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overwrites() {
        let mut store = Store::new();

        store.put(Key::String("key1".into()), Value::Int(100));
        assert_eq!(store.get(&Key::String("key1".into())), Some(BorrowedEntry::Int(100)));

        // Overwrite with new value
        store.put(Key::String("key1".into()), Value::Int(200));
        assert_eq!(store.get(&Key::String("key1".into())), Some(BorrowedEntry::Int(200)));
    }

    #[test]
    fn test_borrowed_entry_returns() {
        let mut store = Store::new();

        store.put(Key::String("text_key".into()), Value::String("borrowed_text".into()));
        store.put(Key::Int(42), Value::Int(12345));

        // Verify that get returns BorrowedEntry
        if let Some(borrowed) = store.get(&Key::String("text_key".into())) {
            match borrowed {
                BorrowedEntry::Text(s) => assert_eq!(s, "borrowed_text"),
                _ => panic!("Expected Text variant"),
            }
        } else {
            panic!("Expected to find key");
        }

        if let Some(borrowed) = store.get(&Key::Int(42)) {
            match borrowed {
                BorrowedEntry::Int(i) => assert_eq!(i, 12345),
                _ => panic!("Expected Int variant"),
            }
        } else {
            panic!("Expected to find key");
        }
    }

    #[test]
    fn test_multiple_conversions() {
        let mut store = Store::new();

        store.put(Key::Int(1), Value::String("first".into()));
        store.put(Key::Int(2), Value::String("second".into()));
        store.put(Key::Int(3), Value::Int(999));

        // Convert all to owned
        let owned1 = borrowed_to_owned(&store.get(&Key::Int(1)).unwrap());
        let owned2 = borrowed_to_owned(&store.get(&Key::Int(2)).unwrap());
        let owned3 = borrowed_to_owned(&store.get(&Key::Int(3)).unwrap());

        assert_eq!(owned1, OwnedEntry::Text("first".to_string()));
        assert_eq!(owned2, OwnedEntry::Text("second".to_string()));
        assert_eq!(owned3, OwnedEntry::Int(999));
    }
}