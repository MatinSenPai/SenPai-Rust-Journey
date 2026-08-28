# Solution

```rust
pub fn from_vec(items: &[i32]) -> List {
    let mut list = List::Nil;
    for &item in items.iter().rev() {
        list = List::Cons(item, Box::new(list));
    }
    list
}
```

The interesting bit is the direction of the loop. A cons list only has
"forward" pointers — each `Cons` points at the *rest* of the list, and
there's no way to append to the end of an existing list without walking
all the way to its `Nil` and rebuilding every node along the way (`List`
isn't mutable in place here). So instead of building front-to-back, this
walks `items` **backward** (`.iter().rev()`) and builds the list
inside-out: the last element becomes `Cons(last, Nil)` first, then each
earlier element wraps what's already been built in one more `Cons` layer.
By the time the loop finishes, the *first* element of `items` is the
outermost `Cons` — which is exactly what "preserves order" means for this
data structure.

`sum` is the other interesting one:

```rust
pub fn sum(&self) -> i32 {
    match self {
        List::Cons(val, rest) => val + rest.sum(),
        List::Nil => 0,
    }
}
```

`rest` here has type `&Box<List>` (matching on `&self` propagates the
borrow into the pattern). Calling `rest.sum()` directly — no `(**rest)` or
`(*rest)`, nothing — works because `Box<List>` implements `Deref<Target =
List>`, and method calls automatically insert as many derefs as needed to
find a matching method (this is "auto-deref," and it chains through
multiple layers of pointer-like types, not just one).

On recall question 3: if `sum` took `self` by value instead of `&self`,
calling `list.sum()` would **consume** the entire list — every `Cons` node
gets moved into the function and dropped as it recurses. That's wasteful
if you only want to *read* the total (you'd need to rebuild the whole list
to use it again afterward), and it's also simply not necessary: summing
never needs to modify or take ownership of the list, so `&self` is the
correct, minimal-permission signature — the same "only ask for the access
you actually need" principle from the structs lesson.
