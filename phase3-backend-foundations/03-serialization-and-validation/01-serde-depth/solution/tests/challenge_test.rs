//! Challenge rung: `#[serde(with = "genre_list")]`.

use p3_03_01_serde_depth_solution::AnimeGenres;

fn show(genres: &[&str]) -> AnimeGenres {
    AnimeGenres {
        title: "Frieren".into(),
        genres: genres.iter().map(|g| g.to_string()).collect(),
    }
}

#[test]
fn genres_are_written_as_one_comma_separated_string() {
    assert_eq!(
        serde_json::to_string(&show(&["fantasy", "adventure"])).unwrap(),
        r#"{"title":"Frieren","genres":"fantasy,adventure"}"#
    );
}

#[test]
fn no_genres_is_an_empty_string() {
    assert_eq!(
        serde_json::to_string(&show(&[])).unwrap(),
        r#"{"title":"Frieren","genres":""}"#
    );
}

#[test]
fn reading_trims_spaces_and_drops_empty_pieces() {
    let a: AnimeGenres =
        serde_json::from_str(r#"{"title":"Frieren","genres":" fantasy , adventure,,"}"#).unwrap();
    assert_eq!(a, show(&["fantasy", "adventure"]));
}

#[test]
fn an_empty_string_is_no_genres() {
    let a: AnimeGenres = serde_json::from_str(r#"{"title":"Frieren","genres":""}"#).unwrap();
    assert_eq!(a, show(&[]));
}

#[test]
fn an_array_is_not_accepted() {
    assert!(serde_json::from_str::<AnimeGenres>(r#"{"title":"F","genres":["a"]}"#).is_err());
}
