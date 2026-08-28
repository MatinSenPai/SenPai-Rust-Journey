# 04 — Lifetimes and conversion

You have been borrowing since Phase 1 without ever naming how long a borrow
is allowed to live. This module gives that duration an explicit name, uses
it to explain why a struct is allowed to hold a reference at all, and then
shows you how to avoid cloning text you do not actually need to own.

1. [Lifetime basics and elision](01-lifetime-basics-and-elision/README.md)
2. [Lifetimes in structs and methods](02-lifetimes-in-structs-and-methods/README.md)
3. [`Deref`, `AsRef`, `Borrow`, `ToOwned`](03-deref-asref-borrow/README.md)
4. [`Cow<'_, str>` and copy-on-write](04-cow-and-clone-on-write/README.md)
