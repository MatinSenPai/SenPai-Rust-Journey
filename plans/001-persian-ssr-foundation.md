# Plan 001: Ship the Persian-first SSR learning shell

> Drift check: `git diff --stat 80ec4d1..HEAD -- web-ui docs/adr`.
> Stop if the tree/progress contracts no longer match the excerpts described
> below; do not replace the Rust server with a frontend framework.

## Current state and target

`web-ui/src/main.rs` serves `/`, `/style.css`, `/mark`, and a wildcard node.
`page.rs` emits `lang="en"`, English UI strings and a fixed desktop sidebar.
`tree.rs` treats every Markdown file as a page, while `render.rs` rewrites
relative links. Add a typed `Locale`, localized UI text, `/fa` and `/en`
routes, legacy redirects, server-side search, localized Markdown companions,
previous/next navigation, an accessible dashboard and typed `senpai-visual`
fences. Keep `.course-progress.json` version 1.

## Done criteria

- `/` redirects to `/fa/`; `/fa/...` and `/en/...` render the same node.
- Persian pages are RTL, English pages LTR, and code remains LTR.
- `/fa/search?q=مالکیت` returns escaped, useful results.
- Mark-complete and complete-and-next preserve locale and validate lesson paths.
- No JavaScript, CDN, React, Tailwind, or third-party browser request exists.
- Unit tests, clippy and formatting pass.

## Verification

Run the shared gates, then inspect `/fa/`, `/en/`, an ownership lesson and
search at 320, 768, 1024 and 1440 CSS pixels. Confirm keyboard focus, skip-link,
200% zoom and reduced-motion behavior.
