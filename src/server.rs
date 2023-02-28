use actix_web::{get, HttpRequest, web, HttpResponse};
use actix_web_actors::ws;
use uuid::Uuid;

use crate::{SignalServerStateData, signal_socket::SignalSocket};


pub mod user;
pub mod error;

#[get("/ws")]
async fn websocket_client(
    state: SignalServerStateData,
    request: HttpRequest,
    //info: web::Query<user::User>,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {

    //let user = info.into_inner();


    let user_name = Uuid::new_v4();
    ws::start(
        SignalSocket::new(user_name, &state.signal_router),
        &request,
        stream,
    )
}
/*

#[get("/ws/{server_name}")]
async fn websocket_server(
    state: SignalServerStateData,
    request: HttpRequest,
    path: web::Path<String>,
    info: web::Query<user::User>,
    stream: web::Payload,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    
    let server_name = path.into_inner();

    let user = info.into_inner();

    if user != user::User::new("admin".to_string(), "admin".to_string()) {
        return Ok(HttpResponse::Unauthorized().json(error::ApiError
        {
            error: "Unauthorized"
        }))
    }


    ws::start(
        SignalSocket::new(server_name, &state.signal_router),
        &request,
        stream,
    )
}
 */
