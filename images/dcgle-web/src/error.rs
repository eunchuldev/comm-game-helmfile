use actix_web::{http::StatusCode, web::HttpResponse, ResponseError};
use thiserror::Error as thisError;

#[derive(thisError, Debug)]
pub enum Error {
    //#[error("{0:?}")]
    //Payload(#[from] io::Error),
    #[error("serde error: {0:?}")]
    SerdeError(#[from] serde_json::Error),
    #[error("io error: {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("sqlx error: {0:?}")]
    Sqlx(#[from] sqlx::Error),
    #[error("sqlx migration error: {0:?}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("search request form is not correct")]
    SearchRequestBadRequest,
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match self {
            Error::SerdeError(_) => StatusCode::BAD_REQUEST,
            Error::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Migrate(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SearchRequestBadRequest => StatusCode::BAD_REQUEST,
        }
    }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
