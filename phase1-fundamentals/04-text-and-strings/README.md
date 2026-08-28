# 04 — Text and strings

Text is where Rust is strictest, and where that strictness matters most to
you. `len()` is a byte count, a `String` cannot be indexed by a number, and
slicing at the wrong offset panics rather than producing broken output.

Every one of those rules exists because a character is not a byte — which is
invisible in English and unavoidable in Persian.

1. [`String` versus `&str`](01-string-vs-str/README.md)
2. [UTF-8: bytes, chars, graphemes](02-utf8-bytes-chars-graphemes/README.md)
3. [Building and transforming strings](03-building-and-transforming-strings/README.md)
4. [Slicing text safely](04-slicing-text-safely/README.md)
