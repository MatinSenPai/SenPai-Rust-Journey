# Module 7 — Auth & security

Django gave you `User`, sessions, and permissions for free. This module
builds the same guarantees by hand: hashing a password so a leaked database
doesn't leak plaintext credentials, the real trade-off between sessions and
tokens, and a permissions model that actually holds up — with OWASP's
concerns threaded through each lesson rather than bolted on as an
afterthought at the end.

1. [01 — Password hashing with `argon2`](01-password-hashing-argon2/README.md)
2. [02 — Sessions vs. JWT: the real trade-off](02-sessions-vs-jwt/README.md)
   — stateful vs. stateless auth, and which problem each one actually
   solves, before you commit to either.
3. [03 — JWTs and `tower` middleware](03-jwt-and-tower-middleware/README.md)
4. [04 — Refresh-token rotation and revocation](04-refresh-token-rotation-and-revocation/README.md)
   — the part a JWT tutorial usually skips: what happens when a token
   needs to stop being valid before it expires.
5. [05 — Modelling RBAC and permissions](05-modelling-rbac-and-permissions/README.md)
   — roles, permissions, and a schema for "can this user do that" which
   survives a real product's growth.
