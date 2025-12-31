# Error Handling Implementation Comparison


## Implementation Process

Starting with anyhow felt almost mechanical. The pattern was straightforward: find every place that returns `Option` or uses `unwrap()`, change the signature to `Result`, and replace error cases with `bail!()`. Most of the work was just typing out descriptive error messages. The whole process took maybe an hour and a half, including tests. It got repetitive pretty quickly - by the tenth `bail!("Buffer too short...")` I was just going through the motions.

With thiserror, I had to stop and think before writing any code. What errors can actually happen? Which ones do users need to know about versus which are just internal implementation details? Should checksum mismatches be separate from other deserialization failures? These design questions took time. Once I'd settled on having `DeserializationError` as internal and `StoreError` as public, the actual implementation still took longer than anyhow because I had to explicitly convert between error types at the API boundary. Total time was closer to three hours.

## Error Messages and Debugging

The error messages from both approaches look pretty similar on the surface. When a key isn't found, anyhow gives you "Key not found: Int(5)" and thiserror gives you basically the same thing. Where they differ is when you have nested errors - anyhow's `.context()` chains are genuinely nice for debugging because you can see the whole path of what went wrong. With thiserror I had to be more deliberate about what information to include in each error variant.

One thing I noticed: with anyhow, I kept adding more and more context because it was so easy. Every function call got wrapped with `.context("doing this thing")`. With thiserror, the error types themselves document what can fail, so I didn't feel the need to add as much extra context. Not sure if that's better or worse, honestly.

## Writing User Code

Here's where the difference really shows up. With anyhow, users of my library are stuck checking error message strings:

```rust
if err.to_string().contains("Key not found") {
    // do something
}
```

This works but it's fragile. If I change the error message, their code breaks. And there's no way for them to exhaustively handle all possible errors because they don't even know what all the errors are.

With thiserror, they can actually match on error types:

```rust
match store.get(&key) {
    Err(StoreError::KeyNotFound(k)) => // use default
    Err(StoreError::DataCorruption { .. }) => // alert admin
    Err(StoreError::InvalidData { .. }) => // skip entry
}
```

The compiler will even warn them if they forget to handle a case. That's a pretty big deal for a library API.

## Internal vs External Errors

This distinction only exists with thiserror. With anyhow, every error is just an `anyhow::Error` - users see everything. With thiserror, I split errors into `DeserializationError` (internal) and `StoreError` (public). Users never see the internal details like "buffer too short at byte 142" - they just see "data corruption detected".

I'm still not entirely sure this separation is worth the effort. It definitely makes the API cleaner, but it also means writing more code to convert between error types. There's a `map_err()` call in almost every public function now.

## What Actually Matters

Looking at my code, most of the time errors just get propagated with `?` regardless of which approach I use. The difference only shows up at the boundaries - when creating errors and when matching on them. For creating errors, anyhow is faster to write. For matching on errors, thiserror is way better.

The question is: who's going to use this library? If it's just me or my team, anyhow is probably fine. We can coordinate on what error messages mean. If it's a public library that other people will use, thiserror starts making more sense because the error types become part of the API contract.

## Testing

Tests are slightly more annoying with thiserror because I can't just check if an error message contains some string. I have to use `matches!()` to check for specific error variants. On the other hand, those tests won't break if I change the error message text, so maybe that's actually better for long-term maintenance.

## Performance

I honestly can't tell the difference in practice. Both approaches have basically zero overhead on the happy path. On the error path, anyhow does a heap allocation for the `Box<dyn Error>` while thiserror keeps the enum on the stack. In theory thiserror is faster, but errors are supposed to be rare, so does it matter? I didn't benchmark it because I doubt it would show anything meaningful.

## What I'd Choose

For this specific project - a key-value store that other people might use - I'd go with thiserror. The ability for users to handle different error types properly is worth the extra implementation time. A library's API is written once but used many times, so it's worth getting it right.

If I was building an application where all the error handling code is mine, I'd probably stick with anyhow. The faster implementation time and easier error context chains would win out.

The hybrid approach (thiserror for public errors, anyhow internally) is interesting but feels like overkill for a project this size. Maybe for a much larger codebase it would make sense.

## Unexpected Lessons

One thing I didn't expect: designing the error types for thiserror actually made me think harder about what can go wrong in my code. With anyhow, I just threw errors whenever something failed. With thiserror, I had to categorize failures and decide which ones were distinct enough to warrant their own variant. That forced me to understand my own code better.

Another surprise: users being unable to access the internal `cause` field (because `DeserializationError` is private) is actually a good thing. They can match on `StoreError::DataCorruption { .. }` to know corruption happened, but they can't accidentally depend on internal implementation details. If I refactor how deserialization works later, their code won't break.
