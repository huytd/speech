use crate::speech::SpeechEngine;
use axum::Router;
use tower_http::services::ServeDir;

use crate::state::ServerState;

mod api;
mod data;
mod speech;
mod state;

#[tokio::main]
async fn main() {
    SpeechEngine::initialize_speech_model();

    let app = Router::new()
        .nest_service("/", ServeDir::new("./public"))
        .nest("/api", api::router())
        .with_state(ServerState::new());

    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    axum::Server::bind(&format!("0.0.0.0:{port}").parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
