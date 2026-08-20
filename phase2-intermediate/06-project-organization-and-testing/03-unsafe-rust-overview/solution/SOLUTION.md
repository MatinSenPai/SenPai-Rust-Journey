# Solution

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

`slice.as_mut_ptr()` (itself completely safe — creating a raw pointer never
requires `unsafe`) gives a `*mut T` pointing at the slice's first element.
`ptr.add(mid)` computes a pointer `mid` elements further along (also safe
to *compute*, per pointer arithmetic rules — only dereferencing/using it
incorrectly is where danger lives). `std::slice::from_raw_parts_mut(ptr,
len)` is the actual `unsafe` operation: it builds a `&mut [T]` out of a raw
pointer and a length, and the function's entire safety contract (documented
in the standard library) is exactly what our `// SAFETY:` comment argues:
the pointer must be valid for `len` elements, and — critically for this
exercise — no other reference to that memory may exist for as long as the
returned slice does.

That last part is the whole reason this function needs `unsafe` at all:
we're personally guaranteeing the two resulting slices don't overlap
(`[0, mid)` and `[mid, len)`, which is true by construction — `mid` is a
single fixed cut point) — a guarantee the borrow checker has no built-in
way to verify from the code alone, because all it can see is "two mutable
borrows of `slice`," without any notion of "but these specific byte ranges
are disjoint." `unsafe` is precisely the keyword for "I'm asserting
something true that the type system can't check for itself here."

On recall question 1: without the `assert!`, `ptr.add(10)` on a
3-element slice computes a pointer past the end of the allocation.
Computing an out-of-bounds pointer with `.add()` is *itself* already
undefined behavior in Rust (not just dereferencing it) — the standard
library's actual documented safety contract for `.add()` requires the
result to point within the same allocated object (or exactly one past its
end). This is a genuinely sharp edge: the danger doesn't wait for you to
read through the pointer, it can already be present in the arithmetic
itself. This is exactly why the `assert!` in safe code, *before* the
`unsafe` block, is what makes the whole function actually safe to call —
delete it, and every one of this exercise's tests would still often "work"
by luck (undefined behavior isn't guaranteed to crash), which is a much
scarier property than a clean panic.
