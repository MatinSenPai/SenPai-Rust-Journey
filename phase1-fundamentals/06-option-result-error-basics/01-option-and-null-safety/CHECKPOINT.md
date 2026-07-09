# Checkpoint

1. `find_by_id` returns `Option<String>`, not `Option<&String>`. Why does
   the exercise ask for a clone here rather than a borrowed reference? What
   would returning `Option<&String>` tie the result's lifetime to?
2. Try calling `.unwrap()` on the result of `find_by_id(&sample_users(),
   99)` (which is `None`) in a scratch test. What happens, and how is that
   different from what happens in Python if you access a dict key that
   doesn't exist with `d[key]` vs. `d.get(key)`?
3. `average_known_age(&[])` returns `None`, not `0.0` or a panic. Why is
   `None` the semantically correct answer here, and not just a defensive
   habit?
4. Where in code you've already written in this repo (any earlier lesson)
   did you already use `Option` without this lesson naming it explicitly?
