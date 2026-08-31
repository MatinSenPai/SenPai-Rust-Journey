//! `Send`/`Sync` need no `impl` block and no `#[derive]` — the compiler
//! computes both, automatically, from a type's fields.

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

// For any `T`: if `T` is `Sync`, `&T` is `Send`. That's the whole definition
// of `Sync`, and it holds for every type, not just this lesson's example.
fn shared_ref_is_send_when_sync<T: Sync>() {
    assert_send::<&T>();
}

struct Ticket {
    id: u32,
    title: String,
}

fn main() {
    let ticket = Ticket {
        id: 7,
        title: "senpai-api outage".to_string(),
    };

    // No `impl Send for Ticket`, no `#[derive(Send)]` — neither is legal
    // syntax. `Ticket` is `Send` and `Sync` purely because `u32` and
    // `String` both are, and the compiler worked that out on its own.
    assert_send::<Ticket>();
    assert_sync::<Ticket>();
    shared_ref_is_send_when_sync::<Ticket>();

    println!(
        "Ticket #{} ({}) is Send and Sync; so is &Ticket",
        ticket.id, ticket.title
    );
}
