use rand::Rng;

pub struct Quote {
    pub anime: String,
    pub character: String,
    pub text: String,
}

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

pub fn find_by_anime<'a>(quotes: &'a [Quote], query: &str) -> Vec<&'a Quote> {
    let query = query.to_lowercase();
    quotes
        .iter()
        .filter(|q| q.anime.to_lowercase().contains(&query))
        .collect()
}

pub fn find_by_character<'a>(quotes: &'a [Quote], query: &str) -> Vec<&'a Quote> {
    let query = query.to_lowercase();
    quotes
        .iter()
        .filter(|q| q.character.to_lowercase().contains(&query))
        .collect()
}

pub fn format_quote(q: &Quote) -> String {
    format!("\"{}\" — {}, {}", q.text, q.character, q.anime)
}

pub fn pick_random(quotes: &[Quote]) -> Option<&Quote> {
    if quotes.is_empty() {
        return None;
    }
    let idx = rand::thread_rng().gen_range(0..quotes.len());
    quotes.get(idx)
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
