use thiserror::Error as thisError;

#[derive(thisError, Debug)]
pub enum Error {
    #[error("database query error(sqlx): {0:?}")]
    Database(#[from] sqlx::Error),
    #[error("sqlx migration error: {0:?}")]
    DatabaseMigrate(#[from] sqlx::migrate::MigrateError),
    #[error("invalid request form. method={0:?} detail={1:?}")]
    BadRequest(&'static str, &'static str),
    #[error("not implemtned yet. method={0:?} detail={1:?}")]
    NotImplemented(&'static str, &'static str),
}
