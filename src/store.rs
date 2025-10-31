use crate::{deserialize_entry, serialize_entry, BorrowedEntry, Key, Value};

pub struct Store {
    data: Vec<u8>,
}

impl Store {
    pub fn new() -> Store {
        Store { data: Vec::new() }
    }
    pub fn put(&mut self, key: Key, value: Value) {
        let serialized = serialize_entry(&key, &value);
        self.data.extend_from_slice(&serialized);
    }

    pub fn get<'a>(&'a self, search_key: &BorrowedEntry) -> Option<BorrowedEntry<'a>> {
        let mut offset = 0;
        while offset < self.data.len() {
            if self.data.len() < offset + 8 {
                break;
            }
            let total_len = u64::from_le_bytes(
                self.data[offset..offset + 8].try_into().unwrap(),
            ) as usize;

            if self.data.len() < offset + 8 + total_len {
                break;
            }

            if let Some(((key, value), _)) = deserialize_entry(&self.data[offset..]) {
                if &key == search_key {
                    return Some(value);
                }
            }
            offset += 8 + total_len;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_entries() {
    let mut store = Store::new();

    store.put(Key::String("key1".into()), Value::Int(1));
    store.put(Key::Int(2), Value::String("value2".into()));
    store.put(Key::String("key3".into()), Value::String("value3".into()));

    assert_eq!(store.get(&BorrowedEntry::String("key1")), Some(BorrowedEntry::Int(1)));
    assert_eq!(store.get(&BorrowedEntry::Int(2)), Some(BorrowedEntry::String("value2")));
    assert_eq!(store.get(&BorrowedEntry::String("key3")), Some(BorrowedEntry::String("value3")));
    assert_eq!(store.get(&BorrowedEntry::Int(100)), None);

    }
}