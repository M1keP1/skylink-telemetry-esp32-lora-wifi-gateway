# Iterator Design Questions - Lab 5

## Question 1: How do lifetimes ensure that BorrowedEntry values cannot outlive the store?

The lifetime annotation `'a` in `StoreIter<'a>` ties the iterator to the store's lifetime. When we return `BorrowedEntry` instances from the iterator, they contain references with the same lifetime `'a` that points into the store's `Vec<u8>` buffer.

The compiler tracks this and won't let you use a `BorrowedEntry` after the store is dropped.

## Question 2: What happens if you mutate the store while iterating? Should this be allowed?

No, it shouldn't be allowed and Rust prevents it automatically.

When you call `iter()`, it borrows the store immutably (`&self`). Rust's borrowing rules say you can't have a mutable borrow while an immutable borrow exists. So if you try to call `put()` or `insert()` (which need `&mut self`) while an iterator is active, it won't compile.

This is good because:
- The iterator assumes the buffer layout stays the same
- Inserting new entries could reallocate the `Vec<u8>`, invalidating all the references
- Even without reallocation, changing the buffer would make the iterator's position pointer unreliable

## Question 3: Could you write a StoreIterMut with mutably-borrowed entries? What would happen if you change data types?

You could technically write a `StoreIterMut` that returns `&mut [u8]` slices pointing to entries in the buffer. But it would be dangerous and pretty much useless.

The problem is that our entries are serialized with a specific format. If you modify the actual bytes:

- Changing the length fields would make the iterator read garbage for the next entry
- Changing a key or value to a different type (like turning a u32 into a String) would require a different amount of space
- You can't just overwrite bytes in place because you might need more or less space than before
- The whole buffer structure would get corrupted

The only "safe" mutation would be to overwrite a value with another value of the exact same byte length, which is super restrictive and not very useful. Better to just use the normal `put()` method which handles all the serialization properly.

So: theoretically possible, practically useless and dangerous.