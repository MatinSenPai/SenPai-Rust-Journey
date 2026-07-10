# Checkpoint

1. `a_sender_also_receives_their_own_message_back` documents that this
   design broadcasts to *every* subscriber of a room, including the
   sender's own connection. Is that the right default for a chat app, or
   would you rather the sender not get an echo of their own message? What
   would you need to change in `handle_socket` to exclude the sender —
   and what extra piece of information would that change need access to
   that it doesn't currently have?
2. `messages_do_not_leak_across_rooms` proves two different rooms'
   broadcast channels are independent. Where, concretely, does that
   isolation actually come from — is it something `handle_socket` checks
   explicitly, or a structural consequence of how `ChatServer` stores
   rooms?
3. `broadcast::channel(100)` gives every room a capacity of 100 buffered
   messages. What happens to a subscriber that falls behind by more than
   100 messages (a slow client, a client that stopped reading)? Look up
   what `Receiver::recv()` returns in that case — is it silent data loss,
   or something more informative?
4. Rooms are created lazily and **never removed**, even after every
   participant disconnects. Concretely, what does that cost, and after
   how much usage would it start to matter? Sketch (in words, not code)
   what you'd need to add to `ChatServer` to clean up empty rooms.
5. This lesson's tests bind to `127.0.0.1:0` and let the OS assign a free
   port, rather than a fixed port like `3000`. Why does that specific
   detail matter for a test suite where multiple tests each spin up their
   own server?
