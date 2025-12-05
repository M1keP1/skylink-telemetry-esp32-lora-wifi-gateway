mod store;
pub use store::Store;

use std::ptr;
use std::convert::TryInto;
use thiserror::Error;

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

#[repr(C, packed)]
struct RawHeader {
    length: u64,
    checksum: u32,
    tag: u8,
}

pub struct StoreIterator<'a> {
    store: &'a Store,
    keys_iter: std::collections::hash_map::Keys<'a, Key, usize>,
}

pub struct StoreIter<'a> {
    buf: &'a [u8],
    pos: usize,
}

#[derive(Debug, Error)]
enum DeserializationError {
    #[error("Buffer too short: expected {expected} bytes, got {actual}")]
    BufferTooShort { expected: usize, actual: usize },

    #[error("Invalid UTF-8 in string data")]
    InvalidUtf8(#[from] std::str::Utf8Error),

    #[error("Unknown tag value: 0x{0:02x}")]
    UnknownTag(u8),

    #[error("Checksum mismatch: expected 0x{expected:08x}, got 0x{actual:08x}")]
    ChecksumMismatch { expected: u32, actual: u32 },

    #[error("Failed to convert bytes")]
    ByteConversionError,
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Key not found: {0:?}")]
    KeyNotFound(Key),

    #[error("Data corruption detected")]
    DataCorruption {
        #[source]
        cause: DeserializationError,
    },

    #[error("Invalid data format")]
    InvalidData {
        #[source]
        cause: DeserializationError,
    },

    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("File corrupted: checksum mismatch")]
    FileCorrupted,

    #[error("Unsupported file version: {0}")]
    UnsupportedVersion(u32),
}

unsafe fn serialize_header_unsafe(header: &RawHeader, buffer: &mut Vec<u8>) {
    let header_size = size_of::<RawHeader>();
    let offset = buffer.len();
    buffer.reserve(header_size);
    unsafe {
        buffer.set_len(offset + header_size);
        let dest_ptr = buffer.as_mut_ptr().add(offset);
        let header_ptr = header as *const RawHeader;
        ptr::copy_nonoverlapping(header_ptr as *const u8, dest_ptr, header_size);
    }
}

unsafe fn deserialize_header_unsafe(bytes: &[u8]) -> Option<RawHeader> {
    let header_size = size_of::<RawHeader>();
    if bytes.len() < header_size {
        return None;
    }
    let header_ptr = bytes.as_ptr() as *const RawHeader;
    unsafe {
        Some(ptr::read_unaligned(header_ptr))
    }
}

pub(crate) fn calculate_crc32(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

fn serialize_value(value: &Value) -> Vec<u8> {
    let (tag, value_data) = match value {
        Value::String(s) => {
            let mut v = Vec::new();
            let b = s.as_bytes();
            v.extend_from_slice(&(b.len() as u64).to_le_bytes());
            v.extend_from_slice(b);
            (0x01u8, v)
        }
        Value::Int(i) => (0x02u8, i.to_le_bytes().to_vec()),
    };

    let checksum = calculate_crc32(&value_data);

    let header = RawHeader {
        length: value_data.len() as u64,
        checksum,
        tag,
    };

    let mut out = Vec::new();
    unsafe { serialize_header_unsafe(&header, &mut out) };
    out.extend_from_slice(&value_data);
    out
}

fn deserialize_value(bytes: &[u8]) -> Result<(BorrowedEntry, usize), DeserializationError> {
    let header_size = size_of::<RawHeader>();
    if bytes.len() < header_size {
        return Err(DeserializationError::BufferTooShort {
            expected: header_size,
            actual: bytes.len(),
        });
    }

    let header = unsafe {
        deserialize_header_unsafe(bytes)
            .ok_or(DeserializationError::BufferTooShort {
                expected: header_size,
                actual: bytes.len(),
            })?
    };

    let length = header.length as usize;
    if bytes.len() < header_size + length {
        return Err(DeserializationError::BufferTooShort {
            expected: header_size + length,
            actual: bytes.len(),
        });
    }

    let value_data = &bytes[header_size..header_size + length];

    let actual = calculate_crc32(value_data);
    if actual != header.checksum {
        return Err(DeserializationError::ChecksumMismatch {
            expected: header.checksum,
            actual,
        });
    }

    match header.tag {
        0x01 => {
            if value_data.len() < 8 {
                return Err(DeserializationError::BufferTooShort {
                    expected: 8,
                    actual: value_data.len(),
                });
            }
            let len = u64::from_le_bytes(
                value_data[0..8].try_into()
                    .map_err(|_| DeserializationError::ByteConversionError)?
            ) as usize;

            if value_data.len() < 8 + len {
                return Err(DeserializationError::BufferTooShort {
                    expected: 8 + len,
                    actual: value_data.len(),
                });
            }

            let s = std::str::from_utf8(&value_data[8..8 + len])?;

            Ok((BorrowedEntry::Text(s), header_size + length))
        }
        0x02 => {
            if value_data.len() < 8 {
                return Err(DeserializationError::BufferTooShort {
                    expected: 8,
                    actual: value_data.len(),
                });
            }
            let v = i64::from_le_bytes(
                value_data[0..8].try_into()
                    .map_err(|_| DeserializationError::ByteConversionError)?
            );
            Ok((BorrowedEntry::Int(v), header_size + length))
        }
        _ => Err(DeserializationError::UnknownTag(header.tag)),
    }
}

pub(crate) fn serialize_key(key: &Key) -> Vec<u8> {
    match key {
        Key::String(s) => {
            let mut out = Vec::new();
            out.push(0x01u8);
            let bytes = s.as_bytes();
            out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
            out.extend_from_slice(bytes);
            out
        }
        Key::Int(i) => {
            let mut out = Vec::new();
            out.push(0x02u8);
            out.extend_from_slice(&i.to_le_bytes());
            out
        }
    }
}

pub(crate) fn deserialize_key(bytes: &[u8]) -> Result<(Key, usize), DeserializationError> {
    if bytes.is_empty() {
        return Err(DeserializationError::BufferTooShort {
            expected: 1,
            actual: 0,
        });
    }

    let tag = bytes[0];
    match tag {
        0x01 => {
            if bytes.len() < 9 {
                return Err(DeserializationError::BufferTooShort {
                    expected: 9,
                    actual: bytes.len(),
                });
            }
            let len = u64::from_le_bytes(
                bytes[1..9].try_into()
                    .map_err(|_| DeserializationError::ByteConversionError)?
            ) as usize;

            if bytes.len() < 9 + len {
                return Err(DeserializationError::BufferTooShort {
                    expected: 9 + len,
                    actual: bytes.len(),
                });
            }

            let s = std::str::from_utf8(&bytes[9..9 + len])?;
            Ok((Key::String(s.to_string()), 9 + len))
        }
        0x02 => {
            if bytes.len() < 9 {
                return Err(DeserializationError::BufferTooShort {
                    expected: 9,
                    actual: bytes.len(),
                });
            }
            let i = i64::from_le_bytes(
                bytes[1..9].try_into()
                    .map_err(|_| DeserializationError::ByteConversionError)?
            );
            Ok((Key::Int(i), 9))
        }
        _ => Err(DeserializationError::UnknownTag(tag)),
    }
}

pub fn borrowed_to_owned(entry: &BorrowedEntry) -> OwnedEntry {
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

impl<'a> Iterator for StoreIterator<'a> {
    type Item = (&'a Key, Result<BorrowedEntry<'a>, StoreError>);

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.keys_iter.next()?;
        let value = self.store.get(&key);
        Some((key, value))
    }
}

impl<'a> Iterator for StoreIter<'a> {
    type Item = Result<BorrowedEntry<'a>, StoreError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.buf.len() {
            return None;
        }

        match deserialize_value(&self.buf[self.pos..]) {
            Ok((entry, bytes_read)) => {
                self.pos += bytes_read;
                Some(Ok(entry))
            }
            Err(e) => {
                self.pos = self.buf.len();
                let store_error = match e {
                    DeserializationError::ChecksumMismatch { .. } => {
                        StoreError::DataCorruption { cause: e }
                    }
                    _ => StoreError::InvalidData { cause: e }
                };
                Some(Err(store_error))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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