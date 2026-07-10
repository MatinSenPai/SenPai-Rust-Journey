//! The binary the `Dockerfile` in the parent lesson directory builds and
//! runs. Not an exercise — just enough plumbing to have a real process
//! that listens on a port, so the Dockerfile has something concrete to
//! package.

use p4_07_01_docker_compose_and_ci_solution::app;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind port 3000");

    println!("listening on http://{addr}");

    axum::serve(listener, app())
        .await
        .expect("server exited unexpectedly");
}
