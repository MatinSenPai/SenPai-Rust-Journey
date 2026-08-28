# Solution — 2.6.5 `RefCell`, `Cell`, and the run-time panic trade

```rust
impl ClubStats {
    pub fn new() -> Self {
        ClubStats {
            members_online: Cell::new(0),
            watch_log: RefCell::new(Vec::new()),
        }
    }

    pub fn member_joined(&self) {
        self.members_online.set(self.members_online.get() + 1);
    }

    pub fn members_online(&self) -> u32 {
        self.members_online.get()
    }

    pub fn log_episode(&self, title: &str) {
        self.watch_log.borrow_mut().push(title.to_string());
    }

    pub fn watch_log(&self) -> Vec<String> {
        self.watch_log.borrow().clone()
    }
}

impl SharedHypeMeter {
    pub fn new() -> Self {
        SharedHypeMeter {
            inner: Rc::new(RefCell::new(0)),
        }
    }

    pub fn hype_up(&self, amount: i32) {
        *self.inner.borrow_mut() += amount;
    }

    pub fn hype(&self) -> i32 {
        *self.inner.borrow()
    }
}
```

Nothing here needed anything beyond what "The concept" showed — the same two tools, aimed at a slightly different domain.

## `ClubStats` — `Cell` for the counter, `RefCell` for the log

`member_joined`/`members_online` are `.set(.get() + 1)` and `.get()` — the exact `Cell` shape from `PageViews`. `log_episode`/`watch_log` are `.borrow_mut().push(...)` and `.borrow().clone()` — the exact `RefCell` shape from `WatchLog`. Both fields live inside one struct, and every method on it takes only `&self`, because that's the entire point: a `ClubStats` reached through a plain `&ClubStats` (or, later, through an `Rc<ClubStats>`) can still be updated.

`watch_log()` clones the `Vec<String>` out of the `Ref` guard rather than returning the guard itself — the guard borrows from `self` and can't outlive the method call, so the caller gets an owned copy instead. That's a `Ref<Vec<String>>` turned into a `Vec<String>` with one `.clone()`, exactly the "just look, don't touch" reading `.replace()`/`.take()` couldn't give you on a `Cell`.

## `SharedHypeMeter` — the classic `Rc<RefCell<T>>` combo

`self.inner` is `Rc<RefCell<i32>>` — `.borrow_mut()` works through the `Rc`'s shared reference to the `RefCell` (no `&mut self` needed anywhere, since `RefCell` moved the mutability check to run time), returning a `RefMut<i32>` which `*` dereferences to read or write the actual `i32`. The `RefMut`/`Ref` guards here are temporaries — dropped at the end of each statement — which is exactly why calling `hype_up()` twice in a row never panics: each call's borrow is fully released before the next one starts.

`.clone()` on a `SharedHypeMeter` is the derived `Clone`, which clones the `Rc` field — cheap, and it points at the same heap allocation (see 2.6.3). That's what makes `clones_share_the_same_underlying_total` pass: `original` and `handle` are two `Rc` pointers into one `RefCell<i32>`, not two independent meters.

## What this lesson was really about

- **The rule doesn't move unless you tell it to.** `Cell`/`RefCell` only appear on the fields that actually need to change from behind `&self`; nothing else in either struct needed touching.
- **A guard is a temporary, and temporaries drop fast.** `*self.inner.borrow_mut() += amount;` never panics on repeated calls precisely because the `RefMut` it creates doesn't survive past the statement.
- **`Rc<RefCell<T>>` is two separate jobs stacked.** `Rc` decides how many owners there are; `RefCell` decides whether any of them can write. Losing either one loses half the pattern — `Rc<i32>` alone would be read-only, and a bare `RefCell<i32>` (no `Rc`) would still only have one owner.
