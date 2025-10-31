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
pub enum BorrowedEntry<'a>{
    Int(i64),
    String(&'a str),
}
fn serialize_key(key: &Key) -> Vec<u8> {
    match key {
        Key::String(s) => {
            let mut bytes = vec![0x01];
            let s_bytes = s.as_bytes();
            bytes.extend_from_slice(&(s_bytes.len() as u64).to_le_bytes());
            bytes.extend_from_slice(s_bytes);
            bytes
        }
        Key::Int(i) => {
            let mut bytes = vec![0x02];
            bytes.extend_from_slice(&i.to_le_bytes());
            bytes
        }
    }
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

fn serialize_entry(key: &Key, value: &Value) -> Vec<u8> {
    let key_bytes = serialize_key(key);
    let value_bytes = serialize_value(value);

    let total_len = (key_bytes.len() + value_bytes.len()) as u64;
    let mut bytes = Vec::with_capacity(8 + total_len as usize);

    bytes.extend_from_slice(&total_len.to_le_bytes());
    bytes.extend_from_slice(&key_bytes);
    bytes.extend_from_slice(&value_bytes);

    bytes
}

fn deserialize_item(bytes: &[u8]) -> Option<(BorrowedEntry, usize)> {
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
            Some((BorrowedEntry::String(s),9 + len))
        }
        0x02 => {
            if bytes.len() < 9 {
                return None;
            }
            let i = i64::from_le_bytes((&bytes[1..9]).try_into().unwrap());
            Some((BorrowedEntry::Int(i),9))
        }
        _ => None,
    }
}

fn deserialize_entry(bytes: &[u8]) -> Option<((BorrowedEntry, BorrowedEntry), usize)> {
    if bytes.len() < 8 {
        return None;
    }
    let total_len = u64::from_le_bytes((&bytes[0..8]).try_into().unwrap()) as usize;
    if bytes.len() < 8 + total_len {
        return None;
    }
    let mut offset = 8;
    let (key, key_len) = deserialize_item(&bytes[offset..])?;
    offset += key_len;
    let (value, value_len) = deserialize_item(&bytes[offset..])?;
    offset += value_len;
    Some(((key, value), offset))
}

fn borrowed_to_owned_key(be: &BorrowedEntry) -> Key {
    match be {
        BorrowedEntry::Int(i) => Key::Int(*i),
        BorrowedEntry::String(s) => Key::String(s.to_string()),
    }
}

fn borrowed_to_owned_value(be: &BorrowedEntry) -> Value {
    match be {
        BorrowedEntry::Int(i) => Value::Int(*i),
        BorrowedEntry::String(s) => Value::String(s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_entry() {
        let key = Key::String("hello".to_string());
        let value = Value::Int(2025);
        let serialized = serialize_entry(&key, &value);
        if let Some(((d_key, d_value), _)) = deserialize_entry(&serialized) {
            assert_eq!(d_key, BorrowedEntry::String("hello"));
            assert_eq!(d_value, BorrowedEntry::Int(2025));
        } else {
            panic!("Failed to deserialize whole entry");
        }
    }

    #[test]
    fn test_borrowed_to_owned_conversions() {
        // Borrowed string with Int
        let borrowed = BorrowedEntry::String("hello");
        let owned_key = borrowed_to_owned_key(&borrowed);
        let owned_value = borrowed_to_owned_value(&borrowed);

        assert_eq!(owned_key, Key::String("hello".to_string()));
        assert_eq!(owned_value, Value::String("hello".to_string()));

        // Borrowed int
        let borrowed_int = BorrowedEntry::Int(42);
        let owned_key2 = borrowed_to_owned_key(&borrowed_int);
        let owned_value2 = borrowed_to_owned_value(&borrowed_int);

        assert_eq!(owned_key2, Key::Int(42));
        assert_eq!(owned_value2, Value::Int(42));
    }
}

//[ total_len: u64 (8 bytes) ]
// [ key_type_tag: u8 (1 byte) ]
// [ key_data_len (if string): u64 (8 bytes), else none ]
// [ key_data (bytes) ]
// [ value_type_tag: u8 (1 byte) ]
// [ value_data_len (if string): u64 (8 bytes), else none ]
// [ value_data (bytes) ]