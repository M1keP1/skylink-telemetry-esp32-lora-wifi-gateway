// Module declarations
mod types;
mod error;
mod serialization;
mod iterator;
mod store;

// Public API re-exports
pub use types::{Key, Value, BorrowedEntry, OwnedEntry, borrowed_to_owned, owned_to_value};
pub use error::StoreError;
pub use store::Store;
pub use iterator::{StoreIterator, StoreIter};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{serialize_value, deserialize_value};
    use crate::error::DeserializationError;
    use crate::serialization::RawHeader;

    #[test]
    fn test_roundtrip_values() -> Result<(), DeserializationError> {
        let v = Value::Int(2025);
        let s = serialize_value(&v);
        let (out, _) = deserialize_value(&s)?;
        assert_eq!(out, BorrowedEntry::Int(2025));

        let v2 = Value::String("hello".into());
        let s2 = serialize_value(&v2);
        let (out2, _) = deserialize_value(&s2)?;
        assert_eq!(out2, BorrowedEntry::Text("hello"));

        Ok(())
    }

    #[test]
    fn test_checksum_catches_corruption() {
        let v = Value::String("abcdef".into());
        let mut s = serialize_value(&v);
        let header_size = size_of::<RawHeader>();
        s[header_size] ^= 0xFF;

        let result = deserialize_value(&s);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, DeserializationError::ChecksumMismatch { .. }));
    }

    #[test]
    fn test_borrowed_owned() {
        let b = BorrowedEntry::Text("hi");
        let o = borrowed_to_owned(&b);
        assert_eq!(o, OwnedEntry::Text("hi".into()));
        let b2 = BorrowedEntry::Int(7);
        let o2 = borrowed_to_owned(&b2);
        assert_eq!(o2, OwnedEntry::Int(7));
    }

    #[test]
    fn test_owned_to_value() {
        let o = OwnedEntry::Text("x".into());
        assert_eq!(owned_to_value(&o), Value::String("x".into()));
        let o2 = OwnedEntry::Int(5);
        assert_eq!(owned_to_value(&o2), Value::Int(5));
    }

    #[test]
    fn test_store_iterator() -> Result<(), StoreError> {
        let mut store = Store::new();

        store.put(Key::String("k1".into()), Value::Int(1));
        store.put(Key::Int(2), Value::String("v2".into()));
        store.put(Key::String("k3".into()), Value::String("v3".into()));

        let entries: Vec<_> = store.iter().collect();

        assert_eq!(entries.len(), 3);

        let mut found_items = 0;
        for (key, value_result) in entries {
            let value = value_result?;
            match (key, value) {
                (Key::String(s), BorrowedEntry::Int(1)) if s == "k1" => found_items += 1,
                (Key::Int(2), BorrowedEntry::Text("v2")) => found_items += 1,
                (Key::String(s), BorrowedEntry::Text("v3")) if s == "k3" => found_items += 1,
                _ => {}
            }
        }
        assert_eq!(found_items, 3);

        Ok(())
    }

    #[test]
    fn test_keys_iterator() {
        let mut store = Store::new();
        store.put(Key::String("a".into()), Value::Int(1));
        store.put(Key::Int(42), Value::String("test".into()));

        let keys: Vec<_> = store.keys().collect();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_values_iterator() -> Result<(), StoreError> {
        let mut store = Store::new();
        store.put(Key::String("a".into()), Value::Int(1));
        store.put(Key::String("b".into()), Value::String("hello".into()));

        let values: Result<Vec<_>, _> = store.values().collect();
        let values = values?;
        assert_eq!(values.len(), 2);

        Ok(())
    }

    #[test]
    fn test_buffer_iterator_preserves_order() -> Result<(), StoreError> {
        let mut store = Store::new();

        store.put(Key::String("first".into()), Value::Int(1));
        store.put(Key::String("second".into()), Value::Int(2));
        store.put(Key::String("third".into()), Value::Int(3));

        let values: Result<Vec<_>, _> = store.buffer_iter().collect();
        let values = values?;

        assert_eq!(values, vec![
            BorrowedEntry::Int(1),
            BorrowedEntry::Int(2),
            BorrowedEntry::Int(3),
        ]);

        Ok(())
    }
}