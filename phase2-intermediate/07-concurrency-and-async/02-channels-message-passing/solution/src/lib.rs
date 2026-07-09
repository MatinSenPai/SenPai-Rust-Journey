use std::sync::mpsc;
use std::thread;

pub fn compute_async_sum(nums: Vec<i32>) -> i32 {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let sum: i32 = nums.iter().sum();
        tx.send(sum).unwrap();
    });
    rx.recv().unwrap()
}

pub fn collect_from_producers(producer_count: usize, values_per_producer: usize) -> Vec<i32> {
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();

    for i in 0..producer_count {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let start = (i * values_per_producer) as i32;
            for v in start..start + values_per_producer as i32 {
                tx.send(v).unwrap();
            }
        }));
    }
    drop(tx); // without this, rx's iterator below would wait forever

    let mut collected: Vec<i32> = rx.iter().collect();

    for handle in handles {
        handle.join().unwrap();
    }

    collected.sort();
    collected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_sum_via_channel() {
        assert_eq!(compute_async_sum(vec![1, 2, 3, 4]), 10);
    }

    #[test]
    fn computes_sum_of_empty_vec() {
        assert_eq!(compute_async_sum(vec![]), 0);
    }

    #[test]
    fn collects_every_value_from_every_producer() {
        let mut result = collect_from_producers(3, 4);
        result.sort();
        assert_eq!(result, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn single_producer_still_works() {
        let mut result = collect_from_producers(1, 5);
        result.sort();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }
}
