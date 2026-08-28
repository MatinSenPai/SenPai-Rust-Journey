/// Returns the largest element in `items`, or `None` if the slice is empty.
///
/// `T: PartialOrd + Copy` means: this function works for *any* type `T`, as
/// long as that type supports ordering comparisons (`>`, `<`, ...) and can
/// be copied cheaply (so we can pull values out of the slice by value
/// instead of juggling references). `i32`, `f64`, and `char` all satisfy
/// both bounds out of the box.
pub fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    todo!("start with items.first().copied(), then loop the rest comparing with >")
}

/// A generic LIFO (last-in, first-out) stack, usable with any element type.
pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    /// Constructs a new, empty `Stack`.
    pub fn new() -> Self {
        todo!("Stack {{ items: Vec::new() }}")
    }

    /// Pushes `item` onto the top of the stack.
    pub fn push(&mut self, item: T) {
        todo!("self.items.push(item)")
    }

    /// Removes and returns the top item, or `None` if the stack is empty.
    pub fn pop(&mut self) -> Option<T> {
        todo!("self.items.pop()")
    }

    /// Returns a reference to the top item without removing it.
    pub fn peek(&self) -> Option<&T> {
        todo!("self.items.last()")
    }

    /// Returns the number of items currently on the stack.
    pub fn len(&self) -> usize {
        todo!("self.items.len()")
    }

    /// Returns `true` if the stack has no items.
    pub fn is_empty(&self) -> bool {
        todo!("self.items.is_empty()")
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn largest_finds_max_i32() {
        assert_eq!(largest(&[3, 7, 2, 9, 4]), Some(9));
    }

    #[test]
    fn largest_finds_max_f64() {
        assert_eq!(largest(&[1.5, 2.5, 0.5]), Some(2.5));
    }

    #[test]
    fn largest_of_empty_slice_is_none() {
        let empty: [i32; 0] = [];
        assert_eq!(largest(&empty), None);
    }

    #[test]
    fn stack_of_i32_pushes_and_pops_in_lifo_order() {
        let mut stack: Stack<i32> = Stack::new();
        assert!(stack.is_empty());
        stack.push(1);
        stack.push(2);
        stack.push(3);
        assert_eq!(stack.len(), 3);
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.peek(), Some(&1));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
        assert!(stack.is_empty());
    }

    #[test]
    fn stack_of_string_works_too() {
        let mut stack: Stack<String> = Stack::new();
        stack.push(String::from("one"));
        stack.push(String::from("two"));
        assert_eq!(stack.peek(), Some(&String::from("two")));
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.pop(), Some(String::from("two")));
        assert_eq!(stack.pop(), Some(String::from("one")));
        assert_eq!(stack.pop(), None);
    }
}
