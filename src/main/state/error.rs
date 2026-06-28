use thiserror::Error;

/// Errors from the store layer.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migration(String),

    #[error("data codec error: {0}")]
    Codec(String),

    #[error("not found")]
    NotFound,
}

/// Top-level application errors returned from axum handlers.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("store error: {0}")]
    Store(#[from] StoreError),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("precondition failed")]
    PreconditionFailed,
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::PreconditionFailed => StatusCode::PRECONDITION_FAILED,
            AppError::Store(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
