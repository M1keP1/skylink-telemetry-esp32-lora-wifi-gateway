pub(crate) use crate::{borrowed_to_owned, deserialize_value, serialize_value, BorrowedEntry, Key, OwnedEntry, StoreIterator, Value};
use std::collections::HashMap;
use crate::StoreIter;
use anyhow::{Result, Context, bail};

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

    pub fn get<'a>(&'a self, key: &Key) -> Result<BorrowedEntry<'a>> {
        let pos = *self.index.get(key)
            .ok_or_else(|| anyhow::anyhow!("Key not found: {:?}", key))?;

        if pos >= self.data.len() {
            bail!("Invalid offset {} for data buffer of size {}", pos, self.data.len());
        }

        let (entry, _) = deserialize_value(&self.data[pos..])
            .context(format!("Failed to deserialize value at offset {}", pos))?;

        Ok(entry)
    }

    pub fn display_all(&self) -> Result<()> {
        println!("=== Store Contents ===");
        let mut count = 0;

        for (key, value_result) in self.iter() {
            match value_result {
                Ok(value) => {
                    println!("{:?} -> {:?}", key, value);
                    count += 1;
                }
                Err(e) => {
                    println!("Error reading {:?}: {}", key, e);
                }
            }
        }

        println!("=== Total: {} entries ===", count);
        Ok(())
    }

    pub fn iter(&self) -> StoreIterator {
        StoreIterator {
            store: self,
            keys_iter: self.index.keys(),
        }
    }

    pub fn buffer_iter(&self) -> StoreIter {
        StoreIter {
            buf: &self.data,
            pos: 0,
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.index.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = Result<BorrowedEntry>> {
        self.iter().map(|(_, value)| value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_entries() -> Result<()> {
        let mut store = Store::new();

        store.put(Key::String("k1".into()), Value::Int(1));
        store.put(Key::Int(2), Value::String("v2".into()));
        store.put(Key::String("k3".into()), Value::String("v3".into()));

        assert_eq!(store.get(&Key::String("k1".into()))?, BorrowedEntry::Int(1));
        assert_eq!(store.get(&Key::Int(2))?, BorrowedEntry::Text("v2"));
        assert_eq!(store.get(&Key::String("k3".into()))?, BorrowedEntry::Text("v3"));

        // Test key not found error
        let result = store.get(&Key::Int(999));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Key not found"));

        Ok(())
    }

    #[test]
    fn test_overwrite_behavior() -> Result<()> {
        let mut store = Store::new();
        store.put(Key::Int(1), Value::Int(10));
        assert_eq!(store.get(&Key::Int(1))?, BorrowedEntry::Int(10));

        store.put(Key::Int(1), Value::Int(20));
        assert_eq!(store.get(&Key::Int(1))?, BorrowedEntry::Int(20));

        Ok(())
    }

    #[test]
    fn test_borrowed_lifetime() -> Result<()> {
        let mut store = Store::new();
        store.put(Key::String("t".into()), Value::String("abc".into()));

        let b = store.get(&Key::String("t".into()))?;
        if let BorrowedEntry::Text(s) = b {
            assert_eq!(s, "abc");
            assert_eq!(s.len(), 3);
        } else {
            panic!("expected borrowed text");
        }

        Ok(())
    }

    #[test]
    fn test_borrowed_to_owned_roundtrip() -> Result<()> {
        let mut store = Store::new();
        store.put(Key::String("c".into()), Value::String("hello".into()));

        let b = store.get(&Key::String("c".into()))?;
        let owned = borrowed_to_owned(&b);
        assert_eq!(owned, OwnedEntry::Text("hello".into()));

        Ok(())
    }
}