# Checkpoint

1. `sorted_by_key` is implemented as `counts.into_iter().collect()` — no
   explicit sort call anywhere. Where does the sorting actually happen, and
   why doesn't `HashMap`'s unordered iteration order "leak through" into
   the resulting `BTreeMap`?
2. `HashMap` lookups/inserts are (average) `O(1)`; `BTreeMap`'s are
   `O(log n)`. Given that, when would you deliberately choose `BTreeMap`
   over `HashMap` even though it's slower? Name a concrete scenario beyond
   "I want sorted output."
3. `shared_genres` and `exclusive_genres` both end with `.collect()`. What
   would the return type of `a.intersection(b)` be if you *didn't*
   `.cloned()` before collecting — and why wouldn't that type compile as
   this function's `HashSet<String>` return type?
4. `WatchQueue::watch_next_priority` uses `push_front`. If `WatchQueue`
   wrapped a `Vec<String>` instead of a `VecDeque<String>`, what method
   would you have to call to get the same "jump to the front" behavior, and
   what's its time complexity compared to `VecDeque::push_front`?
5. `WatchQueue` has both `len` and `is_empty`, and `len` is not implemented
   in terms of `is_empty` or vice versa — the exercise asked for both. Why
   might a crate author bother writing `is_empty` at all when a caller
   could just check `queue.len() == 0`?
