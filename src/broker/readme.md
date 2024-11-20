# Broker

In stock trading systems, positions are typically added, updated, or removed based on unique identifiers (e.g., stock symbols). 

why choose `BTreeMap`?
- `BinaryHeap`: easy to get `front` value, but no direct way to get `back` value
- `HashMap` or `HashSet`: there is no order
- `BTreeSet`: the item in `BTreeSet` is *immutable*. For modification, you have pop and modify, and insert back to `BTreeSet`
- `LinkedList`: doubly-linked list, not support by pyo3
  - **Cons**
    1. Poor Cache Locality
    2. Higher Memory Overhead
    3. Slower Access Times
    4. Inefficient Iteration
    5. Limited Use Cases, Not Fitting Rust’s Ownership Model Well
    6. No Pooling or Memory Reuse
- `VecDeque`: not support by pyo3, easy to get `front` and `back`, but hard to random pop
  - **Pros**: O(1) for inserting and removing elements at both ends.
  - **Cons**: Less Cache-Friendly, not as efficient as `Vec`, Higher Memory Overhead
- `Vec`: Insertion and deletion can be inefficient for large datasets.
  - **Pros**: Contiguous Memory: Better cache locality and lower memory overhead
  - **Cons**: Insertions/Deletions in the Middle: *O(n)* time complexity, but these are often rare in trading applications.  
- `BTreeMap`:
  - **Pros**: Ordered Keys, *O(log n)* for insertions, deletions, and lookups
  - **Cons**: Slower than `HashMap` for Key-Based Access: Due to the underlying tree structure. less cache-friendly than `HashMap`.

Finally, I choose `Vec`, without removal in the container, just modify inplace.