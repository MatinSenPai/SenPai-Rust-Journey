pub fn letter_grade(score: u32) -> &'static str {
    if score >= 90 {
        "A"
    } else if score >= 80 {
        "B"
    } else if score >= 70 {
        "C"
    } else {
        "F"
    }
}

pub fn first_multiple_above(n: u32, min: u32) -> u32 {
    let mut candidate = 0;
    loop {
        candidate += n;
        if candidate > min {
            break candidate;
        }
    }
}

pub fn sum_up_to(n: u32) -> u32 {
    let mut total = 0;
    for i in 1..=n {
        total += i;
    }
    total
}

pub fn classify(n: i32) -> &'static str {
    match n {
        0 => "zero",
        1 | 2 => "small",
        3..=9 => "medium",
        _ => "large",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grades() {
        assert_eq!(letter_grade(95), "A");
        assert_eq!(letter_grade(85), "B");
        assert_eq!(letter_grade(75), "C");
        assert_eq!(letter_grade(50), "F");
    }

    #[test]
    fn multiples() {
        assert_eq!(first_multiple_above(5, 12), 15);
        assert_eq!(first_multiple_above(3, 0), 3);
    }

    #[test]
    fn sums() {
        assert_eq!(sum_up_to(5), 15);
        assert_eq!(sum_up_to(1), 1);
    }

    #[test]
    fn classifies() {
        assert_eq!(classify(0), "zero");
        assert_eq!(classify(1), "small");
        assert_eq!(classify(2), "small");
        assert_eq!(classify(5), "medium");
        assert_eq!(classify(9), "medium");
        assert_eq!(classify(10), "large");
        assert_eq!(classify(-5), "large");
    }
}
