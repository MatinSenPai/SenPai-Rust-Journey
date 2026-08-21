//! Stage 2 — the store. Three methods, three different ownership decisions,
//! each one visible in the signature before you read a line of the body.
//!
//!     cargo run -p p1-07-01-guided-mini-project --example 02-the-store

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Watching {
        episode: u32,
    },
    #[allow(dead_code)]
    Finished {
        rating: u8,
    },
    Planned,
    #[allow(dead_code)]
    Dropped {
        at: u32,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct Entry {
    title: String,
    status: Status,
}

struct Watchlist {
    entries: Vec<Entry>,
}

impl Watchlist {
    fn new() -> Watchlist {
        Watchlist {
            entries: Vec::new(),
        }
    }

    // `entry: Entry`, not `&Entry`. The list is going to hold this entry
    // for as long as the list exists — that is only possible if the list
    // owns it. Taking a reference here would tie the list's lifetime to
    // whoever called `add`, which is exactly backwards.
    fn add(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    // `Option<&Entry>`, not `Option<Entry>`. A search usually wants to read
    // a field or print something — handing back an owned `Entry` would
    // clone a `String` on every lookup that never needed one. `&self` for
    // the same reason: `find` only looks.
    fn find(&self, title: &str) -> Option<&Entry> {
        for entry in &self.entries {
            if entry.title == title {
                return Some(entry);
            }
        }
        None
    }

    // `Vec<&str>`, not `Vec<String>`. Every title is already owned by an
    // `Entry` inside `self.entries` — cloning each one just to list them
    // would be exactly the "wasteful" loop from 1.2.3.
    fn titles(&self) -> Vec<&str> {
        let mut out = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            out.push(entry.title.as_str());
        }
        out
    }
}

fn main() {
    let mut list = Watchlist::new();

    let bebop = Entry {
        title: String::from("Cowboy Bebop"),
        status: Status::Watching { episode: 9 },
    };
    list.add(bebop);
    // `bebop` is gone from here on — `add` took it. Try `println!("{bebop:?}")`
    // on the next line and you get 1.2.2's E0382 back, on your own type.

    list.add(Entry {
        title: String::from("حمله به تایتان"),
        status: Status::Planned,
    });

    println!("titles: {:?}", list.titles());

    match list.find("Cowboy Bebop") {
        Some(entry) => println!("found:     {entry:?}"),
        None => println!("not found"),
    }
    match list.find("Ghost in the Shell") {
        Some(entry) => println!("found:     {entry:?}"),
        None => println!("not found: Ghost in the Shell"),
    }

    // `find`'s reference cannot outlive `list` — the borrow checker of
    // module 1.3 is doing exactly its job, on a type you wrote yourself.
    let borrowed = list.find("Cowboy Bebop");
    println!("still borrowed from list: {}", borrowed.is_some());
}
