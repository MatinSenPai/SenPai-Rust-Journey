//! One `tokio::sync::broadcast` channel, two SSE clients: every subscriber
//! gets its own copy of every message.

use std::convert::Infallible;

use axum::extract::State;
use axum::response::sse::{Event, Sse};
use axum::routing::get;
use axum::Router;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};

async fn events(
    State(tx): State<broadcast::Sender<String>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Subscribe here, before the response goes out, so nothing is missed.
    let stream = BroadcastStream::new(tx.subscribe())
        .filter_map(|item| item.ok())
        .map(|msg| Ok(Event::default().data(msg)));
    Sse::new(stream)
}

/// Connects, reads the response head, and returns the reader.
async fn connect(addr: std::net::SocketAddr) -> BufReader<TcpStream> {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(b"GET /events HTTP/1.1\r\nHost: x\r\n\r\n")
        .await
        .unwrap();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    while line != "\r\n" {
        line.clear();
        reader.read_line(&mut line).await.unwrap();
    }
    reader
}

/// Reads one chunk: the hex size line, the data, the trailing CRLF.
async fn chunk(reader: &mut BufReader<TcpStream>) -> String {
    let mut size = String::new();
    reader.read_line(&mut size).await.unwrap();
    let n = usize::from_str_radix(size.trim(), 16).unwrap();
    let mut data = vec![0; n + 2];
    reader.read_exact(&mut data).await.unwrap();
    String::from_utf8_lossy(&data[..n]).into_owned()
}

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<String>(16);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = Router::new()
        .route("/events", get(events))
        .with_state(tx.clone());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let mut a = connect(addr).await;
    let mut b = connect(addr).await;
    println!("subscribers: {}", tx.receiver_count());

    let reached = tx.send("konnichiwa".to_string()).unwrap();
    println!("delivered to {reached}");
    println!("client A got {:?}", chunk(&mut a).await);
    println!("client B got {:?}", chunk(&mut b).await);
    server.abort();
}
