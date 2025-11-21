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
        let pos = self.data.len();
        let serialized = serialize_value(&value);
        self.data.extend_from_slice(&serialized);
        self.index.insert(key, pos);
    }

    pub fn get<'a>(&'a self, key: &Key) -> Option<BorrowedEntry<'a>> {
        let pos = *self.index.get(key)?;
        if pos >= self.data.len() {
            return None;
        }
        let (entry, _) = deserialize_value(&self.data[pos..])?;
        Some(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_entries() {
        let mut store = Store::new();

        store.put(Key::String("k1".into()), Value::Int(1));
        store.put(Key::Int(2), Value::String("v2".into()));
        store.put(Key::String("k3".into()), Value::String("v3".into()));

        assert_eq!(store.get(&Key::String("k1".into())), Some(BorrowedEntry::Int(1)));
        assert_eq!(store.get(&Key::Int(2)), Some(BorrowedEntry::Text("v2")));
        assert_eq!(store.get(&Key::String("k3".into())), Some(BorrowedEntry::Text("v3")));
        assert_eq!(store.get(&Key::Int(999)), None);
    }

    #[test]
    fn test_overwrite_behavior() {
        let mut store = Store::new();
        store.put(Key::Int(1), Value::Int(10));
        assert_eq!(store.get(&Key::Int(1)), Some(BorrowedEntry::Int(10)));
        store.put(Key::Int(1), Value::Int(20));
        assert_eq!(store.get(&Key::Int(1)), Some(BorrowedEntry::Int(20)));
    }

    #[test]
    fn test_borrowed_lifetime() {
        let mut store = Store::new();
        store.put(Key::String("t".into()), Value::String("abc".into()));
        let b = store.get(&Key::String("t".into()));
        if let Some(BorrowedEntry::Text(s)) = b {
            assert_eq!(s, "abc");
            assert_eq!(s.len(), 3);
        } else {
            panic!("expected borrowed text");
        }
    }

    #[test]
    fn test_borrowed_to_owned_roundtrip() {
        let mut store = Store::new();
        store.put(Key::String("c".into()), Value::String("hello".into()));
        let b = store.get(&Key::String("c".into())).unwrap();
        let owned = borrowed_to_owned(&b);
        assert_eq!(owned, OwnedEntry::Text("hello".into()));
    }
}
