use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};
use std::path::PathBuf;

fn example_path() -> String { 
    format!("{}/{}", env!("CARGO_MANIFEST_DIR").to_owned(), "examples/webrtc-chat/")
}

async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let mut path = PathBuf::from(example_path());
    path.push("index.html");
    Ok(NamedFile::open(path)?)
}

async fn file(req: HttpRequest) -> Result<NamedFile> {
    let mut path = PathBuf::from(example_path());
    path.push(req.match_info().query("filename"));
    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{web, App, HttpServer};

    HttpServer::new(|| App::new().route("/", web::get().to(index)).route("/{filename:.*}", web::get().to(file)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}