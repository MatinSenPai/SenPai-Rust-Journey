# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain each idea using a real example
from this repo, not whether a test passes.

1. Someone on your team says "we should encrypt user passwords instead of
   hashing them, so we can help users who forgot their password recover it."
   Explain, specifically, what's wrong with that plan — what capability
   would encrypting instead of hashing create that you never want to exist,
   and why does `phase3-backend-foundations/06-auth-and-security/01-password-hashing-argon2`'s
   `hash_password` never need to "reverse" a stored hash at all, even for a
   legitimate login?

2. A teammate reviewing a PR says "this JWT payload has the user's role in
   it, that's fine, it's encrypted." Correct them: is a JWT's payload
   encrypted or encoded? What's the one-sentence proof you'd show them
   (something you could actually do with a real JWT, in under 10 seconds)
   that demonstrates it?

3. `jsonwebtoken`'s API in
   `phase3-backend-foundations/06-auth-and-security/02-jwt-and-tower-middleware`
   uses the types `EncodingKey` and `DecodingKey`. Explain why those names
   are a plausible trap for someone learning this for the first time — what
   would a reasonable person assume those types do, and what do they
   actually do instead?

4. Base64 and Argon2 both take arbitrary input and produce output that looks
   like "random-ish characters" to a human glancing at it. Despite that
   surface similarity, one of them is trivially reversible by anyone and the
   other is designed to never be reversible by anyone. Which is which, and
   what's the underlying structural reason one can be undone and the other
   can't?

5. Symmetric encryption (AES) and asymmetric encryption (RSA) are both
   "encryption" by this lesson's definition — reversible, with a key. What's
   the actual difference in *what kind of key(s)* each one uses, and why
   does that difference matter for two parties who have never communicated
   before and share no pre-agreed secret?

6. Fill in the operation (encoding / encryption / hashing) for each, and
   justify each answer in one sentence: (a) `base64::encode` on a JWT
   header, (b) `Argon2::default().hash_password(...)` on a signup password,
   (c) AES encrypting a database backup file before uploading it to
   long-term storage, (d) `HMAC-SHA256` computing a JWT's signature segment.
