# Checkpoint

1. Add a fifth variant to `Status`, e.g. `Announced`, and run `cargo check`
   *without* updating any of the three functions. What does the compiler
   say, for each function? Then actually update all three (however you
   like) so it compiles again, and remove `Announced` when done.
2. Could you model `Status` as a struct instead, with an optional
   `latest_chapter: Option<u32>` field and a separate `is_cancelled: bool`?
   What invalid states would that representation allow that the enum
   version doesn't?
3. In `describe`, each `match` arm destructures a different named field
   (`latest_chapter`, `since_chapter`, `total_chapters`). Why doesn't `_ =>`
   work as a catch-all if you wanted to handle `Ongoing` and `Hiatus`
   identically — could you combine those two arms, and if so, how?
4. This pattern (an enum modeling "one of several named states, some
   carrying different data") — where else in a real backend might you use
   this instead of a string status field plus a pile of optional columns?
   Think about `taskforge`'s eventual `JobStatus` (mentioned in the
   capstone README) as one concrete answer.
