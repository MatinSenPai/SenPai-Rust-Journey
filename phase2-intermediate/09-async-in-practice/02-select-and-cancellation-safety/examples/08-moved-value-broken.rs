//! DELIBERATELY BROKEN — expected: E0382
//! Run `cargo run -p p2-09-02-select-and-cancellation-safety --example
//! 08-moved-value-broken --features broken` and read the error.

async fn send(payload: String) {
    println!("sent {payload}");
}

async fn log_dropped(payload: String) {
    println!("dropped {payload}");
}

#[tokio::main]
async fn main() {
    let payload = String::from("hello");
    tokio::select! {
        _ = send(payload) => {}
        _ = log_dropped(payload) => {}
    }
}
