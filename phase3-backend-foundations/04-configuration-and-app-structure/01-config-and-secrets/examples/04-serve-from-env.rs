//! A server whose port, CORS origins and password come from the real
//! environment, with an optional `config.toml` in the current directory
//! underneath. Needs the exercises done. Stop it with Ctrl+C.
//!
//!     APP_DB_PASSWORD=hunter2 cargo run -p p3-04-01-config-and-secrets --example 04-serve-from-env

use std::collections::HashMap;

use p3_04_01_config_and_secrets::{app, Config};

#[tokio::main]
async fn main() {
    let env: HashMap<String, String> = std::env::vars().collect();
    let file = std::fs::read_to_string("config.toml").ok();

    let config = match Config::load(file.as_deref(), &env) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("cannot start: {err}");
            std::process::exit(1);
        }
    };
    println!("{config:?}");

    let router = app(&config).expect("invalid CORS origin");
    let addr = format!("127.0.0.1:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind address");
    println!("listening on http://{addr}");
    axum::serve(listener, router).await.expect("server error");
}
