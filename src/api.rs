use crate::data::SpeechAnalyzeResult;
use crate::speech::SpeechEngine;
use crate::state::ServerState;
use axum::body::Body;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};

async fn pronounce_lookup_handler(
    state: State<ServerState>,
    Json(words): Json<Vec<String>>,
) -> impl IntoResponse {
    let result = words
        .iter()
        .map(|word| state.lookup_pronounce(word))
        .collect::<Vec<&str>>();
    Json(result)
}

async fn get_random_example_handler(state: State<ServerState>) -> impl IntoResponse {
    Json(state.get_random_example())
}

async fn speech_recognition_handler(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(field) = multipart
        .next_field()
        .await
        .expect("Could not read form data")
    {
        if field.content_type().eq(&Some("audio/wav"))
            || field.content_type().eq(&Some("audio/x-wav"))
        {
            let buffer = field.bytes().await.expect("Could not read audio data!");
            let mut cursor = std::io::Cursor::new(buffer);
            let reader = wav::read(&mut cursor).unwrap();
            let samples = reader.1.as_sixteen().unwrap();
            if let Some(mut rec) = SpeechEngine::create_recognizer(reader.0.sampling_rate as f32) {
                for chunk in samples.chunks(100) {
                    rec.accept_waveform(chunk);
                }
                if let Some(single) = rec.final_result().single() {
                    return Ok(Json(SpeechAnalyzeResult::from(single)));
                }
            }
        }
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn router() -> Router<ServerState, Body> {
    Router::new()
        .route("/pronounce", post(pronounce_lookup_handler))
        .route("/example", get(get_random_example_handler))
        .route("/analyze", post(speech_recognition_handler))
}
