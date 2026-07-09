# Checkpoint

1. `answer` advances to the next question even when the answer is *wrong*.
   Why does that design choice make sense for a quiz bot? What would go
   wrong (from a user's perspective) if a wrong answer left the session
   stuck on the same question forever?
2. `main.rs` stores sessions in `Arc<Mutex<HashMap<ChatId, QuizSession>>>`,
   using `tokio::sync::Mutex` (not `std::sync::Mutex`, which you used in
   Phase 2's threads lesson). Look up why an async codebase generally wants
   `tokio::sync::Mutex` instead of the std one for a lock that might be
   held across an `.await` point — what could go wrong with a std `Mutex`
   there?
3. `handle_answer` uses `let Ok(answer) = text.trim().parse::<usize>() else { return Ok(()); };`
   twice in a row (once for parsing, once for finding the chat's session).
   What is this function's behavior for a plain chat message that isn't a
   number, or arrives before `/quiz` was ever sent? Is silently doing
   nothing the right call here, or would you design it differently?
4. Add a third command, `/skip`, that advances to the next question
   without scoring the current one as right or wrong. What would you need
   to add to `QuizSession` to support it cleanly?
