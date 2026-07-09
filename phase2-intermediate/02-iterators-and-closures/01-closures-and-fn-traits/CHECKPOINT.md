# Checkpoint

1. `apply_twice` is bounded by `F: Fn(i32) -> i32`. If you tried to pass it
   a closure that does `|x| { count += 1; x + 1 }` (mutating a captured
   `count`), would it compile? Why or why not — which trait would that
   closure actually implement?
2. `call_n_times` is bounded by `F: FnMut()` rather than `F: Fn()`. Could
   you change the bound to `Fn()` and still pass `|| log.push("tick")` to
   it? Explain in terms of what `Vec::push` needs from its receiver.
3. `make_multiplier` returns `impl Fn(i32) -> i32` and its closure uses
   `move`. What compile error would you expect if you removed the `move`
   keyword and left everything else the same? (Try it if you're not sure.)
4. In your own words: why is `FnOnce` described as the "weakest" of the
   three traits, and `Fn` the "strongest"? Which one would you reach for
   as a generic bound if you didn't yet know how the closure's body would
   use its captures, and why?
5. Rust infers whether a given closure is `Fn`, `FnMut`, or `FnOnce` from
   its body — you never annotate it yourself. Is there a Python equivalent
   of the compiler statically limiting how many times you're allowed to
   call a function based on what it does internally? What would go wrong
   at runtime in Python if you called a "single use" closure (one that
   moves/consumes a captured value) a second time?
