//! DELIBERATELY BROKEN — expected: E0308
//!
//! The handler asks for `State<Config>`, but the state struct only derives
//! `Clone`. Nothing tells `axum` how to get a `Config` out of an `AppState`,
//! so the router is stuck expecting a bare `Config` as its state.
//!
//!     cargo build -p p3-04-02-app-state-and-dependency-wiring --example 06-no-fromref-for-substate-broken --features broken

use axum::extract::State;
use axum::routing::get;
use axum::Router;

#[derive(Clone)]
struct Config {
    greeting: &'static str,
}

#[derive(Clone)]
struct AppState {
    config: Config,
}

async fn hello(State(config): State<Config>) -> &'static str {
    config.greeting
}

fn main() {
    let state = AppState {
        config: Config {
            greeting: "konnichiwa",
        },
    };
    let _app: Router = Router::new().route("/hello", get(hello)).with_state(state);
}
