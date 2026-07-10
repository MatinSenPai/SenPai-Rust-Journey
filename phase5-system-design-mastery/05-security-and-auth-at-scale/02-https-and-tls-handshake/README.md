# 05.2 — HTTPS & the TLS handshake

No code in this lesson. Every `https://` URL you've typed and every
`sqlx::PgPool::connect` you've written against a real cloud Postgres instance
already relies on the handshake below happening correctly before a single
byte of your actual data moves. This lesson makes that handshake explicit,
using the encoding/encryption/hashing vocabulary from
`phase5-system-design-mastery/05-security-and-auth-at-scale/01-encoding-encryption-and-hashing`
to describe precisely which operation is doing which job at each step.

## What problem TLS actually solves

"HTTPS is encryption" is the popular one-line summary, and it undersells
what TLS (Transport Layer Security, the protocol underneath the `s` in
`https`) actually guarantees. TLS provides three separate properties, and
losing any one of them breaks the "secure connection" promise even if the
other two hold:

- **Confidentiality** — nobody eavesdropping on the network (a coffee-shop
  Wi-Fi sniffer, a compromised router, an ISP) can read the content of the
  request or response. This is the one everyone thinks of as "encryption,"
  and it is: TLS encrypts the traffic.
- **Integrity** — nobody on the network can *tamper* with the request or
  response in transit without the tampering being detected. Confidentiality
  alone doesn't prevent this — an attacker can sometimes flip encrypted bits
  without knowing what they mean and still cause chaos on the receiving end.
  TLS adds message authentication (conceptually a keyed hash, the same
  family of idea as the JWT signature from the previous lesson, computed
  over the encrypted traffic) so any tampering is detected and the
  connection is aborted instead of silently corrupted.
- **Authentication of the server** — this is the property people forget
  about entirely, and it's arguably the most important one for the classic
  attack it prevents: how do you know the server you just connected to is
  actually `yourbank.com`, and not an attacker sitting between you and the
  real server, quietly relaying (and reading) everything in both directions —
  a **man-in-the-middle attack**? Confidentiality and integrity protect the
  *channel*; server authentication protects *who's on the other end of it*.
  A perfectly encrypted, perfectly tamper-proof connection to an impostor is
  worthless. This is what the certificate chain (below) exists to solve.

"HTTPS is encryption" describes one-third of what TLS does.

## The handshake, conceptually

Before any application data (your actual HTTP request) moves, client and
server run a negotiation called the **TLS handshake**. At a conceptual
level, ignoring version-specific wire details:

1. **Client Hello** — the client (your browser, or `sqlx`'s connection pool)
   opens a connection and says, in effect, "I want to talk TLS, here are the
   TLS versions and cipher suites (encryption algorithms) I support, and
   here's some random data to seed the handshake."

2. **Server Hello + certificate** — the server picks a TLS version and
   cipher suite from what the client offered, and sends back its
   **certificate**: a document that says "I am `yourbank.com`," bundled with
   a public key, and signed by a **Certificate Authority (CA)** — an
   organization (Let's Encrypt, DigiCert, and a handful of others) whose job
   is to verify a domain's ownership before signing a certificate for it.

3. **Certificate validation against a trusted CA** — the client checks that
   signature against a built-in list of CAs it already trusts (shipped with
   the OS or browser). This is the **chain of trust**: the client doesn't
   need to have met `yourbank.com` before, it just needs to trust the CA
   that vouches for it, the same way you'd trust a stranger's ID card
   because you trust the government agency that issued it, not because you
   personally know the stranger. If the certificate is expired, signed by an
   untrusted CA, or doesn't match the domain you're actually connecting to,
   the handshake aborts right here — this is exactly the "browser shows a
   big red warning page" moment, and it's the server-authentication property
   from above doing its job: without this step, a man-in-the-middle could
   just present *its own* certificate and the client would have no way to
   tell the difference.

4. **Key exchange** — now that the client trusts it's talking to the real
   server, both sides need to agree on a **shared secret** to encrypt the
   actual session. This is where **asymmetric cryptography** does its one
   and only job in the whole handshake: it lets two parties who've never met
   and share no prior secret establish one, over a network an attacker might
   be watching the entire time, without ever transmitting the secret itself
   in a form an eavesdropper could use. (Modern TLS mostly uses
   Diffie-Hellman-based key exchange rather than literally RSA-encrypting a
   secret, but the role asymmetric crypto plays — solving the "how do
   strangers agree on a secret in public" problem — is the same idea as the
   RSA public/private key pair from the previous lesson.)

5. **Switch to symmetric encryption for the session** — once both sides hold
   the same shared secret, every subsequent byte of the actual HTTP
   traffic — your request, the server's response, all of it — is encrypted
   with a **symmetric** cipher (AES, typically) using that shared secret.

## Why two phases, not just one

This is the detail worth sitting with rather than memorizing: **why not just
use asymmetric encryption for everything?** Asymmetric crypto already solves
confidentiality — why bother switching to a different algorithm partway
through?

Because asymmetric cryptography is **orders of magnitude slower** than
symmetric cryptography for the same amount of data — the math involved
(large-prime operations for RSA, elliptic-curve operations for
Diffie-Hellman) is inherently far more expensive per byte than AES's simpler,
hardware-accelerated operations. Using it for a handshake's small amount of
setup data (a few kilobytes, once, per connection) is cheap. Using it for an
entire HTTP response body, a video stream, a large file download — every
byte of a real workload — would make TLS impractically slow. So TLS uses
each kind of cryptography for exactly the part it's good at: **asymmetric
crypto to solve the hard problem (agreeing on a secret between strangers,
over an untrusted network, without an existing shared secret) exactly once
per connection, then symmetric crypto to solve the easy problem (bulk
encryption) for every byte after that.** This two-phase structure — expensive
asymmetric operation once, cheap symmetric operation repeatedly — shows up
constantly in cryptographic protocol design once you know to look for it;
TLS is the canonical example.

## Where this repo already touches TLS, even if you haven't seen the handshake fire

Check the root `Cargo.toml`:

```
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "macros", "migrate", "chrono", "uuid"] }
```

The `tls-rustls` feature is there. Every lesson and every `taskforge-storage`
connection in this repo that talks to your **local sandbox Postgres**
doesn't actually need it — a connection from your app to a database running
on `localhost`, or inside the same trusted Docker network, never crosses a
network an attacker could realistically sit on, so there's nothing for TLS's
three properties to protect against in that setup. But the moment
`taskforge-storage`'s `PgPoolOptions::connect(...)` points at a real,
production, cloud-hosted Postgres instance — Amazon RDS, Neon, Supabase,
anything reachable over the public internet — that connection crosses
exactly the kind of untrusted network TLS exists for, and `sqlx` runs
precisely the handshake described above before your first query ever
executes: client hello, server certificate, validation against a trusted CA,
asymmetric key exchange, then a switch to symmetric encryption for every
query and every row of every result set after that. The `tls-rustls` feature
flag is what makes that connection *able* to run this handshake at all — it
compiles in a Rust-native TLS implementation (`rustls`) that `sqlx` uses
instead of speaking plaintext Postgres wire protocol over a raw TCP socket.
Same handshake, same guarantees, whether the client is a browser talking
`https://` to a web server or a connection pool talking encrypted Postgres
wire protocol to a database — TLS doesn't care what application protocol
it's wrapping.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
