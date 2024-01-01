use std::sync::Arc;

use crate::{error::Error, logger, signal::Signal};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::broadcast;

use crate::AppState;

pub fn ws_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(websocket_handler))
        .route("/connected_list", get(connected_list))
}

async fn connected_list(State(state): State<Arc<AppState>>) -> Json<Vec<String>> {
    Json(
        state
            .user_set
            .lock()
            .unwrap()
            .keys()
            .map(|f| f.clone())
            .collect::<Vec<String>>(),
    )
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

    {
        let state_clone = state.user_set.lock().unwrap().clone();
        let keys = state_clone
            .keys()
            .map(|f| f.clone())
            .collect::<Vec<String>>();
        for user in state_clone.iter() {
            let _ = user.1.send(Signal::ConnectedList(keys.clone()));
        }
    }
    
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
                Message::Binary(_) => error_handler(
                    Error::ServiceUnavailable,
                    username_clone.clone(),
                    &state_clone,
                ),
                Message::Close(_) => {
                    let _ = &state_clone
                        .user_set
                        .lock()
                        .unwrap()
                        .remove(&username_clone.clone());
                }
                _ => {}
            };
        }
    });
    let _ = state
        .user_set
        .lock()
        .unwrap()
        .get_mut(&username)
        .unwrap()
        .send(Signal::assign(username.clone()));
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
        error_handler(Error::ParseError(err.to_string()), username.clone(), state);
    } else {
        let signal = s.unwrap();
        let target = match signal.clone() {
            Signal::Offer(o) => o.target,
            Signal::Answer(o) => o.target,
            Signal::NewIceCandidate(o) => o.target,
            Signal::Assign(_) => {
                return;
            }
            Signal::Error(err) => {
                error_handler(err, username, state);
                return;
            }
            Signal::ConnectedList(_) => todo!(),
        };
        let target_exists = state.user_set.lock().unwrap().contains_key(&target.clone());

        if !target_exists {
            error_handler(Error::TargetNotFound(target.clone()), username, state);
            return;
        }

        let _ = state
            .user_set
            .lock()
            .unwrap()
            .get_mut(&target)
            .unwrap()
            .send(signal);
    }
}

fn error_handler(err: Error, username: String, state: &Arc<AppState>) {
    if let Err(err1) = state
        .user_set
        .lock()
        .unwrap()
        .get_mut(&username)
        .unwrap()
        .send(Signal::Error(err))
    {
        logger::logln!(error => "{}", err1);
    }
}
