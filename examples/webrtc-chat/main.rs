use axum::{
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    services::{ServeDir},
    trace::TraceLayer,
};

#[tokio::main]
async fn main() {

        serve(using_serve_dir(), 3001).await;
}

fn using_serve_dir() -> Router {
    // serve the file in the "assets" directory under `/assets`
    Router::new().nest_service("/", ServeDir::new("./"))
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    axum::Server::bind(&addr)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await
        .unwrap();
}