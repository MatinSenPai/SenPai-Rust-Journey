use std::sync::Arc;

use p3_07_02_sessions_vs_jwt_solution::{app, AppState, SessionStore, SystemClock};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:3180";
    let store = Arc::new(SessionStore::new(Arc::new(SystemClock), 3600));
    let state = AppState {
        store,
        secure: false, // plain http on localhost; true behind HTTPS
        origin: format!("http://{addr}"),
    };
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");
    println!("sessions demo listening on {addr}");
    axum::serve(listener, app(state))
        .await
        .expect("server error");
}
