use thiserror::Error;
use crate::types::Key;

#[derive(Debug, Error)]
pub enum DeserializationError {
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
