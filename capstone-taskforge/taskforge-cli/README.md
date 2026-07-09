# taskforge-cli

A `clap`-based CLI over `taskforge-api` — the second independent thin
client (alongside `taskforge-admin-bot`) over the exact same HTTP surface,
demonstrating "thin client, thick core": neither client contains any real
domain logic, just HTTP calls and terminal-output formatting.

```sh
export TASKFORGE_API_URL=http://localhost:8080
export TASKFORGE_API_TOKEN=your-token

taskforge enqueue send_email --payload '{"to": "a@b.com"}'
taskforge list --job-type send_email --limit 10
taskforge get <job-id>
taskforge cancel <job-id>
```

## Running the tests

```sh
cargo test -p taskforge-cli
```

`src/format.rs`'s terminal-formatting logic is fully unit-tested with zero
I/O; `src/client.rs`'s HTTP calls need a real running `taskforge-api` and
aren't exercised by the default test suite (same reasoning as
`taskforge-admin-bot`).

## A deliberate bit of duplication

`src/client.rs` here and `taskforge-admin-bot/src/client.rs` are two
separate, small `reqwest`-based clients rather than one shared
`taskforge-client` crate. At ~40-60 lines each, extracting a shared crate
for two call sites would be premature abstraction — worth revisiting if a
third client ever shows up (a web dashboard? a Slack bot?), not before.
