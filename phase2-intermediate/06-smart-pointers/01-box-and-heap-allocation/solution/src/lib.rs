/// A classic "cons list": either empty (`Nil`), or one value plus the rest
/// of the list. The rest of the list is boxed — without `Box`, this type
/// would have an infinite size (see the README for the compiler error you'd
/// get without it).
pub enum List {
    Cons(i32, Box<List>),
    Nil,
}

impl List {
    /// Recursively walks the list, summing every value.
    ///
    /// Note this method takes `&self`, not `self` — it borrows the list to
    /// read it, rather than consuming it. Thanks to `Box<T>`'s `Deref`,
    /// `rest.sum()` below works directly on `rest: &Box<List>` with no
    /// manual dereferencing.
    pub fn sum(&self) -> i32 {
        match self {
            List::Cons(val, rest) => val + rest.sum(),
            List::Nil => 0,
        }
    }
}

/// Builds a `List` out of a slice, preserving order (the first element of
/// `items` ends up as the first `Cons` in the list).
///
/// Built from the back: start with `Nil`, then walk `items` in reverse,
/// wrapping the list built so far in one more `Cons` each time. Building
/// front-to-back instead would require either mutating "the last node,"
/// which cons lists don't allow (no back-pointers), or reversing at the
/// end — walking in reverse up front is simpler.
pub fn from_vec(items: &[i32]) -> List {
    let mut list = List::Nil;
    for &item in items.iter().rev() {
        list = List::Cons(item, Box::new(list));
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_empty_list() {
        assert_eq!(List::Nil.sum(), 0);
    }

    #[test]
    fn sums_a_manually_built_list() {
        let list = List::Cons(
            1,
            Box::new(List::Cons(2, Box::new(List::Cons(3, Box::new(List::Nil))))),
        );
        assert_eq!(list.sum(), 6);
    }

    #[test]
    fn builds_from_vec_and_preserves_order() {
        let list = from_vec(&[10, 20, 30]);
        assert_eq!(list.sum(), 60);
    }

    #[test]
    fn from_vec_of_empty_slice_is_nil() {
        let list = from_vec(&[]);
        assert_eq!(list.sum(), 0);
    }

    #[test]
    fn from_vec_matches_manual_construction_order() {
        // Building [1, 2] should be Cons(1, Cons(2, Nil)) — i.e. summing
        // still works out to the same total regardless of order here since
        // addition is commutative, but this pins down that from_vec doesn't
        // panic or drop elements for a longer list.
        let list = from_vec(&[1, 2, 3, 4, 5]);
        assert_eq!(list.sum(), 15);
    }
}
