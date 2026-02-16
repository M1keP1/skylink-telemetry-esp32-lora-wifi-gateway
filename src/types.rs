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

pub fn borrowed_to_owned(entry: &BorrowedEntry) -> OwnedEntry {
    match entry {
        BorrowedEntry::Int(i) => OwnedEntry::Int(*i),
        BorrowedEntry::Text(s) => OwnedEntry::Text(s.to_string()),
    }
}

pub fn owned_to_value(entry: &OwnedEntry) -> Value {
    match entry {
        OwnedEntry::Int(i) => Value::Int(*i),
        OwnedEntry::Text(s) => Value::String(s.clone()),
    }
}
