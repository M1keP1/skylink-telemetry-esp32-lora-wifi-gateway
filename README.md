# Kiwi Store

A simple, fast key-value store built in Rust. Stores data in memory with optional persistence to disk.

## What is this?

This is a learning project implementing a basic key-value database. It supports string and integer keys/values, handles deletions with compaction, and persists data across restarts. Think of it as a minimal version of something like Redis or LevelDB, but much simpler.

## Project Structure

```
src/
├── lib.rs              # Public API exports
├── types.rs            # Key, Value, and Entry types
├── error.rs            # Error handling
├── store.rs            # Main Store implementation
├── iterator.rs         # Iterator types for traversing data
└── serialization/      # Binary format handling
    ├── mod.rs          # Module exports
    ├── header.rs       # Low-level header serialization (unsafe code)
    ├── key.rs          # Key encoding/decoding
    └── value.rs        # Value encoding/decoding
```

The code is organized by responsibility - types in one place, errors in another, serialization logic separated from store logic. This makes it easier to find things and modify specific parts without touching everything else.

## API

### Basic Operations

```rust
use kiwi_store::{Store, Key, Value};

let mut store = Store::new();

// Put data
store.put(Key::String("name".into()), Value::String("Alice".into()));
store.put(Key::Int(42), Value::Int(100));

// Get data
let value = store.get(&Key::String("name".into()))?;

// Delete data
store.delete(&Key::String("name".into()))?;

// Clear everything
store.clear();
```

### Persistence

```rust
// Create store with a file path
let mut store = Store::with_path("my_store")?;

// Data auto-saves when dropped
// Or save manually:
store.save()?;

// Load existing store
let store = Store::load("my_store")?;
```

### Maintenance

```rust
// Check fragmentation
let ratio = store.fragmentation_ratio();

// Compact to reclaim space
let bytes_freed = store.compact()?;
```

### Iteration

```rust
// Iterate over key-value pairs
for (key, value_result) in store.iter() {
    let value = value_result?;
    println!("{:?} -> {:?}", key, value);
}

// Just keys
for key in store.keys() {
    println!("{:?}", key);
}

// Just values
for value_result in store.values() {
    let value = value_result?;
    println!("{:?}", value);
}
```

## How It Works

### Storage Model

The store uses an append-only log for values and a HashMap index for lookups:

- **Index**: `HashMap<Key, usize>` - maps keys to byte offsets in the data buffer
- **Data**: `Vec<u8>` - raw bytes containing all serialized values
- **Path**: `Option<PathBuf>` - optional file path for persistence

When you put a value, it gets serialized and appended to the data buffer. The index stores where to find it. This makes writes fast (just append) but means updates and deletes create "dead" space that needs compaction.

### Serialization Format

Each value is stored with a header:

```
[Header: 13 bytes][Value data: variable]
```

Header structure (packed):
- `length: u64` - size of value data
- `checksum: u32` - CRC32 of value data
- `tag: u8` - type identifier (0x01 = String, 0x02 = Int)

Keys use a simpler format:
- `tag: u8` - type identifier
- `length: u64` - for strings
- `data: [u8]` - actual key bytes

The checksum catches corruption. The tag lets us know what type we're deserializing.

### Fragmentation & Compaction

When you update or delete a key, the old value stays in the data buffer but becomes unreachable. This is fragmentation. The `fragmentation_ratio()` method tells you what percentage of the buffer is wasted space.

Compaction rebuilds the data buffer with only active values:
1. Create new empty buffer
2. For each key in index, copy its value to new buffer
3. Update index with new offsets
4. Replace old buffer with new one

Auto-compaction triggers on save if fragmentation exceeds 35%.

### File Format

Persistence uses three files:

- `<path>.keys` - serialized index (key + offset pairs)
- `<path>.data` - raw value data
- `<path>.meta` - metadata (version, checksums, entry count)

The meta file checksums both keys and data files to detect corruption. On load, we verify checksums before trusting the data.

## Examples

### ex1.rs, ex2.rs, ex3.rs
Basic usage examples showing put/get/delete operations and iteration.

### wikipedia_stress_test.rs
Loads ~7 million Wikipedia pageview records to test performance and correctness under load. Processes about 1.15 million records per second in release mode. Good for finding bugs that only show up with large datasets.

Run it:
```bash
cargo run --release --bin wikipedia_stress_test
```

## Design Choices

**Why append-only?** Fast writes, simple implementation. The tradeoff is fragmentation, but compaction handles that.

**Why HashMap index?** O(1) lookups. Could use a B-tree for range queries, but that's more complex and we don't need it.

**Why separate serialization modules?** Keeps unsafe code isolated in `header.rs`. Makes it easier to change the format without touching store logic.

**Why CRC32 checksums?** Fast to compute, good enough for detecting corruption. Not cryptographically secure, but we're not trying to be.

**Why three files for persistence?** Separating keys and data lets us load the index without reading all values. The meta file provides integrity checking.

## Testing

Run tests:
```bash
cargo test
```

Run with output:
```bash
cargo test -- --nocapture
```

The test suite covers serialization, store operations, persistence, compaction, and edge cases. 19 tests total.

## Performance Notes

- Release builds are ~3x faster than debug builds
- Throughput: ~1.15M operations/sec (Wikipedia stress test)
- Memory: O(n) for index + data
- Compaction: O(n) time, creates temporary copy of data

## License

This is a learning project. Use it however you want.