pub fn keep_and_modify(n: i32) -> (i32, i32) {
    let mut m = n;
    m += 10;
    (n, m)
}

pub fn duplicate_and_uppercase(tag: String) -> (String, String) {
    let upper = tag.clone().to_uppercase();
    (tag, upper)
}

pub fn double_prices(prices: Vec<f64>) -> (Vec<f64>, Vec<f64>) {
    let original = prices.clone();
    let mut doubled = Vec::new();
    for price in prices {
        doubled.push(price * 2.0);
    }
    (original, doubled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_and_modifies() {
        assert_eq!(keep_and_modify(5), (5, 15));
    }

    #[test]
    fn duplicates_and_uppercases() {
        assert_eq!(
            duplicate_and_uppercase("rust".to_string()),
            ("rust".to_string(), "RUST".to_string())
        );
    }

    #[test]
    fn doubles_prices() {
        let original = vec![1.0, 2.5, 10.0];
        let (kept, doubled) = double_prices(original.clone());
        assert_eq!(kept, original);
        assert_eq!(doubled, vec![2.0, 5.0, 20.0]);
    }
}
