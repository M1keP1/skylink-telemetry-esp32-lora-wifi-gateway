use std::convert::TryInto;
use crate::types::{Value, BorrowedEntry};
use crate::error::DeserializationError;
use super::header::{RawHeader, serialize_header_unsafe, deserialize_header_unsafe, calculate_crc32};

pub(crate) fn serialize_value(value: &Value) -> Vec<u8> {
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

pub(crate) fn deserialize_value(bytes: &[u8]) -> Result<(BorrowedEntry, usize), DeserializationError> {
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
