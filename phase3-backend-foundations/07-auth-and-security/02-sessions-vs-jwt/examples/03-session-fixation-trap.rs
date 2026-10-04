//! No compiler error, no panic: this login keeps whatever session id the
//! browser arrived with and just marks it as logged in. An attacker who
//! planted that id beforehand (session fixation) is now logged in as the
//! victim.
//!
//!     cargo run -p p3-07-02-sessions-vs-jwt --example 03-session-fixation-trap

use std::collections::HashMap;

/// session id -> logged-in user (None = anonymous session)
type Sessions = HashMap<String, Option<String>>;

fn login(sessions: &mut Sessions, incoming_id: &str, user: &str) -> String {
    // bug: should mint a NEW id here and drop the old one
    sessions.insert(incoming_id.to_string(), Some(user.to_string()));
    incoming_id.to_string()
}

fn main() {
    let mut sessions = Sessions::new();
    // The attacker visits, gets an anonymous id, and plants it in the victim's browser.
    sessions.insert("planted-by-attacker".into(), None);

    let id = login(&mut sessions, "planted-by-attacker", "victim");
    println!("victim logs in; the browser's id is still: {id}");
    println!(
        "attacker uses the id they planted: {:?}",
        sessions["planted-by-attacker"]
    );
}
