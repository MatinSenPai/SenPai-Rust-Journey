# Solution

## `split_at_mut_demo`

```rust
pub fn split_at_mut_demo<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    let len = slice.len();
    assert!(mid <= len, "mid out of bounds");
    let ptr = slice.as_mut_ptr();
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}
```

`slice.as_mut_ptr()` (itself completely safe — creating a raw pointer never requires `unsafe`) gives a `*mut T` pointing at the slice's first element. `ptr.add(mid)` computes a pointer `mid` elements further along (also safe to *compute*, per pointer arithmetic rules — only dereferencing/using it incorrectly is where danger lives). `std::slice::from_raw_parts_mut(ptr, len)` is the actual `unsafe` operation: it builds a `&mut [T]` out of a raw pointer and a length, and the function's entire safety contract (documented in the standard library) is exactly what our `// SAFETY:` comment argues: the pointer must be valid for `len` elements, and — critically for this exercise — no other reference to that memory may exist for as long as the returned slice does.

That last part is the whole reason this function needs `unsafe` at all: we're personally guaranteeing the two resulting slices don't overlap (`0..mid` and `mid..len`, which is true by construction — `mid` is a single fixed cut point) — a guarantee the borrow checker has no built-in way to verify from the code alone, because all it can see is "two mutable borrows of `slice`," without any notion of "but these specific byte ranges are disjoint." `unsafe` is precisely the keyword for "I'm asserting something true that the type system can't check for itself here."

On recall question 1 in the README's warm up: without the `assert!`, `ptr.add(10)` on a 3-element slice computes a pointer past the end of the allocation. Computing an out-of-bounds pointer with `.add()` is *itself* already undefined behavior in Rust (not just dereferencing it) — the standard library's actual documented safety contract for `.add()` requires the result to point within the same allocated object (or exactly one past its end). This is exactly why the `assert!` in safe code, *before* the `unsafe` block, is what makes the whole function actually safe to call — delete it, and this exercise's tests would still often "work" by luck (undefined behavior isn't guaranteed to crash), which is a much scarier property than a clean panic.

## `OwnedBox<T>`

```rust
pub struct OwnedBox<T> {
    ptr: *mut T,
    _marker: PhantomData<T>,
}

unsafe impl<T: Send> Send for OwnedBox<T> {}

impl<T> OwnedBox<T> {
    pub fn new(value: T) -> Self {
        Self {
            ptr: Box::into_raw(Box::new(value)),
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.ptr }
    }

    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for OwnedBox<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.ptr));
        }
    }
}
```

`new` puts `value` on the heap with `Box::new`, then immediately gives up that `Box`'s ownership with `Box::into_raw`, keeping only the raw pointer it hands back. Between that call and `Drop::drop`, nothing in the type system owns the allocation on `OwnedBox`'s behalf — `OwnedBox<T>` itself is standing in for that ownership, entirely by convention, backed by the three methods around it.

`get` and `get_mut` are the payoff of `new`'s hand-off: `Box::into_raw` guarantees the pointer it returns is valid and properly aligned for a `T`, and nothing else in this type ever frees it before `Drop::drop` runs, so dereferencing it back into a `&T` (or `&mut T`, behind `&mut self`) is sound. `Drop::drop` is `Box::into_raw`'s exact mirror: `Box::from_raw` takes the same raw pointer and reconstructs the `Box<T>` around it, and letting that reconstructed `Box` fall out of scope runs `T`'s destructor exactly once, the same way any other `Box<T>` would.

`unsafe impl<T: Send> Send for OwnedBox<T> {}` is the line this lesson is really about. Without it, `OwnedBox<T>` is `!Send` for every `T`, full stop — not because the compiler inspected `T` and found a problem, but because the raw pointer field alone is enough to block the compiler's automatic `Send`. Writing this line personally asserts something the compiler can't verify on its own: that moving an `OwnedBox<T>`'s ownership to another thread, whenever `T` itself is `Send`, is genuinely safe. It's easy to check because `OwnedBox<T>` behaves exactly like `Box<T>` — exclusive ownership, exactly one thing pointing at the value at a time — and `Box<T>` is already `Send` whenever `T` is. This `unsafe impl` isn't claiming anything beyond what `Box` itself already guarantees; it's restating that guarantee for a type the compiler can't see through on its own.

`_marker: PhantomData<T>` doesn't change what compiles here — `ptr: *mut T` already mentions `T`, so removing the marker still builds cleanly. It stays for the same reason every real owning-raw-pointer type in the standard library carries one: it's the honest, conventional signal that this pointer represents ownership of a `T`, not a borrow of one.

### Challenge, part two

Granting `unsafe impl<T: Sync> Sync for OwnedBox<T> {}` on top of this exact API (`get`, `get_mut`, `Drop`, nothing else) would not create an aliasing violation today — `get_mut` needs `&mut self`, and nothing about `Sync` lets two threads obtain `&mut OwnedBox<T>` from a shared `&OwnedBox<T>`. The real danger is what `Sync` promises going forward. Because the borrow checker has no say over what happens behind a raw pointer, it is entirely legal Rust to add a method like `fn set(&self, value: T) { unsafe { *self.ptr = value; } }` — a *safe*-looking public method needing only `&self`, mutating through the pointer anyway. The moment such a method existed, two threads sharing a `Sync` `OwnedBox<T>` could call it at the same time: an unsynchronized write from two places at once, a genuine data race. Staying `!Sync` by default, and granting it only when every present and future method has been checked against exactly this risk, is why the standard library's own raw-pointer types make the same conservative choice.
