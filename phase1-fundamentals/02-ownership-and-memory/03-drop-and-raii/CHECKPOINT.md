# Checkpoint

1. Why do values drop in *reverse* declaration order within a scope? (Hint:
   think about a struct that holds a reference into another value declared
   earlier in the same scope — Phase 1's next module. What would go wrong
   if drop order were forward instead?)
2. In `move_extends_lifetime`, the `Tracker` is created *inside*
   `create_tracker`'s body but doesn't drop there. What ownership rule from
   the move-semantics lesson explains why?
3. Name a real resource (not memory) where "cleaned up deterministically the
   instant its owner's scope ends, no exceptions" is a property you'd
   actively want — think about what you'll build in Phase 3-4 (a database
   connection, a file, a lock).
4. Python's `with open(...) as f:` gives you scope-tied cleanup, but only if
   you remember to use `with` — a plain `f = open(...)` doesn't force it.
   What's the equivalent "forgetting" mistake in Rust, if any?
