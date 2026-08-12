---
name: senpai-rust-course-author
description: Author and review bilingual SenPai Rust lessons with accurate Rust semantics, Persian pedagogy, accessible SVG concept visuals, and repository verification gates.
---

# SenPai Rust Course Author

Use when adding, translating, or reviewing curriculum content in this repo.

1. Keep the English canonical file and place Persian in `<stem>.fa.md`.
2. Preserve Rust identifiers, crate names, commands, protocols and exact
   compiler diagnostics; translate the explanation around them.
3. Introduce a technical term as `معادل فارسی (English term)` once, then use
   the established glossary form.
4. Teach in this order: outcome, familiar Persian example, exact Rust rule,
   backend/Python bridge, common compiler failure, exercise/checkpoint/solution.
5. State where an analogy stops being exact. Never trade semantic accuracy for
   a catchy metaphor.
6. Keep code and paths LTR. Treat Persian UTF-8 data carefully in lessons about
   bytes, `.len()`, slicing and character counts.
7. Add at least one valid `senpai-visual` JSON fence per lesson; use multiple
   focused figures for ownership, borrowing, lifetimes, async and distributed
   systems.
8. Run formatting, course UI tests, strict course UI clippy, workspace no-run
   compilation and the translation audit before marking a batch complete.

## Persian voice and editing standard

- Translate meaning and intent, not English word order. Read the whole lesson
  and its code before choosing the Persian sentence.
- Address Matin directly in a warm, precise, conversational voice. Prefer
  short active sentences and natural Persian verbs; avoid stiff passive prose.
- Keep technical English only when it is an identifier, an established Rust
  term, or genuinely clearer. Introduce it once beside the Persian term and do
  not alternate among several translations later.
- Use Persian ی and ک, Persian punctuation and digits in prose, and ZWNJ in
  forms such as `می‌شود`, `به‌جای` and `همه‌ی`.
- Do not invent colorful idioms just to make the text lively. A familiar
  Iranian example must clarify the exact rule, and its limits must be stated.
- Preserve every factual qualification from the canonical lesson. Do not add,
  omit, soften or strengthen technical claims for the sake of fluency.
- After drafting, read the Persian paragraph by itself. If it sounds like a
  sentence translated from English, rewrite it as a Persian teacher would say
  it aloud.
