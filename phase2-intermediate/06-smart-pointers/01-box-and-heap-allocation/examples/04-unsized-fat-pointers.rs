//! When `T` is unsized — `str`, or `dyn Trait` — `Box<T>` becomes a fat
//! pointer, two words instead of one. Still fixed, still not a function of
//! *which* `str` or *which* concrete type is inside.

use std::mem::size_of;

trait Playable {
    fn play(&self) -> String;
}

struct Song {
    title: String,
}

impl Playable for Song {
    fn play(&self) -> String {
        format!("playing song: {}", self.title)
    }
}

struct Podcast {
    title: String,
    duration_minutes: u32,
}

impl Playable for Podcast {
    fn play(&self) -> String {
        format!(
            "playing podcast: {} ({} min)",
            self.title, self.duration_minutes
        )
    }
}

fn main() {
    println!("--- Box<str>: no capacity word, same shape as &str ---");
    let owned_str: Box<str> = "senpai".into();
    println!("size_of::<String>()   = {}", size_of::<String>());
    println!("size_of::<Box<str>>() = {}", size_of::<Box<str>>());
    println!("size_of::<&str>()     = {}", size_of::<&str>());
    println!("owned_str = {owned_str}");

    println!("--- Box<dyn Playable>: fixed, whichever concrete type is inside ---");
    println!("size_of::<Song>()    = {}", size_of::<Song>());
    println!("size_of::<Podcast>() = {}", size_of::<Podcast>());
    println!(
        "size_of::<Box<dyn Playable>>() = {}",
        size_of::<Box<dyn Playable>>()
    );

    let playlist: Vec<Box<dyn Playable>> = vec![
        Box::new(Song {
            title: "Intro".to_string(),
        }),
        Box::new(Podcast {
            title: "Deep Dive".to_string(),
            duration_minutes: 42,
        }),
    ];
    for item in &playlist {
        println!("{}", item.play());
    }
}
