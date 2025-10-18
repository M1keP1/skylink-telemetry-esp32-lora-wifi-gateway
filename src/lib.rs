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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get() {
        let mut store = Store::new();
        store.put(Key::Int(5), Value::Int(6));
        let v = store.get(&Key::Int(5));
        let v_expected = Some(&Value::Int(6));
        assert_eq!(v_expected, v,"Expected {:?}, got {:?}", v_expected, v);
    }
    #[test]
    fn test_put_update_and_get() {
        let mut store = Store::new();
        store.put(Key::Int(5), Value::Int(6));
        store.update(Key::Int(5), Value::Int(9));
        assert_eq!(store.get(&Key::Int(5)),Some(&Value::Int(9)),"Expected {:?}, got {:?}", Some(5), Some(Value::Int(6)));
    }

    #[test]
    fn test_put_contains_key() {
        let mut store = Store::new();
        store.put(Key::Int(5), Value::Int(6));
        assert_eq!(true,store.contains_key(&Key::Int(5)));
    }

}
