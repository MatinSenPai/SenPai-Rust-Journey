# Checkpoint — What is a compiled language?

Answer these in your own words (out loud or written) before moving on. There's
no code to run for this lesson — the point is whether you can explain it, not
whether a test passes.

1. In your own words, what does `rustc` (the compiler) actually produce, and
   how is that different from what `python3` produces when you run a `.py`
   file?
2. If a Rust program has a bug on line 400 that would only run if a user
   picks a very rare menu option, at what point could that bug realistically
   be caught: while compiling, or only once that menu option is actually
   picked at run time? Why? Would your answer change for the equivalent
   Python program?
3. You copy a compiled Rust binary to a friend's Linux machine (same CPU
   architecture) that has never had Rust installed. Will it run? Would the
   same be true if you copied over just your `.py` file to a machine with no
   Python installed?
4. Where do you expect "more friction" as a Python developer switching to
   Rust: before you run the program, or while it's running? Does that trade
   sound worth it to you for backend work — why or why not?
