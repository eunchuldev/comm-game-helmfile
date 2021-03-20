use juniper::graphql_value;
use thiserror::Error as thisError;

#[derive(thisError, Debug)]
pub enum Error {
    #[error("database query error(sqlx): {0:?}")]
    Database(#[from] sqlx::Error),
    #[error("sqlx migration error: {0:?}")]
    DatabaseMigrate(#[from] sqlx::migrate::MigrateError),
    #[error("invalid request form. method={0:?} detail={1:?}")]
    BadRequest(&'static str, &'static str),
}

impl<S: juniper::ScalarValue> juniper::IntoFieldError<S> for Error {
    fn into_field_error(self) -> juniper::FieldError<S> {
        match self {
            Error::Database(e) => juniper::FieldError::new(
                "Database error",
                graphql_value!({
                    "detail": (e.to_string()),
                }),
            ),
            Error::BadRequest(method, detail) => juniper::FieldError::new(
                "Bad request",
                graphql_value!({
                    "method": method,
                    "detail": detail,
                }),
            ),
            Error::DatabaseMigrate(_) => panic!("graphql meet migration error"),
        }
    }
}
