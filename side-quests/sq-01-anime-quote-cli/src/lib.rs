pub struct Quote {
    pub anime: String,
    pub character: String,
    pub text: String,
}

/// The built-in dataset. Already implemented — add your own favorites once
/// the rest of this file is working, if you like.
pub fn all_quotes() -> Vec<Quote> {
    let raw = [
        ("One Piece", "Monkey D. Luffy", "I don't want to conquer anything. I just think the guy with the most freedom in this whole ocean... is the Pirate King!"),
        ("One Piece", "Roronoa Zoro", "Nothing happened."),
        ("Attack on Titan", "Eren Yeager", "If you win, you live. If you lose, you die. If you don't fight, you can't win."),
        ("Fullmetal Alchemist", "Edward Elric", "A lesson without pain is meaningless. That's because no one can gain without sacrificing something."),
        ("Naruto", "Naruto Uzumaki", "I'm not gonna run away, I never go back on my word! That's my nindo: my ninja way!"),
        ("Death Note", "L", "I am justice."),
        ("Fullmetal Alchemist", "Alphonse Elric", "There's no such thing as a painless lesson. They just don't exist."),
        ("Attack on Titan", "Mikasa Ackerman", "The world is cruel, and also very beautiful."),
        ("Chainsaw Man", "Denji", "I want to live a normal, happy life."),
        ("Vinland Saga", "Thors Snorrason", "You have no enemies, nobody does. There is no such thing in this world."),
    ];

    raw.into_iter()
        .map(|(anime, character, text)| Quote {
            anime: anime.to_string(),
            character: character.to_string(),
            text: text.to_string(),
        })
        .collect()
}

/// Returns every quote whose `anime` field contains `query`, matched
/// case-insensitively (so `"one piece"` matches `"One Piece"`).
///
/// You'll notice `<'a>` here — an explicit **lifetime annotation**, new
/// syntax. Phase 2 covers lifetimes properly; for now, just read it as
/// "the `&Quote`s in the returned `Vec` are borrowed from `quotes`, and stay
/// valid for exactly as long as `quotes` does." It's required here (unlike
/// earlier lessons' single-reference-in-single-reference-out functions,
/// which the compiler could infer automatically) because this function has
/// *two* reference parameters (`quotes` and `query`), and the compiler
/// needs to be told explicitly which one the output actually borrows from.
pub fn find_by_anime<'a>(quotes: &'a [Quote], query: &str) -> Vec<&'a Quote> {
    todo!("filter quotes where q.anime.to_lowercase().contains(&query.to_lowercase())")
}

/// Same idea as `find_by_anime`, but matching against `character` instead.
pub fn find_by_character<'a>(quotes: &'a [Quote], query: &str) -> Vec<&'a Quote> {
    todo!("filter quotes where q.character.to_lowercase().contains(&query.to_lowercase())")
}

/// Formats a quote as: `"text" — Character, Anime`
pub fn format_quote(q: &Quote) -> String {
    todo!("format!(\"\\\"{{}}\\\" — {{}}, {{}}\", q.text, q.character, q.anime)")
}

/// Picks a uniformly random quote from `quotes`, or `None` if it's empty.
pub fn pick_random(quotes: &[Quote]) -> Option<&Quote> {
    todo!("if quotes.is_empty() return None, else pick a random index with rand::thread_rng().gen_range(..)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_by_anime_case_insensitively() {
        let quotes = all_quotes();
        let results = find_by_anime(&quotes, "one piece");
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|q| q.anime == "One Piece"));
    }

    #[test]
    fn finds_by_anime_returns_empty_for_no_match() {
        let quotes = all_quotes();
        assert!(find_by_anime(&quotes, "Bleach").is_empty());
    }

    #[test]
    fn finds_by_character_case_insensitively() {
        let quotes = all_quotes();
        let results = find_by_character(&quotes, "luffy");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].character, "Monkey D. Luffy");
    }

    #[test]
    fn formats_a_quote() {
        let q = Quote {
            anime: "Test Anime".to_string(),
            character: "Test Character".to_string(),
            text: "A test quote.".to_string(),
        };
        assert_eq!(
            format_quote(&q),
            "\"A test quote.\" — Test Character, Test Anime"
        );
    }

    #[test]
    fn picks_random_from_nonempty() {
        let quotes = all_quotes();
        let picked = pick_random(&quotes);
        assert!(picked.is_some());
        assert!(quotes.iter().any(|q| q.text == picked.unwrap().text));
    }

    #[test]
    fn picks_none_from_empty() {
        let empty: Vec<Quote> = vec![];
        assert!(pick_random(&empty).is_none());
    }
}
