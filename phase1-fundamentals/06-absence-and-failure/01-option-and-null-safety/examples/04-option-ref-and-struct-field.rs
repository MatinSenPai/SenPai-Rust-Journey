//! `Option<&T>` and `&Option<T>` are not the same type, even though they're
//! built from the same two words. And `Option<T>` as a struct field is how
//! "this field is optional" gets said in Rust.
//!
//!     cargo run -p p1-06-01-option-and-null-safety --example 04-option-ref-and-struct-field

// `&Option<T>` — a reference to the whole box, `Some` or `None` and all.
fn describe_ref(nickname: &Option<String>) -> String {
    match nickname {
        Some(name) => format!("&Option<T>:    Some(\"{name}\")"),
        None => "&Option<T>:    None".to_string(),
    }
}

// `Option<&T>` — a box that, if it has anything in it, has a reference.
fn describe_inner(nickname: Option<&String>) -> String {
    match nickname {
        Some(name) => format!("Option<&T>:    Some(\"{name}\")"),
        None => "Option<&T>:    None".to_string(),
    }
}

struct Profile {
    nickname: Option<String>,
}

impl Profile {
    // `.as_ref()` turns the `&Option<String>` you get from `&self.nickname`
    // into an `Option<&String>` you can match on — without moving the field
    // out of `self`, which a plain-value match would try to do.
    fn nickname_len(&self) -> Option<usize> {
        match self.nickname.as_ref() {
            Some(name) => Some(name.len()),
            None => None,
        }
    }
}

fn main() {
    let known = Some(String::from("Matin"));
    let unknown: Option<String> = None;

    println!("{}", describe_ref(&known));
    println!("{}", describe_ref(&unknown));

    // Same value, viewed through `.as_ref()`: `known` is still ours after.
    println!();
    println!("{}", describe_inner(known.as_ref()));
    println!("still have `known`: {known:?}");

    // `.first()` already hands back `Option<&T>` — no `.as_ref()` needed to
    // get one; `.cloned()` is how you turn that borrowed look into an owned
    // value, the same `.cloned()` from 1.2.3.
    let words = vec![String::from("hello"), String::from("world")];
    let borrowed: Option<&String> = words.first();
    let owned: Option<String> = borrowed.cloned();
    println!();
    println!("borrowed: {borrowed:?}");
    println!("owned:    {owned:?}");

    // `Option<T>` as a struct field: the field itself says "might not be
    // there" — no sentinel string, no magic number.
    let matin = Profile {
        nickname: Some(String::from("Matin")),
    };
    let anon = Profile { nickname: None };

    println!();
    println!("matin.nickname_len(): {:?}", matin.nickname_len());
    println!("anon.nickname_len():  {:?}", anon.nickname_len());
    // Called twice — proof `nickname_len` never took ownership of the field.
    println!("matin.nickname_len(): {:?}", matin.nickname_len());
}
