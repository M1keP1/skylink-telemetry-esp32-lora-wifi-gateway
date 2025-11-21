mod store;
pub use store::Store;

use std::ptr;
use std::convert::TryInto;

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

unsafe fn serialize_header_unsafe(header: &RawHeader, buffer: &mut Vec<u8>) {
    // SAFETY: repr(C, packed) guarantees layout, we allocate enough space and immediately write
    let header_size = std::mem::size_of::<RawHeader>();
    let offset = buffer.len();
    buffer.reserve(header_size);
    buffer.set_len(offset + header_size);
    let dest_ptr = buffer.as_mut_ptr().add(offset);
    let header_ptr = header as *const RawHeader;
    ptr::copy_nonoverlapping(header_ptr as *const u8, dest_ptr, header_size);
}

unsafe fn deserialize_header_unsafe(bytes: &[u8]) -> Option<RawHeader> {
    // SAFETY: read_unaligned is used because data may not be aligned
    let header_size = std::mem::size_of::<RawHeader>();
    if bytes.len() < header_size {
        return None;
    }
    let header_ptr = bytes.as_ptr() as *const RawHeader;
    Some(ptr::read_unaligned(header_ptr))
}

fn calculate_crc32(data: &[u8]) -> u32 {
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

fn deserialize_value(bytes: &[u8]) -> Option<(BorrowedEntry, usize)> {
    let header_size = std::mem::size_of::<RawHeader>();
    if bytes.len() < header_size {
        return None;
    }

    let header = unsafe { deserialize_header_unsafe(bytes)? };

    let length = header.length as usize;
    if bytes.len() < header_size + length {
        return None;
    }

    let value_data = &bytes[header_size..header_size + length];

    let actual = calculate_crc32(value_data);
    if actual != header.checksum {
        panic!("Checksum mismatch! Data corruption detected.");
    }

    match header.tag {
        0x01 => {
            if value_data.len() < 8 {
                return None;
            }
            let len = u64::from_le_bytes(value_data[0..8].try_into().unwrap()) as usize;
            if value_data.len() < 8 + len {
                return None;
            }
            let s = std::str::from_utf8(&value_data[8..8 + len]).ok()?;
            Some((BorrowedEntry::Text(s), header_size + length))
        }
        0x02 => {
            if value_data.len() < 8 {
                return None;
            }
            let v = i64::from_le_bytes(value_data[0..8].try_into().unwrap());
            Some((BorrowedEntry::Int(v), header_size + length))
        }
        _ => None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_values() {
        let v = Value::Int(2025);
        let s = serialize_value(&v);
        let (out, _) = deserialize_value(&s).unwrap();
        assert_eq!(out, BorrowedEntry::Int(2025));

        let v2 = Value::String("hello".into());
        let s2 = serialize_value(&v2);
        let (out2, _) = deserialize_value(&s2).unwrap();
        assert_eq!(out2, BorrowedEntry::Text("hello"));
    }

    #[test]
    #[should_panic(expected = "Checksum mismatch")]
    fn test_checksum_catches_corruption() {
        let v = Value::String("abcdef".into());
        let mut s = serialize_value(&v);
        let header_size = std::mem::size_of::<RawHeader>();
        s[header_size] ^= 0xFF;
        deserialize_value(&s);
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
}

// Vec<u8> layout:
// [ RawHeader(length u64 | checksum u32 | tag u8) ]
// [ value_data ]
//
// value_data for String = [ u64 len ][ bytes ]
// value_data for Int    = [ i64 bytes ]
