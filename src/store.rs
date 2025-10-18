use std::collections::HashMap;
use crate::{Key, Value};

pub struct Store {
    map: HashMap<Key, Value>,
}

impl Store {
    pub fn new() -> Store {
        Store { map: HashMap::new() }
    }
    pub fn put(&mut self, key: Key, value: Value) {
        self.map.insert(key, value);
    }
    pub fn get(&self, key: &Key) -> Option<&Value> {
        self.map.get(key)
    }
    pub fn delete(&mut self, key: &Key) -> Option<Value> {
        self.map.remove(key)
    }
    pub fn update(&mut self, key: Key, value: Value) -> Option<Value> {
        self.map.insert(key, value)
    }
    pub fn contains_key(&self, key: &Key) -> bool {
        self.map.contains_key(key)
    }
}