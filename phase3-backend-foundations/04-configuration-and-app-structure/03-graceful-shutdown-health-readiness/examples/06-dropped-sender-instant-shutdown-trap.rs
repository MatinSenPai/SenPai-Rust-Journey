//! No compiler error, no panic: a server that shuts itself down the moment
//! it starts, because the `oneshot` sender that was meant to trigger the
//! shutdown was dropped. `rx.await` returns `Err` for a dropped sender, and
//! `.ok()` turns that into "shutdown, now".
//!
//!     cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 06-dropped-sender-instant-shutdown-trap

use std::time::Duration;

use axum::Router;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (stop, stopped) = oneshot::channel::<()>();
    drop(stop); // meant to be kept and fired later; here it just goes out of scope

    let server = axum::serve(listener, Router::new()).with_graceful_shutdown(async {
        stopped.await.ok();
    });
    let result = tokio::time::timeout(Duration::from_secs(5), server).await;

    println!("server finished on its own: {result:?}");
    match TcpStream::connect(addr).await {
        Ok(_) => println!("a client can still connect"),
        Err(e) => println!("a client trying to connect gets: {:?}", e.kind()),
    }
}
