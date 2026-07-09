# Solution

```rust
pub fn format_question(number: usize, question: &Question) -> String {
    let mut out = format!("Q{number}: {}\n", question.text);
    for (i, option) in question.options.iter().enumerate() {
        out.push_str(&format!("{}) {option}\n", i + 1));
    }
    out.trim_end().to_string()
}
```

`.enumerate()` gives 0-indexed positions; `i + 1` converts to the
1-indexed display format the README promises. `.trim_end()` drops the
trailing newline from the loop's last `push_str`.

```rust
pub fn answer(&mut self, answer_1_indexed: usize) -> bool {
    let Some(question) = self.current_question() else {
        return false;
    };
    let correct = answer_1_indexed.checked_sub(1) == Some(question.correct_index);
    if correct {
        self.score += 1;
    }
    self.current += 1;
    correct
}
```

`answer_1_indexed.checked_sub(1)` rather than a bare `answer_1_indexed - 1`:
`usize` subtraction underflows (panics in debug builds) if `answer_1_indexed`
is `0` — a real possibility since this value ultimately comes from
user-typed text in `main.rs`. `checked_sub` returns `None` instead of
panicking, and `None == Some(question.correct_index)` is simply `false` —
an invalid answer like `0` is treated the same as "just wrong," not a
crash. This is the same "never trust external input" instinct from
Phase 3's future auth/validation lessons, applied a little early because
Telegram messages are about as "external" as input gets.

## The `main.rs` wiring, briefly

`Sessions = Arc<Mutex<HashMap<ChatId, QuizSession>>>` is shared,
`tokio::sync::Mutex`-protected state — one `QuizSession` per chat, so
multiple users (or multiple chats) can each have their own quiz in
progress simultaneously. `dptree::entry().branch(...).branch(...)` tries
each branch in order: first "is this a recognized `/quiz` or `/score`
command," and if not, falls through to "treat it as a plain-text answer
attempt." `tokio::sync::Mutex` (not `std::sync::Mutex`) matters here
specifically because the lock is held *across* `.await` points in a couple
of places (e.g. sending a message while still holding the guard would be
a mistake with either kind, but `tokio::sync::Mutex`'s guard is itself
`Send` across `.await` in ways the std version's isn't designed for,
and holding a std `Mutex` guard across an `.await` risks deadlocking the
whole async runtime if that task yields to another one that then also
wants the same lock).
