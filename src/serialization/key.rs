use std::convert::TryInto;
use crate::types::Key;
use crate::error::DeserializationError;

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
