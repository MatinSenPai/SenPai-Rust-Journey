# Solution — 3.1.3 HTTP semantics you must know

## `is_safe` and `is_idempotent`

```rust
pub fn is_safe(method: &Method) -> bool {
    matches!(method, Method::Get | Method::Head | Method::Options | Method::Trace)
}

pub fn is_idempotent(method: &Method) -> bool {
    matches!(
        method,
        Method::Get | Method::Head | Method::Put | Method::Delete | Method::Options | Method::Trace
    )
}
```

`matches!` is a `match` that returns `true` for the listed patterns and `false` for everything else. Notice that `is_idempotent`'s list is `is_safe`'s list plus `Put` and `Delete`. That's the "every safe method is idempotent" rule, visible in the code itself. You could write `is_safe(method) || matches!(method, Method::Put | Method::Delete)` to make it explicit, but two flat lists are easier to check against the spec's table.

## `status_class`

```rust
pub fn status_class(status: u16) -> Option<StatusClass> {
    match status {
        100..=199 => Some(StatusClass::Informational),
        200..=299 => Some(StatusClass::Success),
        300..=399 => Some(StatusClass::Redirection),
        400..=499 => Some(StatusClass::ClientError),
        500..=599 => Some(StatusClass::ServerError),
        _ => None,
    }
}
```

The `_ => None` arm is the fix for the `E0004` in "Errors you will meet". It's also the honest answer: a `u16` can hold `999`, and the spec gives it no class. `status / 100` with a `match` on `1..=5` would work too. The ranges are just easier to read against the table.

## `best_content_type`

The solution parses the header once, into a small struct per entry:

```rust
let entries: Vec<AcceptEntry> = accept
    .split(',')
    .map(str::trim)
    .filter(|entry| !entry.is_empty())
    .map(|entry| {
        let (media, q) = match entry.split_once(";q=") {
            Some((media, weight)) => (media.trim(), weight.trim().parse().unwrap_or(1.0)),
            None => (entry, 1.0),
        };
        let (kind, subkind) = media_parts(media);
        AcceptEntry { kind, subkind, q }
    })
    .collect();
```

`unwrap_or(1.0)` is the fix from example `07`: an unparseable `q` counts as `1.0` instead of crashing the handler. `media_parts` is `split_once('/')`, with a fallback for an entry that has no `/` at all, so a malformed entry quietly matches nothing.

Then, for each type the server can produce, it finds that type's *most specific* matching entry. Specificity is a number: `2` for an exact match, `1` for `TYPE/*`, `0` for `*/*`:

```rust
let specificity = if entry.kind.eq_ignore_ascii_case("*") {
    0u8
} else if !entry.kind.eq_ignore_ascii_case(want_kind) {
    continue;
} else if entry.subkind.eq_ignore_ascii_case("*") {
    1
} else if entry.subkind.eq_ignore_ascii_case(want_subkind) {
    2
} else {
    continue;
};
```

A match with higher specificity replaces the current one, whatever its `q`. That's rule 2: `text/html;q=0.1` beats `*/*;q=0.9` for `text/html`. Only when two matches are equally specific does the higher `q` win.

After that, two small steps. A weight of `0` drops the type (`continue`), which is rule 3: `q=0` means "not acceptable", not "last choice". Then the best candidate is replaced only when a new one's weight is *strictly* greater (`q > best_q`). Because `available` is walked in order, a tie keeps the earlier type, and that's the server's own preference breaking the tie.

The solution collects `entries` into a `Vec` once instead of re-parsing `accept` for every candidate. That matters little with two formats, but it's the habit you want: parse the input once, then work on structured data.

## `decode_chunked` (the "Build" exercise)

```rust
loop {
    let line_end = find_crlf(rest).ok_or(ChunkedDecodeError::UnexpectedEnd)?;
    let size_line = &rest[..line_end];
    rest = &rest[line_end + 2..];
    // ... size_line -> size (hexadecimal) ...
    if size == 0 {
        let ends_cleanly = rest.len() >= 2 && &rest[..2] == b"\r\n";
        return if ends_cleanly { Ok(out) } else { Err(ChunkedDecodeError::UnexpectedEnd) };
    }
    if rest.len() < size + 2 || &rest[size..size + 2] != b"\r\n" {
        return Err(ChunkedDecodeError::UnexpectedEnd);
    }
    out.extend_from_slice(&rest[..size]);
    rest = &rest[size + 2..];
}
```

(Shortened: the full version, with the size parsing, is in `src/lib.rs`.)

`rest` is a slice that moves forward through the body. Nothing is copied except the chunk data itself, into `out`. `find_crlf` is used **only** for the size line. For the data, the solution trusts `size` and slices exactly that many bytes. That's what makes the classic test's third chunk (` in\r\n\r\nchunks.`, 14 bytes with two `\r\n` pairs inside it) come out right.

Parsing the size line takes two steps, and each has its own error:

```rust
let size_text = std::str::from_utf8(size_line)
    .map_err(|_| ChunkedDecodeError::InvalidLength(String::from_utf8_lossy(size_line).into_owned()))?;
let size = usize::from_str_radix(size_text, 16)
    .map_err(|_| ChunkedDecodeError::InvalidLength(size_text.to_string()))?;
```

`usize::from_str_radix(text, 16)` parses hexadecimal. Both `"E"` and `"e"` work. The spec allows either, and so does `from_str_radix`. Every out-of-bounds case (a chunk cut short, a missing `\r\n` after the data, a missing terminating chunk) is checked *before* slicing. So a malformed body is an `Err`, never an index-out-of-bounds panic.

## On the challenge (optional)

The smallest change is in how each entry is parsed. Split it on `;`, trim every piece, take the first piece as the media type, and look through the rest for one that starts with `q=`:

```rust
let mut parts = entry.split(';').map(str::trim);
let media = parts.next().unwrap_or("");
let q = parts
    .find_map(|param| param.strip_prefix("q="))
    .map(|weight| weight.trim().parse().unwrap_or(1.0))
    .unwrap_or(1.0);
```

`text/html ; q=0.5` and `text/html;level=1;q=0.5` both give `("text/html", 0.5)`. Other parameters, like `level=1`, are ignored. The spec lets them make a match *more* specific, but no server you'll write in this course needs that.

## What this lesson was really about

None of these functions is hard to write. What's hard is knowing the rule each one encodes. `axum` will pick your status line's reason phrase, keep connections alive, and chunk a streaming body for you. It won't pick your status code or your method, or decide whether a handler is safe to retry. Those choices stay yours for the rest of the course.
