//! Exercises for 2.3.5 — associated types versus generic parameters.
//!
//! Two traits, already fully written below — the point of this exercise is
//! not designing the traits, it's filling in method bodies that respect
//! what each trait shape already promises: `Measures` gives every type
//! exactly one `Output`; `DescribesAs<T>` lets a type answer differently
//! for different `T`.

/// A trait for measuring one property of a value that has exactly one
/// correct answer, computed the same way every time.
pub trait Measures {
    type Output;
    fn measure(&self) -> Self::Output;
}

/// A trait for describing a value in a representation the caller picks.
pub trait DescribesAs<T> {
    fn describe(&self) -> T;
}

/// A rectangle, in whatever unit the caller is using.
pub struct Rectangle {
    pub width: u32,
    pub height: u32,
}

impl Measures for Rectangle {
    type Output = u32;

    /// Returns the rectangle's area: `width` times `height`.
    fn measure(&self) -> u32 {
        self.width * self.height
    }
}

impl DescribesAs<u32> for Rectangle {
    /// Returns the rectangle's perimeter: twice the sum of `width` and
    /// `height`.
    fn describe(&self) -> u32 {
        2 * (self.width + self.height)
    }
}

impl DescribesAs<String> for Rectangle {
    /// Formats the rectangle as `width`, then the letter `x`, then
    /// `height`, with no spaces — for example, a rectangle 3 wide and 4
    /// tall formats as `"3x4"`.
    fn describe(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_area_needs_no_annotation_anywhere() {
        let r = Rectangle {
            width: 3,
            height: 4,
        };
        // `Output` is fixed to `u32`, so nothing here has to say so.
        assert_eq!(r.measure(), 12);
    }

    #[test]
    fn measures_area_for_a_different_rectangle() {
        let r = Rectangle {
            width: 10,
            height: 2,
        };
        assert_eq!(r.measure(), 20);
    }

    #[test]
    fn describes_as_perimeter() {
        let r = Rectangle {
            width: 3,
            height: 4,
        };
        let perimeter: u32 = r.describe();
        assert_eq!(perimeter, 14);
    }

    #[test]
    fn describes_as_label() {
        let r = Rectangle {
            width: 3,
            height: 4,
        };
        let label: String = r.describe();
        assert_eq!(label, "3x4");
    }

    #[test]
    fn describes_as_label_for_a_different_rectangle() {
        let r = Rectangle {
            width: 10,
            height: 1,
        };
        let label: String = r.describe();
        assert_eq!(label, "10x1");
    }
}
