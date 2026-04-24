//! Mixed example — Rust server / Python client.
//!
//! Run this first:
//!   cargo run --bin server
//!
//! Then in another terminal, run the Python client:
//!   python client.py

use actix_web::{web, App, HttpServer};
use eruspy::server::transfer_scope;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host    = "0.0.0.0:3000";
    let storage = "./storage";

    println!("[ rust server ] http://{host}  storage: {storage}");
    println!("Waiting for Python client... Press Ctrl+C to stop.\n");

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
