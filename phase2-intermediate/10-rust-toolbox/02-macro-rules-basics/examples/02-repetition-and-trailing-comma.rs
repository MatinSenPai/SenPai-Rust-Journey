//! Repetition (`$( ... ),+`) replaying a captured fragment once per matched
//! item, plus the `$(,)?` idiom that accepts an optional trailing comma.

macro_rules! string_vec {
    () => {
        Vec::<String>::new()
    };
    ( $( $s:expr ),+ $(,)? ) => {{
        let mut v = Vec::new();
        $( v.push($s.to_string()); )+
        v
    }};
}

fn main() {
    let empty: Vec<String> = string_vec![];
    let names = string_vec!["hero", "mage", "sage"];
    let trailing = string_vec!["one", "two",];
    println!("empty:    {empty:?}");
    println!("names:    {names:?}");
    println!("trailing: {trailing:?}");
}
