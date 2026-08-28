//! Static dispatch: `announce<T: Summarize>` gets monomorphized exactly as
//! 2.3.2 taught — one separate compiled copy of the function per concrete
//! `T` actually used. Proof: the two instantiations do not even share an
//! address.
//!
//!     cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 01-static-two-compiled-functions

trait Summarize {
    fn summary(&self) -> String;
}

struct AnimeSeries {
    title: String,
    episodes: u32,
}

impl Summarize for AnimeSeries {
    fn summary(&self) -> String {
        format!("{} - {} episodes", self.title, self.episodes)
    }
}

struct MangaVolume {
    title: String,
    chapters: u32,
}

impl Summarize for MangaVolume {
    fn summary(&self) -> String {
        format!("{} - {} chapters", self.title, self.chapters)
    }
}

fn announce<T: Summarize>(item: &T) -> String {
    format!("now: {}", item.summary())
}

fn main() {
    let anime = AnimeSeries {
        title: "Trigun".to_string(),
        episodes: 26,
    };
    let manga = MangaVolume {
        title: "Blame!".to_string(),
        chapters: 10,
    };

    println!("{}", announce(&anime));
    println!("{}", announce(&manga));

    // Two different concrete `T`s at two call sites. If the compiler really
    // built two separate functions, they live at two separate addresses.
    #[allow(function_casts_as_integer)]
    let (anime_fn, manga_fn) = (
        announce::<AnimeSeries> as usize,
        announce::<MangaVolume> as usize,
    );
    println!("same compiled function? {}", anime_fn == manga_fn);
}
