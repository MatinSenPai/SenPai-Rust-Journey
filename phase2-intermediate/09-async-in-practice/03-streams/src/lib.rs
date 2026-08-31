use tokio_stream::StreamExt;

/// Turns `values` into a stream (with `tokio_stream::iter`) and sums every
/// item by driving that stream with a `while let Some(...) = ... .next().await`
/// loop — not `values.iter().sum()`. An empty `values` sums to `0`.
pub async fn sum_stream(values: Vec<i32>) -> i32 {
    todo!(
        "build a stream from `values` with tokio_stream::iter, then drive it with a \
         while-let .next().await loop, adding each item to a running total"
    )
}

/// Builds a stream from `values`, keeps only the even numbers, keeps only
/// the first `limit` of those (in their original order), and returns them
/// as a `Vec<i32>`. If fewer than `limit` even numbers exist, returns all
/// of them.
pub async fn first_n_even(values: Vec<i32>, limit: usize) -> Vec<i32> {
    todo!(
        "build a stream from `values`; keep only the even numbers; keep only the first \
         `limit` of those, in order; collect them into a Vec by driving the stream"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sums_an_empty_stream_to_zero() {
        assert_eq!(sum_stream(vec![]).await, 0);
    }

    #[tokio::test]
    async fn sums_every_item() {
        assert_eq!(sum_stream(vec![1, 2, 3, 4]).await, 10);
    }

    #[tokio::test]
    async fn keeps_only_the_first_n_even_numbers_in_order() {
        let result = first_n_even(vec![1, 2, 3, 4, 5, 6, 7, 8], 3).await;
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[tokio::test]
    async fn returns_fewer_than_limit_if_not_enough_even_numbers_exist() {
        let result = first_n_even(vec![1, 3, 5, 6], 5).await;
        assert_eq!(result, vec![6]);
    }
}
