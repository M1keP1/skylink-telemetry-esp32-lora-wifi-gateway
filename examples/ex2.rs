use kiwi_store::{Store, Key, Value, BorrowedEntry, StoreError};
use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn main() -> Result<(), StoreError> {
    println!("=== Example 2: Buffer Iterator Tests with Persistence ===\n");

    test_iterator_stops_correctly()?;
    test_no_garbage_beyond_buffer()?;
    test_iterator_borrows_no_allocations()?;
    test_persistence_with_iteration()?;

    println!("\n=== All tests passed! ===");
    Ok(())
}

fn test_iterator_stops_correctly() -> Result<(), StoreError> {
    println!("Test 1: Iterator stops at correct position");

    let mut store = Store::new();

    store.put(Key::String("first".into()), Value::Int(1));
    store.put(Key::String("second".into()), Value::Int(2));
    store.put(Key::String("third".into()), Value::Int(3));
    store.put(Key::String("fourth".into()), Value::String("hello".into()));
    store.put(Key::String("fifth".into()), Value::String("world".into()));

    let entries: Result<Vec<_>, _> = store.buffer_iter().collect();
    let entries = entries?;

    assert_eq!(entries.len(), 5, "Iterator should yield exactly 5 entries");

    assert_eq!(entries[0], BorrowedEntry::Int(1));
    assert_eq!(entries[1], BorrowedEntry::Int(2));
    assert_eq!(entries[2], BorrowedEntry::Int(3));
    assert_eq!(entries[3], BorrowedEntry::Text("hello"));
    assert_eq!(entries[4], BorrowedEntry::Text("world"));

    println!("  ✓ Iterator yielded exactly {} entries", entries.len());
    println!("  ✓ All entries are correct\n");

    Ok(())
}

fn test_no_garbage_beyond_buffer() -> Result<(), StoreError> {
    println!("Test 2: No garbage beyond buffer");

    let mut store = Store::new();

    store.put(Key::Int(1), Value::Int(100));
    store.put(Key::Int(2), Value::String("test".into()));
    store.put(Key::Int(3), Value::Int(200));

    let mut count = 0;
    for entry_result in store.buffer_iter() {
        let entry = entry_result?;
        count += 1;
        match entry {
            BorrowedEntry::Int(i) => {
                assert!(i == 100 || i == 200, "Unexpected integer value: {}", i);
            }
            BorrowedEntry::Text(s) => {
                assert_eq!(s, "test", "Unexpected text value: {}", s);
            }
        }
    }

    assert_eq!(count, 3, "Should iterate exactly 3 times, not more");

    println!("  ✓ Iterator stopped after {} valid entries", count);
    println!("  ✓ No garbage values were yielded\n");

    Ok(())
}

fn test_iterator_borrows_no_allocations() -> Result<(), StoreError> {
    println!("Test 3: Iteration borrows data (no heap allocations)");

    let mut store = Store::new();

    store.put(Key::String("key1".into()), Value::String("value1".into()));
    store.put(Key::String("key2".into()), Value::Int(42));
    store.put(Key::String("key3".into()), Value::String("value3".into()));
    store.put(Key::String("key4".into()), Value::Int(99));

    let reg = Region::new(&GLOBAL);

    let mut entry_count = 0;
    for entry_result in store.buffer_iter() {
        let entry = entry_result?;
        entry_count += 1;
        match entry {
            BorrowedEntry::Int(i) => {
                let _ = i;
            }
            BorrowedEntry::Text(s) => {
                let _ = s.len();
            }
        }
    }

    let stats = reg.change();

    println!("  Iteration statistics:");
    println!("    - Entries processed: {}", entry_count);
    println!("    - Allocations: {}", stats.allocations);
    println!("    - Deallocations: {}", stats.deallocations);
    println!("    - Bytes allocated: {}", stats.bytes_allocated);
    println!("    - Bytes deallocated: {}", stats.bytes_deallocated);

    assert_eq!(
        stats.allocations, 0,
        "Iterator should not allocate memory (found {} allocations)",
        stats.allocations
    );
    assert_eq!(
        stats.bytes_allocated, 0,
        "Iterator should not allocate bytes (found {} bytes)",
        stats.bytes_allocated
    );

    println!("  ✓ Zero allocations during iteration");
    println!("  ✓ Data is borrowed, not cloned\n");

    Ok(())
}

fn test_persistence_with_iteration() -> Result<(), StoreError> {
    println!("Test 4: Persistence preserves iteration order");

    let store_path = "/tmp/ex2_test_store";

    {
        let mut store = Store::with_path(store_path)?;
        store.put(Key::String("a".into()), Value::Int(1));
        store.put(Key::String("b".into()), Value::Int(2));
        store.put(Key::String("c".into()), Value::Int(3));
        store.save()?;
    }

    let loaded_store = Store::with_path(store_path)?;

    let entries: Result<Vec<_>, _> = loaded_store.buffer_iter().collect();
    let entries = entries?;

    assert_eq!(entries.len(), 3, "Should have 3 entries after reload");
    assert_eq!(entries[0], BorrowedEntry::Int(1));
    assert_eq!(entries[1], BorrowedEntry::Int(2));
    assert_eq!(entries[2], BorrowedEntry::Int(3));

    println!("  ✓ Loaded store has {} entries", entries.len());
    println!("  ✓ Buffer iteration order preserved after save/load\n");

    std::fs::remove_file(format!("{}.keys", store_path)).ok();
    std::fs::remove_file(format!("{}.data", store_path)).ok();
    std::fs::remove_file(format!("{}.meta", store_path)).ok();

    Ok(())
}