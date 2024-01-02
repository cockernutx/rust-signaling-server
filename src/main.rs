
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use axum::Router;
use tokio::sync::broadcast;

use crate::signal::Signal;

mod error;
mod logger;
mod signal;
mod websocket;
mod name_generator;

struct AppState {
    // We require unique usernames. This tracks which usernames have been taken.
    user_set: Mutex<HashMap<String, broadcast::Sender<Signal>>>
}

#[tokio::main]
async fn main() {
    // Set up application state for use with with_state().
    let user_set = Mutex::new(HashMap::new());

    let app_state = Arc::new(AppState { user_set });

    let app = Router::new()
        .nest("/ws", websocket::ws_routes())
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    println!("server starting at:");
    println!("http: http://localhost:8081");
    println!("websocket: ws://localhost:8081/ws");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

