# Checkpoint

1. `long_titles` takes `&[String]` and returns `Vec<&str>` rather than
   `Vec<String>`. What would you have to change in the implementation to
   return `Vec<String>` instead, and what's the practical tradeoff (think
   about what happens in memory) between the two signatures?
2. Write this line without running it: what does
   `vec![1, 2, 3].iter().map(|n| n * 2)` evaluate to *by itself*, with no
   `.collect()`, `.sum()`, or other consumer at the end — does anything get
   multiplied? Why does Rust design iterators to work this way instead of
   running eagerly like a Python list comprehension?
3. `total_episodes` can be written with either `.sum()` or `.fold(0, |acc,
   episodes| acc + episodes)`. They produce the same answer here — when
   would you have to reach for `.fold()` because `.sum()` genuinely
   couldn't do the job?
4. If you called `.collect()` on an iterator without ever writing a type
   annotation or turbofish anywhere, what error would the compiler give
   you, and why can't Rust just guess "you probably want a `Vec`"?
5. `first_n_completed` combines `.filter()` and `.take(n)`. Why does
   `.take(n)` need to come *after* `.filter()` in the chain rather than
   before it, given the goal is "the first `n` shows that are completed"?
