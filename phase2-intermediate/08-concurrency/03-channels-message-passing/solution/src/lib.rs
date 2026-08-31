use std::sync::mpsc;
use std::thread;

pub fn sum_via_channel(nums: Vec<i32>) -> i32 {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let total: i32 = nums.iter().sum();
        tx.send(total).unwrap();
    });
    rx.recv().unwrap()
}

pub fn collect_from_workers(worker_count: usize, values_per_worker: usize) -> Vec<i32> {
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();

    for i in 0..worker_count {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let start = (i * values_per_worker) as i32;
            let end = start + values_per_worker as i32;
            for value in start..end {
                tx.send(value).unwrap();
            }
        }));
    }
    drop(tx); // without this, rx.iter() below would wait forever

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
    fn sums_a_handful_of_numbers() {
        assert_eq!(sum_via_channel(vec![1, 2, 3, 4]), 10);
    }

    #[test]
    fn sums_an_empty_vec_to_zero() {
        assert_eq!(sum_via_channel(vec![]), 0);
    }

    #[test]
    fn collects_every_value_from_every_worker() {
        let mut result = collect_from_workers(3, 4);
        result.sort();
        assert_eq!(result, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn a_single_worker_still_works() {
        let mut result = collect_from_workers(1, 5);
        result.sort();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn zero_workers_collects_nothing() {
        assert!(collect_from_workers(0, 5).is_empty());
    }

    #[test]
    fn zero_values_per_worker_collects_nothing() {
        assert!(collect_from_workers(4, 0).is_empty());
    }
}
