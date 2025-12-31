use crate::types::{Key, BorrowedEntry};
use crate::error::{StoreError, DeserializationError};
use crate::serialization::deserialize_value;
use crate::Store;

pub struct StoreIterator<'a> {
    pub(crate) store: &'a Store,
    pub(crate) keys_iter: std::collections::hash_map::Keys<'a, Key, usize>,
}

pub struct StoreIter<'a> {
    pub(crate) buf: &'a [u8],
    pub(crate) pos: usize,
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
