//! Exercises for 1.4.1 — `String` versus `&str`.
//!
//! Look at each signature before you write anything. Three of these take a
//! view and hand back an owner; one takes an owner because it is going to
//! grow it. Which one is which is the lesson.

/// How many **bytes** `text` occupies.
///
/// This is a view in, a number out — nothing is copied and nothing is
/// allocated. It has to accept a literal, a `&str`, and a `&String` alike.
///
/// # Examples
///
/// `byte_length("hello")` is 5.
/// `byte_length("سلام")` is 8 — two bytes per Persian letter.
/// `byte_length("")` is 0.
pub fn byte_length(text: &str) -> usize {
    todo!("give back the number of bytes `text` occupies")
}

/// `text` in upper case, as new owned text.
///
/// A view goes in and an owner comes out, because the answer is text that
/// did not exist before and has to live somewhere after this returns.
/// Persian has no upper case, so Persian text comes back unchanged.
///
/// # Examples
///
/// `shout("senpai")` is `"SENPAI"`.
/// `shout("Matin")` is `"MATIN"`.
/// `shout("سلام")` is `"سلام"`.
/// `shout("")` is `""`.
pub fn shout(text: &str) -> String {
    todo!("give back an owned upper-case version of `text`")
}

/// `first` and `second` one after the other, with nothing between them, as
/// one owned `String`.
///
/// Reserve the room up front: the answer's capacity must come out **equal to
/// its length**, so the buffer is asked for once and never grown.
///
/// # Examples
///
/// `joined("سلام", " دنیا")` is `"سلام دنیا"`, whose length and capacity are
/// both 17.
/// `joined("a", "b")` is `"ab"`, length and capacity both 2.
/// `joined("", "")` is `""`, length and capacity both 0.
pub fn joined(first: &str, second: &str) -> String {
    todo!("build one owned string holding both parts, with no room to spare")
}

/// `text` with `extra` on the end of it.
///
/// This one takes an owner rather than a view, and that is deliberate: when
/// `text` already has room for `extra`, the answer must be **the same buffer
/// `text` was using**, not a fresh one. A caller who hands you a `String`
/// they are finished with should not pay for a second allocation.
///
/// # Examples
///
/// `extended("سلام".to_string(), " دنیا")` is `"سلام دنیا"`.
/// `extended(String::new(), "hi")` is `"hi"`.
/// `extended("hi".to_string(), "")` is `"hi"`.
pub fn extended(text: String, extra: &str) -> String {
    todo!("put `extra` on the end of `text` and give the result back")
}

/// An owned copy of every item in `items`, in the same order.
///
/// `items` is a list of views; the answer is a list of owners. That means one
/// allocation per element — worth noticing, because nothing in the code says
/// so out loud.
///
/// # Examples
///
/// `all_owned(&["a", "b"])` is `["a", "b"]` as two `String`s.
/// `all_owned(&["سلام"])` is `["سلام"]`, whose one element is 8 bytes long.
/// `all_owned(&[])` is `[]`.
pub fn all_owned(items: &[&str]) -> Vec<String> {
    todo!("give back an owned copy of every item, in order")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_bytes_from_every_kind_of_caller() {
        assert_eq!(byte_length("hello"), 5);
        assert_eq!(byte_length(""), 0);

        // Persian is two bytes a letter, and this is where the two numbers
        // stop agreeing.
        assert_eq!(byte_length("سلام"), 8);

        // The same function, called with an owner and with a view of it.
        let owned = "سلام دنیا".to_string();
        assert_eq!(byte_length(&owned), 17);
        assert_eq!(byte_length(owned.as_str()), 17);
    }

    #[test]
    fn shouts_latin_and_leaves_persian_alone() {
        assert_eq!(shout("senpai"), "SENPAI");
        assert_eq!(shout("Matin"), "MATIN");
        assert_eq!(shout("سلام"), "سلام");
        assert_eq!(shout(""), "");

        // A `&String` where a `&str` is wanted, and it just works.
        let owned = "rust".to_string();
        assert_eq!(shout(&owned), "RUST");
    }

    #[test]
    fn joins_with_no_room_to_spare() {
        let persian = joined("سلام", " دنیا");
        assert_eq!(persian, "سلام دنیا");
        assert_eq!(persian.len(), 17);
        assert_eq!(persian.capacity(), persian.len(), "ask for the room once");

        let latin = joined("a", "b");
        assert_eq!(latin, "ab");
        assert_eq!(latin.capacity(), 2);

        let empty = joined("", "");
        assert_eq!(empty, "");
        assert_eq!(empty.capacity(), 0);
    }

    #[test]
    fn extends_without_taking_a_second_buffer() {
        let mut roomy = String::with_capacity(64);
        roomy.push_str("سلام");
        let was_at = roomy.as_ptr();

        let grown = extended(roomy, " دنیا");
        assert_eq!(grown, "سلام دنیا");
        assert_eq!(
            grown.as_ptr(),
            was_at,
            "there was room already — the buffer should have been reused"
        );

        assert_eq!(extended(String::new(), "hi"), "hi");
        assert_eq!(extended("hi".to_string(), ""), "hi");
    }

    #[test]
    fn owns_every_element() {
        assert_eq!(all_owned(&["a", "b"]), vec!["a", "b"]);
        assert_eq!(all_owned(&[]), Vec::<String>::new());

        let persian = all_owned(&["سلام"]);
        assert_eq!(persian, vec!["سلام"]);
        assert_eq!(persian[0].len(), 8);

        // Each one owns its own buffer, so changing one leaves the rest alone.
        let mut made = all_owned(&["x", "x"]);
        made[0].push('!');
        assert_eq!(made[0], "x!");
        assert_eq!(made[1], "x");
    }
}
