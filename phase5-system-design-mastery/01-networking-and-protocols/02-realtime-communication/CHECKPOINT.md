# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can defend a protocol choice for a
concrete system, not recite definitions.

1. Long polling and WebSocket both require your server to hold open one
   connection per connected client. Explain what's actually different about
   the resource commitment between the two, and why "it holds a connection
   open" alone isn't a complete comparison.
2. Explain, specifically, why SSE would be an awkward choice for the "design
   a chat system" lesson (module 7) and why WebSocket would be an awkward,
   over-built choice for a live sports score ticker. What property of each
   use case's data flow drives the answer in each direction?
3. A webhook and long polling both solve "tell me when something happens
   without me polling for it" — but they put the held-open connection (or
   lack of one) on opposite sides. Explain who holds what, in each case, and
   why webhooks don't need a browser-side EventSource/WebSocket equivalent
   at all.
4. `phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
   describes `POST /jobs` retries as a real, unsolved gap in `taskforge-api`.
   Explain how that exact same problem shows up on a webhook receiver, and
   why "the sender promises to only retry once" is not a solution you can
   rely on.
5. Short polling has no held-open connections at all, unlike the other three
   options. Explain why that doesn't make it the "cheapest" option overall —
   what cost does it have that long polling, SSE, and WebSocket don't, and
   under what traffic pattern does that cost actually dominate?
