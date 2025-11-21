mod store;
pub use store::Store;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Key {
    String(String),
    Int(i64),
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Value {
    String(String),
    Int(i64),
}

#[derive(Debug, PartialEq, Eq)]
pub enum BorrowedEntry<'a> {
    Int(i64),
    Text(&'a str),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OwnedEntry {
    Int(i64),
    Text(String),
}

fn serialize_value(value: &Value) -> Vec<u8> {
    match value {
        Value::String(s) => {
            let mut bytes = vec![0x01];
            let s_bytes = s.as_bytes();
            bytes.extend_from_slice(&(s_bytes.len() as u64).to_le_bytes());
            bytes.extend_from_slice(s_bytes);
            bytes
        }
        Value::Int(i) => {
            let mut bytes = vec![0x02];
            bytes.extend_from_slice(&i.to_le_bytes());
            bytes
        }
    }
}

fn deserialize_value(bytes: &[u8]) -> Option<(BorrowedEntry, usize)> {
    if bytes.is_empty() {
        return None;
    }
    match bytes[0] {
        0x01 => {
            if bytes.len() < 9 {
                return None;
            }
            let len = u64::from_le_bytes((&bytes[1..9]).try_into().unwrap()) as usize;
            if bytes.len() < 9 + len {
                return None;
            }
            let s = std::str::from_utf8(&bytes[9..9 + len]).ok()?;
            Some((BorrowedEntry::Text(s), 9 + len))
        }
        0x02 => {
            if bytes.len() < 9 {
                return None;
            }
            let i = i64::from_le_bytes((&bytes[1..9]).try_into().unwrap());
            Some((BorrowedEntry::Int(i), 9))
        }
        _ => None,
    }
}

fn borrowed_to_owned(entry: &BorrowedEntry) -> OwnedEntry {
    match entry {
        BorrowedEntry::Int(i) => OwnedEntry::Int(*i),
        BorrowedEntry::Text(s) => OwnedEntry::Text(s.to_string()),
    }
}

fn owned_to_value(entry: &OwnedEntry) -> Value {
    match entry {
        OwnedEntry::Int(i) => Value::Int(*i),
        OwnedEntry::Text(s) => Value::String(s.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_value() {
        let value = Value::Int(2025);
        let serialized = serialize_value(&value);
        if let Some((d_value, _)) = deserialize_value(&serialized) {
            assert_eq!(d_value, BorrowedEntry::Int(2025));
        } else {
            panic!("Failed to deserialize value");
        }

        let value2 = Value::String("hello".to_string());
        let serialized2 = serialize_value(&value2);
        if let Some((d_value2, _)) = deserialize_value(&serialized2) {
            assert_eq!(d_value2, BorrowedEntry::Text("hello"));
        } else {
            panic!("Failed to deserialize value");
        }
    }

    #[test]
    fn test_borrowed_to_owned_conversions() {
        // Test Text conversion
        let borrowed_text = BorrowedEntry::Text("hello");
        let owned = borrowed_to_owned(&borrowed_text);
        assert_eq!(owned, OwnedEntry::Text("hello".to_string()));

        // Test Int conversion
        let borrowed_int = BorrowedEntry::Int(42);
        let owned_int = borrowed_to_owned(&borrowed_int);
        assert_eq!(owned_int, OwnedEntry::Int(42));
    }

    #[test]
    fn test_owned_to_value_conversions() {
        let owned_text = OwnedEntry::Text("world".to_string());
        let value = owned_to_value(&owned_text);
        assert_eq!(value, Value::String("world".to_string()));

        let owned_int = OwnedEntry::Int(100);
        let value_int = owned_to_value(&owned_int);
        assert_eq!(value_int, Value::Int(100));
    }

}

// Storage format (in data Vec<u8>):
// [ value_type_tag: u8 (1 byte) ]
// [ value_data_len (if string): u64 (8 bytes), else none ]
// [ value_data (bytes) ]