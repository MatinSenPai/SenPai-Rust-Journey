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

On recall question 1: `fn longest<'a>(a: &'a str, b: &str) -> &'a str`
**does not** compile with this implementation. `b` gets an independent
lifetime, so the compiler cannot prove that a reference borrowed from `b`
will remain valid for all of `'a`. Returning `b` from the second branch would
violate the signature's promise. Tying both inputs to `'a` supplies the
required relationship; it does not make either input live longer.
