use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use error::Error;
use futures::{
    sink::SinkExt,
    stream::{StreamExt},
};
use std::{
    collections::{HashMap},
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use uuid;

use crate::signal::Signal;

mod error;
mod logger;
mod signal;

// Our shared state
struct AppState {
    // We require unique usernames. This tracks which usernames have been taken.
    user_set: Mutex<HashMap<String, broadcast::Sender<Signal>>>,
    // Channel used to send messages to all connected clients.
}

#[tokio::main]
async fn main() {
    // Set up application state for use with with_state().
    let user_set = Mutex::new(HashMap::new());

    let app_state = Arc::new(AppState { user_set });

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let shared_state = Arc::clone(&state);
    let mut uuid_str = uuid::Uuid::new_v4().to_string();
    let user_set = shared_state.user_set.lock().unwrap();
    while user_set.contains_key(&uuid_str) {
        uuid_str = uuid::Uuid::new_v4().to_string();
    }

    let username = uuid_str.clone();

    ws.on_upgrade(|socket| websocket(socket, state, username))
}

// This function deals with a single websocket connection, i.e., a single
// connected client / user, for which we will spawn two independent tasks (for
// receiving / sending chat messages).
async fn websocket(stream: WebSocket, state: Arc<AppState>, username: String) {
    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();
    let (tx, mut rx) = broadcast::channel(1024);
    state.user_set.lock().unwrap().insert(username.clone(), tx);
    


    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {

            let json_str = serde_json::to_string(&msg).unwrap();
            // In any websocket error, break loop.
            if sender.send(Message::Text(json_str)).await.is_err() {
                break;
            }
        }
    });

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let state_clone = Arc::clone(&state);
    let username_clone = username.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(m)) = receiver.next().await {
            match m {
                Message::Text(msg) => handle_msg(msg.clone(), username_clone.clone(), &state_clone),
                Message::Binary(_) => error_handler(Error::ServiceUnavailable, username_clone.clone(), &state_clone),
                Message::Close(_) => {
                    let _ = &state_clone.user_set.lock().unwrap().remove(&username_clone.clone());
                },
                _ => {}
            }
            ;
        }
    });
    let _ = state.user_set.lock().unwrap().get_mut(&username).unwrap().send(Signal::assign(username.clone()));
    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };


    // Remove username from map so new clients can take it again.
    state.user_set.lock().unwrap().remove(&username);
}

fn handle_msg(msg: String, username: String, state: &Arc<AppState>) {
    let s: Result<Signal, serde_json::Error> = serde_json::from_str(&msg);

    if let Err(err) = s {
        error_handler(Error::ParseError(err.to_string()),  username.clone(), state);
    }
    else {
        let signal = s.unwrap();
        let target = match signal.clone() {
            Signal::Offer(o) => o.target,
            Signal::Answer(o) => o.target,
            Signal::NewIceCandidate(o) => o.target,
            Signal::Assign(_) => {
                return;
            },
            Signal::Error(err) => {
                error_handler(err, username, state);
                return;
            }
        };
        let target_exists = state.user_set.lock().unwrap().contains_key(&target.clone());
 

        if !target_exists {
            error_handler(Error::TargetNotFound(target.clone()), username, state);
            return;
        }

        let _ = state.user_set.lock().unwrap().get_mut(&target).unwrap().send(signal);


    }
}

fn error_handler(err: Error, username: String, state: &Arc<AppState>) {

    if let Err(err1) = state.user_set.lock().unwrap().get_mut(&username).unwrap().send(Signal::Error(err)) {
        logger::logln!(error => "{}", err1);
    }
}