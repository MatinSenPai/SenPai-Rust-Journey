# 05.1 — Encoding vs encryption vs hashing

No code in this lesson. You've already built one hashing scheme
(`phase3-backend-foundations/06-auth-and-security/01-password-hashing-argon2`)
and one signed-token scheme
(`phase3-backend-foundations/06-auth-and-security/02-jwt-and-tower-middleware`)
that quietly depends on the distinction this lesson makes explicit. The goal
here isn't new mechanics — it's the vocabulary triangle: three operations
that get lumped together as "security stuff" in casual conversation, but
that have almost nothing in common once you look at what each one actually
guarantees.

## The three operations, and the one question that tells them apart

Ask one question about any transformation: **can the original be recovered,
and if so, by whom?**

- **Encoding** — reversible by *anyone*. No key, no secret, nothing. Encoding
  is a format transformation, full stop — it exists to make data safe to
  transmit or store in a context that can't handle arbitrary bytes (Base64
  turns arbitrary binary into ASCII; URL-encoding turns reserved characters
  like `?` and `&` into a form that survives being embedded in a URL). It
  provides **zero confidentiality**. If you can encode it, everyone who
  receives it can decode it, instantly, with a one-line library call or a
  website like <https://jwt.io>.
- **Encryption** — reversible, but *only* by whoever holds the right key.
  Symmetric encryption (AES) uses the *same* key to encrypt and decrypt —
  fast, but both parties need that shared key already. Asymmetric encryption
  (RSA) uses a *key pair*: encrypt with the public key, and only the holder
  of the matching private key can decrypt. Slower, but solves the "how do
  two strangers agree on a shared secret without ever having met" problem —
  which, spoiler for the next lesson, is exactly what TLS uses it for.
  Either way, the entire point of encryption is that it's **reversible with
  the key** — that's not a weakness, that's the feature. You *want* the
  ciphertext to turn back into the plaintext, for the intended recipient.
- **Hashing** — **not reversible, ever, by anyone, including the person who
  computed it.** A hash function takes input of any size and produces a
  fixed-size output, and that process deliberately throws information away —
  there is no "decrypt this hash" operation, because a hash isn't a
  scrambled version of the input, it's a fingerprint of it. The only thing
  you can ever do with a hash is compute a *new* hash from a *guess* and
  compare the two outputs. That's it. That's the whole API surface: hash,
  compare, never unhash.

Put them in one line: encoding hides nothing from anyone; encryption hides
something from everyone except the key holder; hashing hides something from
literally everyone, permanently, including you.

## Worked example: three "secure-looking" pieces of data, three different guarantees

Take a single, ordinary login flow and look at three pieces of data flowing
through it, each one going through a different operation:

1. **A password, at signup.** This gets **hashed**
   (`phase3-backend-foundations/06-auth-and-security/01-password-hashing-argon2`'s
   `hash_password`, `argon2id$v=19$m=19456,t=2,p=1$...`). There is no
   function anywhere in that lesson, or anywhere in the `argon2` crate, that
   takes a stored hash and returns the original password. That's not an
   oversight — it's the entire design goal. If your database leaks, the
   attacker has fingerprints, not passwords, and turning a fingerprint back
   into a person requires guessing-and-checking, which Argon2's memory-hard
   cost function makes deliberately, expensively slow (see that lesson's
   "Why Argon2, not bcrypt or plain SHA-256" section). Encrypting the
   password instead — "store it reversibly, just behind a key" — would be
   *strictly worse* here: it would mean some key, somewhere, could turn every
   stored password back into plaintext, which is precisely the capability
   you never want to exist, not even for yourself, not even for "customer
   support needs to help a user recover their account." You never need the
   plaintext back. That's why it's hashed, not encrypted.

2. **A JWT's claims, on every authenticated request after login.** This gets
   **encoded**, not encrypted — and this is the single most common
   misconception a JWT's own shape invites. Look again at
   `phase3-backend-foundations/06-auth-and-security/02-jwt-and-tower-middleware`'s
   breakdown of a token:

   ```
   eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyLTQyIiwiZXhwIjoxNzUyMDgwMDAwfQ.dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk
   └──────── header ────────┘└─────────────── payload ───────────────┘└──────────── signature ────────────┘
   ```

   The header and payload are **base64url-encoded** — reversible by anyone,
   no key required, no secret involved at all. Paste that token into
   <https://jwt.io> right now and its claims appear in plain text instantly.
   That lesson's own README says it outright: "a JWT is signed, not
   encrypted... never put a password, a secret, or anything you wouldn't put
   in a URL query string into a JWT's claims." The *signature* (the third
   segment) is the only part doing anything resembling security work, and
   even that isn't encryption — it's `HMAC-SHA256(header + "." + payload,
   secret)`, which is closer to a keyed hash than to encryption: it proves
   the payload came from your server and hasn't been tampered with, but it
   does **not** hide the payload from anyone holding the token. A `sub` claim
   and an `exp` claim are fine to put in a JWT precisely because they were
   never confidential to begin with — anyone with the token can already read
   them.

   This is also why `jsonwebtoken`'s naming in that lesson's `issue_token`
   and `require_auth` — `EncodingKey::from_secret(...)` and
   `DecodingKey::from_secret(...)` — is a genuine trap for the unwary. Those
   names sound like they're describing encryption (`EncodingKey` /
   `DecodingKey` reads a lot like "the key you encrypt/decrypt with"). They
   aren't. They're the key used to *compute and verify the HMAC signature* —
   `EncodingKey` signs (produces the third segment), `DecodingKey` verifies
   that signature against a freshly recomputed one. Nothing in either
   operation encrypts the header or payload; both remain plain base64url the
   entire time. The crate's own vocabulary is arguably mislabeled relative to
   what's actually happening, which is exactly why it's worth naming
   explicitly here rather than picking it up wrong by osmosis.

3. **The HTTP request itself, in transit.** This gets **encrypted** — TLS
   wraps the entire request (headers, JWT, body, everything) in symmetric
   encryption for the trip across the network, and only the server on the
   other end (holding the corresponding key material from the TLS handshake)
   can decrypt it back to plaintext. This is genuinely reversible-with-a-key
   confidentiality, the thing neither of the other two operations provide.
   The next lesson in this module,
   `phase5-system-design-mastery/05-security-and-auth-at-scale/02-https-and-tls-handshake`,
   is entirely about how that key gets established in the first place.

Three pieces of data, one login flow, three different operations, three
different guarantees — and mixing them up has real consequences: "encoding"
a password (storing it reversibly, thinking that's "encoded so it's safe")
is a security bug; assuming a JWT's payload is confidential because it
"looks encrypted" is a security bug (people have shipped API keys and
internal role flags in JWT claims believing they were hidden — they never
were); and encrypting a password instead of hashing it means a database leak
plus a key leak equals every plaintext password recovered at once.

## The tell: how to spot which one you're looking at

- If there's **no key anywhere** in the operation and everyone agrees on the
  same fixed algorithm (Base64's alphabet, URL-encoding's escape table) —
  it's **encoding**. Ask "is this secret?" If the answer is "no, it's just a
  format," it's encoding.
- If there's a **key**, and the whole point is that the *same* data comes
  back out given that key — it's **encryption**. Ask "can the right party
  get the original back?" If yes, it's encryption (symmetric if one shared
  key, asymmetric if a public/private pair).
- If the output is **fixed-size regardless of input size**, and there is
  **no operation that reverses it** — only "compute it again and compare" —
  it's **hashing**. Ask "can *anyone*, even with a key, get the original
  back?" If the honest answer is "no, never," it's hashing.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
