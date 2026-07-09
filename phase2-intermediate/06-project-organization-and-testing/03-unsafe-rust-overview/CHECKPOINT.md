# Checkpoint

1. Remove the `assert!(mid <= len, ...)` line entirely and call
   `split_at_mut_demo(&mut [1, 2, 3], 10)`. It won't panic anymore — what
   actually happens? (You don't have to run this if you're not comfortable
   — reason through it: `ptr.add(mid)` walks 10 elements past a 3-element
   allocation. Is that operation itself unsafe, or only "wrong" once you
   read through the resulting pointer?)
2. Find the `// SAFETY:` comment above the `unsafe` block. What specific
   claim is it making, and whose responsibility is it to keep that claim
   true — the compiler's, or yours? What happens to that responsibility if
   someone later changes the assertion above from `mid <= len` to something
   subtly wrong?
3. List, from memory, the five things `unsafe` unlocks. Which one does this
   lesson's exercise use?
4. Why can't the borrow checker verify that `&mut slice[..mid]` and
   `&mut slice[mid..]` are non-overlapping, even though they obviously are
   to you, reading the code? What would it need to know that it doesn't?
