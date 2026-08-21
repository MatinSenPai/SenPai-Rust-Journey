//! The whole Phase 1 toolkit, used together in one small program.
//!
//! Nothing here is new: an ownership decision, a slice, `Option` instead of
//! a sentinel, `Result` instead of a panic, and a `match`. The same ideas
//! from thirty lessons ago, all in one place.
//!
//! Run: cargo run -p p1-07-02-phase-review --example 01-the-whole-toolkit

#[derive(Debug)]
enum Ticket {
    Open,
    Closed { resolution: String },
}

/// `tickets` is consumed, not lent back — the caller handed it over because
/// it wanted the closed-out version, not its old copy. See 1.2.4.
fn close_all(tickets: Vec<Ticket>, resolution: &str) -> Vec<Ticket> {
    let mut closed = Vec::with_capacity(tickets.len());
    for ticket in tickets {
        let outcome = match ticket {
            Ticket::Open => Ticket::Closed {
                resolution: resolution.to_string(),
            },
            already @ Ticket::Closed { .. } => already,
        };
        closed.push(outcome);
    }
    closed
}

/// A borrowed look at the tail of the slice. No allocation, nothing moved.
/// See 1.3.4.
fn most_recent(tickets: &[Ticket], count: usize) -> &[Ticket] {
    let start = tickets.len().saturating_sub(count);
    &tickets[start..]
}

/// `None` when there is nothing to report — not a made-up ticket, not `-1`.
/// See 1.6.1.
fn first_open(tickets: &[Ticket]) -> Option<&Ticket> {
    for ticket in tickets {
        if let Ticket::Open = ticket {
            return Some(ticket);
        }
    }
    None
}

/// A failure the caller can act on, not a crash. See 1.6.3.
fn parse_ticket_count(input: &str) -> Result<u32, String> {
    input
        .trim()
        .parse::<u32>()
        .map_err(|_| format!("not a ticket count: {input:?}"))
}

fn main() {
    let tickets = vec![
        Ticket::Open,
        Ticket::Closed {
            resolution: "duplicate".to_string(),
        },
        Ticket::Open,
    ];

    match first_open(&tickets) {
        Some(_) => println!("there is still an open ticket"),
        None => println!("everything is closed"),
    }

    let recent = most_recent(&tickets, 2);
    println!("last {} tickets: {recent:?}", recent.len());

    let closed = close_all(tickets, "resolved in review");
    for ticket in &closed {
        match ticket {
            Ticket::Closed { resolution } => println!("closed: {resolution}"),
            Ticket::Open => println!("still open"),
        }
    }

    match parse_ticket_count("4") {
        Ok(n) => println!("parsed: {n}"),
        Err(message) => println!("could not parse: {message}"),
    }

    match parse_ticket_count("۴") {
        Ok(n) => println!("parsed: {n}"),
        Err(message) => println!("could not parse: {message}"),
    }
}
