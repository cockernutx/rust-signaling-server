use actix::prelude::{Actor, Addr};
use actix_web::{middleware, web, App, HttpServer};


use std::sync::Arc;

use error::Error;
use signal::Signal;
use signal_router::{ExitMessage, JoinMessage, SignalMessage, SignalRouter};
mod error;
mod signal;
mod signal_router;
mod signal_socket;
mod server;

type SignalServerStateData = web::Data<Arc<SignalServerState>>;

struct SignalServerState {
    signal_router: Addr<SignalRouter>,
}

impl SignalServerState {
    fn new(signal_router: Addr<SignalRouter>) -> Self {
        SignalServerState { signal_router }
    }
}



#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let server_fut = async {
        let signal_router = SignalRouter::default();
        let signal_router_addr = signal_router.start();
        let state = Arc::new(SignalServerState::new(signal_router_addr));
        HttpServer::new(move || {
            App::new()
                .data(state.clone())
                .wrap(middleware::Logger::default())
                .service(server::websocket_client)
                //.service(server::websocket_server)
        })
    };

    let server = server_fut.await;
    let port = 8081;
    server.bind(format!("0.0.0.0:{}", port))?.run().await
}

