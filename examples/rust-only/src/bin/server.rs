//! Rust-only example — server side.
//!
//! Run:
//!   cargo run --bin server
//!
//! Then run the client in another terminal:
//!   cargo run --bin client

use actix_web::{web, App, HttpServer};
use eruspy::server::transfer_scope;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host    = "0.0.0.0:3000";
    let storage = "./storage";

    println!("[ rust server ] http://{host}  storage: {storage}");

    HttpServer::new(|| {
        App::new().service(
            web::scope("/transfer")
                .service(transfer_scope(storage, true)),
        )
    })
    .bind(host)?
    .run()
    .await
}
