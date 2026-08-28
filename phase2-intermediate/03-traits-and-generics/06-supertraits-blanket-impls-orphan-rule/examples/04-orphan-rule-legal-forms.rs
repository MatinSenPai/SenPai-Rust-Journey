//! Two legal shapes under the orphan rule: your trait for a foreign type, and
//! a foreign trait for your type. Exactly one side is local, either time.

use std::fmt;

trait Summarized {
    fn summarize(&self) -> String;
}

// Foreign type (`Vec` is std's), local trait (`Summarized` is ours).
impl Summarized for Vec<i32> {
    fn summarize(&self) -> String {
        let total: i32 = self.iter().sum();
        format!("{} numbers, sum {}", self.len(), total)
    }
}

struct Point {
    x: i32,
    y: i32,
}

// Local type (`Point` is ours), foreign trait (`Display` is std's).
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

fn main() {
    let nums = vec![1, 2, 3, 4];
    println!("{}", nums.summarize());

    let p = Point { x: 3, y: 4 };
    println!("{p}");
}
