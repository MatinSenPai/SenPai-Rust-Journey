//! Exercises for 2.6.4 — `Weak` and reference cycles.
//!
//! Nothing below needs `RefCell` or `Rc::new_cyclic` — the sample tree the
//! tests use is already built for you. Every function here only *reads* a
//! `Weak` or a `Folder` that already exists.

use std::rc::{Rc, Weak};

/// A node in a small folder tree: a name, a possibly-absent parent, and zero
/// or more owned children.
pub struct Folder {
    pub name: String,
    pub parent: Weak<Folder>,
    pub children: Vec<Rc<Folder>>,
}

/// `true` if `weak`'s target is still alive, `false` once every strong owner
/// has already dropped it.
///
/// # Examples
///
/// While the strong owner is still alive, `is_reachable(&weak)` is `true`.
/// After the strong owner is dropped, `is_reachable(&weak)` is `false`.
pub fn is_reachable<T>(weak: &Weak<T>) -> bool {
    todo!("try to upgrade `weak` and report whether that succeeded")
}

/// Exactly `format!("alive: {value:?}")` when `weak` can still be upgraded,
/// exactly `"gone"` once every strong owner has dropped its target.
///
/// # Examples
///
/// Given `let rc = Rc::new(5); let weak = Rc::downgrade(&rc);` — while `rc`
/// is still alive, `describe(&weak)` is `"alive: 5"`. After `drop(rc)`,
/// `describe(&weak)` is `"gone"`.
pub fn describe<T: std::fmt::Debug>(weak: &Weak<T>) -> String {
    todo!("upgrade `weak`; format the two cases exactly as the doc comment states")
}

/// The parent's `name`, cloned — or `None` if `folder` has no living parent
/// (it is the root, or its parent has already been dropped).
pub fn parent_name(folder: &Folder) -> Option<String> {
    todo!("upgrade `folder`'s parent and, if it is still there, clone its name out")
}

/// How many living parents there are between `folder` and the root — `0` if
/// `folder` itself has no living parent.
///
/// # Examples
///
/// The root's own depth is `0`. A direct child of the root has depth `1`.
pub fn depth(folder: &Rc<Folder>) -> usize {
    todo!(
        "walk upward one step at a time, upgrading the current folder's parent each time, \
         counting steps, until there is no parent left to upgrade"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `root` with one child, `docs`. Built once, here, so nothing above has
    /// to construct a tree — only read one.
    fn sample_tree() -> Rc<Folder> {
        Rc::new_cyclic(|weak_root| Folder {
            name: "root".to_string(),
            parent: Weak::new(),
            children: vec![Rc::new(Folder {
                name: "docs".to_string(),
                parent: weak_root.clone(),
                children: vec![],
            })],
        })
    }

    #[test]
    fn is_reachable_is_true_while_the_owner_is_alive() {
        let rc = Rc::new(5);
        let weak = Rc::downgrade(&rc);
        assert!(is_reachable(&weak));
    }

    #[test]
    fn is_reachable_is_false_after_the_owner_drops() {
        let rc = Rc::new(5);
        let weak = Rc::downgrade(&rc);
        drop(rc);
        assert!(!is_reachable(&weak));
    }

    #[test]
    fn describe_reports_alive() {
        let rc = Rc::new(5);
        let weak = Rc::downgrade(&rc);
        assert_eq!(describe(&weak), "alive: 5");
    }

    #[test]
    fn describe_reports_gone() {
        let rc = Rc::new(String::from("temp"));
        let weak = Rc::downgrade(&rc);
        drop(rc);
        assert_eq!(describe(&weak), "gone");
    }

    #[test]
    fn parent_name_of_the_root_is_none() {
        let root = sample_tree();
        assert_eq!(parent_name(&root), None);
    }

    #[test]
    fn parent_name_of_a_child_is_its_parents_name() {
        let root = sample_tree();
        assert_eq!(parent_name(&root.children[0]), Some("root".to_string()));
    }

    #[test]
    fn depth_of_the_root_is_zero() {
        let root = sample_tree();
        assert_eq!(depth(&root), 0);
    }

    #[test]
    fn depth_of_a_direct_child_is_one() {
        let root = sample_tree();
        assert_eq!(depth(&root.children[0]), 1);
    }
}
