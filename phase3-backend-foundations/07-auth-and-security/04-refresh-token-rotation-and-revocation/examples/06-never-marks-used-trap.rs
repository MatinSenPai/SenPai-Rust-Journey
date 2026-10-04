//! No error at all: a `rotate` that forgets to mark the old token as used.
//! It compiles and runs, and a stolen token works forever.

use std::collections::HashMap;

struct NaiveService {
    tokens: HashMap<String, String>, // token -> user
    next: u32,
}

impl NaiveService {
    fn login(&mut self, user: &str) -> String {
        self.next += 1;
        let token = format!("token-{}", self.next);
        self.tokens.insert(token.clone(), user.to_string());
        token
    }

    fn rotate(&mut self, presented: &str) -> Option<String> {
        let user = self.tokens.get(presented)?.clone();
        // BUG: the presented token is never invalidated or marked used.
        Some(self.login(&user))
    }
}

fn main() {
    let mut svc = NaiveService {
        tokens: HashMap::new(),
        next: 0,
    };
    let original = svc.login("matin");
    let client_copy = svc.rotate(&original);
    let thief_copy = svc.rotate(&original);
    println!("client rotates: {client_copy:?}");
    println!("thief replays:  {thief_copy:?}  (should have been refused)");
}
