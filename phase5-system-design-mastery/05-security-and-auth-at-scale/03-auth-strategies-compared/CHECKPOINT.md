# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain each idea using a real example
from this repo, not whether a test passes.

1. `phase3-backend-foundations/06-auth-and-security/02-jwt-and-tower-middleware`'s
   `require_auth` middleware checks exactly two things about an incoming
   token. Name both, and then explain precisely why "revoke this one user's
   token right now" is impossible to implement using only those two checks —
   what question would you need to ask that neither check answers?

2. Rotating the JWT signing secret would technically "revoke" a compromised
   token. Explain why this is rarely an acceptable fix in practice, and then
   describe the short-lived-access-token-plus-refresh-token pattern: which
   part stays fully stateless, which part becomes stateful again, and why
   putting the statefulness only in the refresh flow (rather than on every
   request) is the whole point of the design.

3. A session-based app currently runs on one server. The team scales it to
   three replicas behind a load balancer, with no other changes. Describe
   exactly what breaks for a logged-in user, and connect it to the
   statelessness discussion in
   `phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
   — why does the equivalent JWT-based version of the same app not have this
   problem?

4. A coworker says "our app has 'Sign in with Google,' so we're using OAuth2
   for authentication." Correct the terminology precisely: what does OAuth2
   actually provide, what does OpenID Connect add on top of it, and which of
   the two is actually answering "who is this user"?

5. SSO and OAuth2 are often mentioned in the same breath. Explain the actual
   relationship between them — is SSO a replacement for OAuth2, a
   competitor to it, or something built using it? Give a one-sentence
   example of each that makes the distinction concrete.

6. Fill in a table cell exercise: for each of (a) a banking app where an
   admin must be able to force-logout a compromised account within seconds,
   and (b) a public read-heavy API serving millions of stateless requests
   across dozens of replicas with no admin-triggered revocation
   requirement — would you lean toward session-based auth or JWT-based
   auth, and justify each choice using the tradeoff table from this lesson.
