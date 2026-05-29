use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tokio::task::JoinError;

#[derive(thiserror::Error, Debug)]
#[error("...")]
pub enum Error {
    #[error("{0}")]
    BadRequest(#[from] BadRequest),

    #[error("{0}")]
    NotFound(#[from] NotFound),

    #[error("External request failed: {0}")]
    External(#[from] reqwest::Error),

    #[error("Geocode returned no results for: {0}")]
    GeocodeMiss(String),

    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("Task error: {0}")]
    RunSyncTask(#[from] JoinError),
}

impl Error {
    fn get_codes(&self) -> (StatusCode, u16) {
        match self {
            Error::BadRequest(_) => (StatusCode::BAD_REQUEST, 40002),
            Error::NotFound(_) => (StatusCode::NOT_FOUND, 40003),
            Error::GeocodeMiss(_) => (StatusCode::BAD_REQUEST, 40007),
            Error::External(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5006),
            Error::Db(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5007),
            Error::RunSyncTask(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5005),
        }
    }

    pub fn bad_request() -> Self {
        Error::BadRequest(BadRequest {})
    }

    pub fn not_found() -> Self {
        Error::NotFound(NotFound {})
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status_code, code) = self.get_codes();
        let message = self.to_string();
        let body = Json(json!({ "code": code, "message": message }));

        (status_code, body).into_response()
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Bad Request")]
pub struct BadRequest {}

#[derive(thiserror::Error, Debug)]
#[error("Not found")]
pub struct NotFound {}
