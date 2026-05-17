use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

mod app;
mod db;
mod models;
mod repositories;
mod routes;
mod settings;
mod state;

use settings::SETTINGS;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port = SETTINGS.server.port;
    let address = SocketAddr::from(([127, 0, 0, 1], port));

    let app = app::create_app().await;

    let listener = TcpListener::bind(address).await.unwrap();
    info!("Server listening on {}", &address);

    axum::serve(listener, app).await.unwrap();
}
