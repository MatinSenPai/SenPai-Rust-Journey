# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain each idea using a real example
from this repo, not whether a test passes.

1. "HTTPS is just encryption" is the popular one-line summary of TLS. Name
   the two properties TLS provides beyond confidentiality, and for each one,
   describe a concrete attack that a *confidentiality-only* protocol
   (traffic is encrypted, but nothing else is checked) would still be
   vulnerable to.

2. Walk through the TLS handshake in order, from "client opens a connection"
   to "the first byte of the actual HTTP request is sent." At which specific
   step would the handshake abort if an attacker tried a man-in-the-middle
   attack by presenting their own certificate instead of the real server's,
   and what specifically about that step catches it?

3. Explain why TLS doesn't just use asymmetric encryption (RSA-style) for
   the entire session instead of switching to symmetric encryption after the
   handshake. What specific property of asymmetric cryptography makes it
   the wrong tool for encrypting bulk data (a large file download, a long
   HTTP response body), and what property makes it exactly the right tool
   for the handshake's key exchange step?

4. The root `Cargo.toml` in this repo includes `sqlx`'s `tls-rustls`
   feature, but your local sandbox Postgres connections don't actually
   perform a meaningful TLS handshake in practice. Explain why that's fine
   for local development, and describe the specific circumstance under which
   `taskforge-storage`'s database connection would start actually needing
   that feature to do real work.

5. A certificate authority (CA) signs a server's certificate. Explain, in
   your own words, why a client that has never communicated with
   `yourbank.com` before can still trust that it's really talking to
   `yourbank.com` — what is the client actually trusting, if not the server
   itself?
