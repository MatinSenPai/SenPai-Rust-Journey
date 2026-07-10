# Checkpoint

1. In your own words: why is "hash it, don't encrypt it" the right call for
   passwords specifically, when for other sensitive data (say, a credit
   card number you need to charge later) encryption *is* the right tool?
   What's different about what you need to *do* with each piece of data
   later?
2. A rainbow table only works against **unsalted** (or identically-salted)
   hashes. Explain exactly why adding a random, per-password salt breaks a
   rainbow table attack — what would an attacker need to rebuild, and why
   is that rebuild as expensive as just brute-forcing each password
   individually?
3. `hashing_the_same_password_twice_produces_different_hashes` asserts
   `first != second` for the *same* input password. Why is that the correct
   and *desired* behavior, rather than a bug — what would it mean for your
   app's security if `hash_password` were deterministic instead?
4. Argon2 is described as "memory-hard." In your own words, what does
   forcing every hash attempt to use real memory actually deny an attacker
   who has built custom cracking hardware (GPUs/ASICs), that a purely
   CPU-slow algorithm like bcrypt does not deny them?
5. `verify_password` returns `bool`, not `Result<bool, SomeError>`, and
   treats "wrong password" and "malformed hash string" identically as
   `false`. Walk through what could go wrong — for your database, or for an
   attacker probing your login endpoint — if a malformed hash instead
   caused `verify_password` to panic.
6. The PHC string returned by `hash_password` embeds its own cost
   parameters (`m=...,t=...,p=...`). Why does storing those parameters
   *with* the hash (rather than as a fixed constant in your application
   code) matter for how you'd eventually raise Argon2's cost settings as
   hardware gets faster, without breaking every existing user's stored
   hash?
