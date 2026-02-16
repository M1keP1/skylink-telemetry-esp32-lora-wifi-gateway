# **Unsafe vs Safe Parser**

The unsafe parser feels much more “direct.” You’re just copying the bytes of the header straight into the buffer, no extra steps. It’s fast and kind of satisfying once you understand the layout. For small headers like ours, it makes sense and you can see why low-level code does it this way.

The downside is that it’s easy to mess something up. One wrong pointer or `set_len` and you end up with weird bugs that the compiler can’t help with. Also, this approach isn’t very portable—different machines might have different endianness or alignment, so the bytes might not be interpreted the same way everywhere. The safe version was slower but a lot more straightforward to read and harder to break accidentally.

I’d use the unsafe approach when performance matters or when the format is fixed and well-defined. For anything that changes often, needs to be portable, or easy to maintain, I’d stick with the safe parser.

Overall, it was useful to try both. The unsafe one is cool, but definitely something I’d only use when I really need it.
