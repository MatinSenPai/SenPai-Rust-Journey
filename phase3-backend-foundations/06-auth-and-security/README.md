# Module 6 — Auth & security

Every API up to this point has been open — anyone who can reach it can call
it. This module closes that gap in the two pieces every real system needs:
proving a password without ever storing it, and proving a request's
identity on every call after that without resending credentials each time.

1. [01 — Password hashing with `argon2`](01-password-hashing-argon2/README.md)
   — never store plaintext, what a salt actually defeats, and why Argon2's
   memory-hardness beats bcrypt/SHA-256 for this specific job.
2. [02 — JWTs and `tower` middleware](02-jwt-and-tower-middleware/README.md)
   — what a JWT actually is (signed, *not* encrypted), issuing one at
   login, and verifying it in an `axum` middleware modeled directly on
   `capstone-taskforge/taskforge-api`'s bearer-token auth.

By the end of this module you'll be able to build a real login flow: hash
and verify a password, then issue and verify a token that gates every
protected route after that.
