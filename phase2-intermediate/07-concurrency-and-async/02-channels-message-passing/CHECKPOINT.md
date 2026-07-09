# Checkpoint

1. In `collect_from_producers`, what happens if you forget to `drop(tx)`
   (the original sender kept in the parent thread) after spawning every
   producer? Try it — does the test hang, panic, or something else? Why?
2. `compute_async_sum` only ever sends *one* message before the thread
   ends. Could you have used a plain `JoinHandle<i32>` and `.join()`
   instead of a channel here? What would change if the background thread
   needed to send multiple *progress updates* before a final result?
3. `Sender<T>` is `Clone`, but `Receiver<T>` is not. Why does that
   asymmetry make sense given "multi-producer, single-consumer"?
4. Compare this lesson's `collect_from_producers` to last lesson's
   `count_matching_in_threads` (which used `Arc<Mutex<i32>>`). Both gather
   results from multiple threads — what's different about *what* each one
   is gathering that makes a channel the more natural fit here, and a
   `Mutex` the more natural fit there?
