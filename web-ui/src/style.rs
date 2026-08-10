//! The entire stylesheet, served from `/style.css`.
//!
//! Compiled into the binary as a `const` so there's no build step, no asset
//! directory, and no CDN — `cargo run -p course-ui` is the whole install.

pub const CSS: &str = r#"
:root {
  --bg: #000;
  --fg: #fff;
  --dim: #888;
  --line: #333;
  --panel: #0a0a0a;
  /* Body copy sits a shade back from the headings so structure reads first.
     Scoped to `main` only — the sidebar stays pure white. */
  --prose: #d6d3d1;
  --link: #60a5fa;
  --link-hover: #93c5fd;
}

* { box-sizing: border-box; }

html { scroll-padding-top: 1rem; }

body {
  margin: 0;
  background: var(--bg);
  color: var(--fg);
  font: 16px/1.65 ui-sans-serif, system-ui, -apple-system, "Helvetica Neue", sans-serif;
}

a { color: var(--fg); text-decoration: underline; text-underline-offset: 2px; }
a:hover { background: var(--fg); color: var(--bg); text-decoration: none; }

.layout { display: flex; align-items: flex-start; }

/* ---- sidebar ---- */

nav {
  position: sticky;
  top: 0;
  flex: 0 0 22rem;
  height: 100vh;
  overflow-y: auto;
  padding: 1.5rem 1rem 4rem;
  border-right: 1px solid var(--line);
  font-size: 14px;
}

nav .home { display: block; margin-bottom: 1rem; font-weight: 600; }

nav ul { list-style: none; margin: 0; padding-left: 0.85rem; }
nav li { margin: 0.15rem 0; }

nav details > summary {
  cursor: pointer;
  list-style: none;
  padding: 0.1rem 0;
}
nav details > summary::-webkit-details-marker { display: none; }
nav details > summary::before {
  content: "+";
  display: inline-block;
  width: 1rem;
  color: var(--dim);
}
nav details[open] > summary::before { content: "\2212"; }

nav a { text-decoration: none; }
nav a:hover { background: none; color: var(--fg); text-decoration: underline; }
nav .current > a, nav .current > summary { font-weight: 700; }
nav .count { color: var(--dim); font-size: 12px; margin-left: 0.35rem; }

/* completed: checkmark + line through, everywhere it appears */
.done > a,
.done > summary > a,
a.done,
li.done > .label { text-decoration: line-through; color: var(--dim); }
.done-mark { color: var(--fg); margin-right: 0.3rem; }

/* ---- content ---- */

main {
  flex: 1 1 auto;
  min-width: 0;
  max-width: 52rem;
  padding: 2.5rem 3rem 6rem;
  color: var(--prose);
}

/* Links in the prose are blue; the sidebar's stay white (see `nav a` above). */
main a { color: var(--link); }
main a:hover { background: none; color: var(--link-hover); text-decoration: underline; }

.crumbs { color: var(--dim); font-size: 13px; margin-bottom: 1.5rem; }
.crumbs a { color: var(--dim); text-decoration: none; }
.crumbs a:hover { color: var(--fg); background: none; text-decoration: underline; }

/* Headings stay full white so the page structure still reads at a glance. */
main h1, main h2, main h3, main h4, main h5, main h6 { color: var(--fg); }
h1, h2, h3, h4 { line-height: 1.25; margin: 2rem 0 0.75rem; }
h1 { font-size: 1.9rem; margin-top: 0; }
h2 { font-size: 1.35rem; border-bottom: 1px solid var(--line); padding-bottom: 0.35rem; }
h3 { font-size: 1.1rem; }

p, ul, ol, blockquote, table { margin: 0.9rem 0; }
li { margin: 0.25rem 0; }

blockquote {
  margin-left: 0;
  padding-left: 1rem;
  border-left: 2px solid var(--line);
  color: var(--dim);
}

code {
  font: 0.88em ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 3px;
  padding: 0.1em 0.35em;
}

pre {
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 4px;
  padding: 1rem;
  overflow-x: auto;
}
pre code { background: none; border: 0; padding: 0; font-size: 0.85rem; }

code.inert { color: var(--dim); }

table { border-collapse: collapse; width: 100%; font-size: 0.92rem; }
th, td { border: 1px solid var(--line); padding: 0.45rem 0.7rem; text-align: left; }
th { background: var(--panel); }

hr { border: 0; border-top: 1px solid var(--line); margin: 2.5rem 0; }

img { max-width: 100%; }

input[type="checkbox"] { accent-color: #fff; }

/* ---- page sections ---- */

.page + .page { margin-top: 3rem; padding-top: 2rem; border-top: 1px solid var(--line); }

.reveal { margin-top: 3rem; border: 1px solid var(--line); border-radius: 4px; padding: 0 1rem; }
.reveal > summary {
  cursor: pointer;
  padding: 1rem 0;
  color: var(--dim);
}
.reveal[open] > summary { border-bottom: 1px solid var(--line); margin-bottom: 1rem; }

/* ---- children index ---- */

.children { list-style: none; padding-left: 0; }
.children li { margin: 0.4rem 0; }

/* ---- mark complete ---- */

.mark {
  margin-top: 3.5rem;
  padding-top: 2rem;
  border-top: 1px solid var(--line);
}
.mark button {
  font: inherit;
  background: var(--bg);
  color: var(--fg);
  border: 1px solid var(--fg);
  border-radius: 4px;
  padding: 0.6rem 1.1rem;
  cursor: pointer;
}
.mark button:hover { background: var(--fg); color: var(--bg); }
.mark .state { color: var(--dim); margin-left: 0.75rem; font-size: 0.9rem; }

.notice { color: var(--dim); font-size: 0.9rem; }
"#;
