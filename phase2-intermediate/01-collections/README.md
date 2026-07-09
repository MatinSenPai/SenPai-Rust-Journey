# 01 — Collections

Python hands you `list` and `dict` and calls it a day — both are flexible
enough for almost anything. Rust instead gives you a small toolbox of
collections, each with different performance and ordering guarantees, and
expects *you* to pick the right one for the job. That's more upfront
thinking than Python asks of you, but it pays off: reaching for `HashSet`
instead of a `Vec` you manually de-duplicate, or `VecDeque` instead of a
`Vec` you keep calling `.remove(0)` on, isn't a micro-optimization here —
it's the idiomatic, obvious choice once you know the shapes exist. Both
lessons build on the same running example: analyzing a list of anime/show
watch data.

1. [`Vec` and `HashMap`](01-vec-and-hashmap/README.md)
2. [`BTreeMap`, `HashSet`, `VecDeque`](02-btreemap-hashset-vecdeque/README.md)
