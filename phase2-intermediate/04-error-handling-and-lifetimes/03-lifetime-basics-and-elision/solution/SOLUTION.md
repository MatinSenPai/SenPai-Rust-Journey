# Solution

`longest<'a>(a: &'a str, b: &'a str) -> &'a str` forces both inputs and the
output to share one lifetime — the compiler will only accept this function
at a call site where it can find a single `'a` valid for `a`, `b`, and
however long the caller keeps using the result. That's the whole meaning of
the annotation: not "make these live longer," but "these three are
constrained to the *same* validity window."

`FirstSentence<'a>` needs the parameter on the struct because the compiler
can't ever infer a struct's field lifetimes the way it infers a function's
— there's no "elision rule for structs," full stop, so every struct
holding a reference is explicit. But `as_str(&self) -> &str` doesn't repeat
`<'a>` on the method: elision rule 3 kicks in (a method taking `&self`
elides the output reference's lifetime to `self`'s), and since `self`
already carries `FirstSentence<'a>`'s `'a`, the compiler resolves
`as_str`'s return type to `&'a str` without you writing it again.

On checkpoint question 1: `fn longest<'a>(a: &'a str, b: &str) -> &'a str`
**does** compile — `b` just gets its own independent, unconstrained
lifetime, and the return type is only tied to `a`. This changes the
function's actual behavior contract: the caller is now only guaranteed the
returned reference is valid as long as `a` is (even though the real
implementation might sometimes return `b`) — which would be a genuine bug
if `b` could outlive `a` and get returned; the compiler would catch a
caller trying to use the result past `a`'s lifetime, correctly, even though
at runtime the value might have come from `b`.
