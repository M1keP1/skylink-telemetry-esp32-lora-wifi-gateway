pub(crate) use crate::{borrowed_to_owned, deserialize_value, serialize_value, BorrowedEntry, Key, OwnedEntry, StoreIterator, Value, DeserializationError, serialize_key, deserialize_key, calculate_crc32};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{self};
use std::io::{Write, Read};
use crate::{StoreIter, StoreError};

const FILE_VERSION: u32 = 1;

pub struct Store {
    index: HashMap<Key, usize>,
    data: Vec<u8>,
    path: Option<PathBuf>,
}

impl Store {
    pub fn new() -> Store {
        Store {
            index: HashMap::new(),
            data: Vec::new(),
            path: None,
        }
    }

    pub fn put(&mut self, key: Key, value: Value) {
        let pos = self.data.len();
        let serialized = serialize_value(&value);
        self.data.extend_from_slice(&serialized);
        self.index.insert(key, pos);
    }

    pub fn get<'a>(&'a self, key: &Key) -> Result<BorrowedEntry<'a>, StoreError> {
        let pos = *self.index.get(key)
            .ok_or_else(|| StoreError::KeyNotFound(key.clone()))?;

        if pos >= self.data.len() {
            return Err(StoreError::InvalidData {
                cause: DeserializationError::BufferTooShort {
                    expected: pos + 1,
                    actual: self.data.len(),
                },
            });
        }

        let (entry, _) = deserialize_value(&self.data[pos..])
            .map_err(|cause| {
                match cause {
                    DeserializationError::ChecksumMismatch { .. } => {
                        StoreError::DataCorruption { cause }
                    }
                    _ => StoreError::InvalidData { cause }
                }
            })?;

        Ok(entry)
    }

    pub fn display_all(&self) -> Result<(), StoreError> {
        println!("=== Store Contents ===");
        let mut count = 0;

        for (key, value_result) in self.iter() {
            match value_result {
                Ok(value) => {
                    println!("{:?} -> {:?}", key, value);
                    count += 1;
                }
                Err(e) => {
                    println!("Error reading {:?}: {}", key, e);
                }
            }
        }

        println!("=== Total: {} entries ===", count);
        Ok(())
    }

    pub fn iter(&self) -> StoreIterator {
        StoreIterator {
            store: self,
            keys_iter: self.index.keys(),
        }
    }

    pub fn buffer_iter(&self) -> StoreIter {
        StoreIter {
            buf: &self.data,
            pos: 0,
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.index.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = Result<BorrowedEntry, StoreError>> {
        self.iter().map(|(_, value)| value)
    }

    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Store, StoreError> {
        let path_buf = path.as_ref().to_path_buf();

        if Self::files_exist(&path_buf) {
            Self::load(&path_buf)
        } else {
            Ok(Store {
                index: HashMap::new(),
                data: Vec::new(),
                path: Some(path_buf),
            })
        }
    }

    pub fn save(&self) -> Result<(), StoreError> {
        let base_path = self.path.as_ref()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::Other,
                "No path set for store"
            ))?;

        let keys_path = Self::keys_path(base_path);
        let data_path = Self::data_path(base_path);
        let meta_path = Self::meta_path(base_path);

        let mut keys_buf = Vec::new();
        for (key, offset) in &self.index {
            let key_bytes = serialize_key(key);
            keys_buf.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
            keys_buf.extend_from_slice(&key_bytes);
            keys_buf.extend_from_slice(&(*offset as u64).to_le_bytes());
        }

        let keys_checksum = calculate_crc32(&keys_buf);
        let data_checksum = calculate_crc32(&self.data);

        let mut meta_buf = Vec::new();
        meta_buf.extend_from_slice(&FILE_VERSION.to_le_bytes());
        meta_buf.extend_from_slice(&keys_checksum.to_le_bytes());
        meta_buf.extend_from_slice(&data_checksum.to_le_bytes());
        meta_buf.extend_from_slice(&(self.index.len() as u64).to_le_bytes());

        fs::write(&meta_path, &meta_buf)?;
        fs::write(&keys_path, &keys_buf)?;
        fs::write(&data_path, &self.data)?;

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Store, StoreError> {
        let base_path = path.as_ref();
        let keys_path = Self::keys_path(base_path);
        let data_path = Self::data_path(base_path);
        let meta_path = Self::meta_path(base_path);

        let meta_buf = fs::read(&meta_path)?;
        if meta_buf.len() < 20 {
            return Err(StoreError::InvalidData {
                cause: DeserializationError::BufferTooShort {
                    expected: 20,
                    actual: meta_buf.len(),
                },
            });
        }

        let version = u32::from_le_bytes(meta_buf[0..4].try_into().unwrap());
        if version != FILE_VERSION {
            return Err(StoreError::UnsupportedVersion(version));
        }

        let stored_keys_checksum = u32::from_le_bytes(meta_buf[4..8].try_into().unwrap());
        let stored_data_checksum = u32::from_le_bytes(meta_buf[8..12].try_into().unwrap());
        let entry_count = u64::from_le_bytes(meta_buf[12..20].try_into().unwrap());

        let keys_buf = fs::read(&keys_path)?;
        let data_buf = fs::read(&data_path)?;

        let actual_keys_checksum = calculate_crc32(&keys_buf);
        if actual_keys_checksum != stored_keys_checksum {
            return Err(StoreError::FileCorrupted);
        }

        let actual_data_checksum = calculate_crc32(&data_buf);
        if actual_data_checksum != stored_data_checksum {
            return Err(StoreError::FileCorrupted);
        }

        let mut index = HashMap::new();
        let mut pos = 0;

        while pos < keys_buf.len() {
            if pos + 4 > keys_buf.len() {
                break;
            }

            let key_len = u32::from_le_bytes(keys_buf[pos..pos+4].try_into().unwrap()) as usize;
            pos += 4;

            if pos + key_len + 8 > keys_buf.len() {
                return Err(StoreError::InvalidData {
                    cause: DeserializationError::BufferTooShort {
                        expected: pos + key_len + 8,
                        actual: keys_buf.len(),
                    },
                });
            }

            let (key, _) = deserialize_key(&keys_buf[pos..pos+key_len])
                .map_err(|cause| StoreError::InvalidData { cause })?;
            pos += key_len;

            let offset = u64::from_le_bytes(keys_buf[pos..pos+8].try_into().unwrap()) as usize;
            pos += 8;

            index.insert(key, offset);
        }

        if index.len() != entry_count as usize {
            return Err(StoreError::FileCorrupted);
        }

        Ok(Store {
            index,
            data: data_buf,
            path: Some(base_path.to_path_buf()),
        })
    }

    fn files_exist(base_path: &Path) -> bool {
        let keys_path = Self::keys_path(base_path);
        let data_path = Self::data_path(base_path);
        let meta_path = Self::meta_path(base_path);
        keys_path.exists() && data_path.exists() && meta_path.exists()
    }

    fn keys_path(base_path: &Path) -> PathBuf {
        let mut p = base_path.to_path_buf();
        p.set_extension("keys");
        p
    }

    fn data_path(base_path: &Path) -> PathBuf {
        let mut p = base_path.to_path_buf();
        p.set_extension("data");
        p
    }

    fn meta_path(base_path: &Path) -> PathBuf {
        let mut p = base_path.to_path_buf();
        p.set_extension("meta");
        p
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        if self.path.is_some() {
            let _ = self.save();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_multiple_entries() -> Result<(), StoreError> {
        let mut store = Store::new();

        store.put(Key::String("k1".into()), Value::Int(1));
        store.put(Key::Int(2), Value::String("v2".into()));
        store.put(Key::String("k3".into()), Value::String("v3".into()));

        assert_eq!(store.get(&Key::String("k1".into()))?, BorrowedEntry::Int(1));
        assert_eq!(store.get(&Key::Int(2))?, BorrowedEntry::Text("v2"));
        assert_eq!(store.get(&Key::String("k3".into()))?, BorrowedEntry::Text("v3"));

        let result = store.get(&Key::Int(999));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StoreError::KeyNotFound(_)));

        Ok(())
    }

    #[test]
    fn test_overwrite_behavior() -> Result<(), StoreError> {
        let mut store = Store::new();
        store.put(Key::Int(1), Value::Int(10));
        assert_eq!(store.get(&Key::Int(1))?, BorrowedEntry::Int(10));

        store.put(Key::Int(1), Value::Int(20));
        assert_eq!(store.get(&Key::Int(1))?, BorrowedEntry::Int(20));

        Ok(())
    }

    #[test]
    fn test_borrowed_lifetime() -> Result<(), StoreError> {
        let mut store = Store::new();
        store.put(Key::String("t".into()), Value::String("abc".into()));

        let b = store.get(&Key::String("t".into()))?;
        if let BorrowedEntry::Text(s) = b {
            assert_eq!(s, "abc");
            assert_eq!(s.len(), 3);
        } else {
            panic!("expected borrowed text");
        }

        Ok(())
    }

    #[test]
    fn test_borrowed_to_owned_roundtrip() -> Result<(), StoreError> {
        let mut store = Store::new();
        store.put(Key::String("c".into()), Value::String("hello".into()));

        let b = store.get(&Key::String("c".into()))?;
        let owned = borrowed_to_owned(&b);
        assert_eq!(owned, OwnedEntry::Text("hello".into()));

        Ok(())
    }

    #[test]
    fn test_save_load_roundtrip() -> Result<(), StoreError> {
        let temp_path = "/tmp/test_store_roundtrip";

        {
            let mut store = Store::with_path(temp_path)?;
            store.put(Key::String("key1".into()), Value::Int(42));
            store.put(Key::Int(100), Value::String("test".into()));
            store.save()?;
        }

        let loaded_store = Store::load(temp_path)?;
        assert_eq!(loaded_store.get(&Key::String("key1".into()))?, BorrowedEntry::Int(42));
        assert_eq!(loaded_store.get(&Key::Int(100))?, BorrowedEntry::Text("test"));

        fs::remove_file(format!("{}.keys", temp_path)).ok();
        fs::remove_file(format!("{}.data", temp_path)).ok();
        fs::remove_file(format!("{}.meta", temp_path)).ok();

        Ok(())
    }

    #[test]
    fn test_auto_load_on_with_path() -> Result<(), StoreError> {
        let temp_path = "/tmp/test_store_autoload";

        {
            let mut store = Store::with_path(temp_path)?;
            store.put(Key::String("auto".into()), Value::Int(123));
            store.save()?;
        }

        let reloaded = Store::with_path(temp_path)?;
        assert_eq!(reloaded.get(&Key::String("auto".into()))?, BorrowedEntry::Int(123));

        fs::remove_file(format!("{}.keys", temp_path)).ok();
        fs::remove_file(format!("{}.data", temp_path)).ok();
        fs::remove_file(format!("{}.meta", temp_path)).ok();

        Ok(())
    }

    #[test]
    fn test_auto_save_on_drop() -> Result<(), StoreError> {
        let temp_path = "/tmp/test_store_drop";

        {
            let mut store = Store::with_path(temp_path)?;
            store.put(Key::String("drop_test".into()), Value::Int(777));
        }

        let reloaded = Store::load(temp_path)?;
        assert_eq!(reloaded.get(&Key::String("drop_test".into()))?, BorrowedEntry::Int(777));

        fs::remove_file(format!("{}.keys", temp_path)).ok();
        fs::remove_file(format!("{}.data", temp_path)).ok();
        fs::remove_file(format!("{}.meta", temp_path)).ok();

        Ok(())
    }
}