use actix_web::client::{PayloadError, SendRequestError};
use err_derive::Error;

#[derive(Error, Debug)]
pub enum DocumentParseError {
    #[error(display = "fail to select `{}`: html: {}", path, html)]
    Select { path: &'static str, html: String },
    #[error(display = "fail to parse `{}`", path)]
    NumberParse { path: &'static str },
    #[error(display = "fail to parse `{}`", path)]
    DatetimeParse { path: &'static str },
    #[error(display = "fail to parse")]
    JsonParse(#[source] serde_json::Error),
    #[error(display = "need adult auth")]
    AdultPage,
    #[error(display = "closed minor gallery")]
    MinorGalleryClosed,
    #[error(display = "minor gallery become major gallery")]
    MinorGalleryPromoted,
    #[error(display = "minor gallery access not allowed")]
    MinorGalleryAccessNotAllowed,
}

#[derive(Error, Debug)]
pub enum CommentParseError {
    #[error(display = "fail to select `{}`", path)]
    Select { path: &'static str },
    #[error(display = "fail to parse `{}`", path)]
    NumberParse { path: &'static str },
    #[error(display = "fail to parse `{}`", path)]
    DatetimeParse { path: &'static str },
    #[error(
        display = "fail to parse at {}.{} due to {}. body: {}",
        gallery_id,
        doc_id,
        source,
        target
    )]
    JsonParse {
        source: serde_json::Error,
        target: String,
        doc_id: usize,
        gallery_id: String,
    },
}
#[derive(Error, Debug)]
pub enum DocumentBodyParseError {
    #[error(display = "fail to select `{}`: html: {}", path, html)]
    Select { path: &'static str, html: String },
    #[error(display = "fail to parse page: {}", _0)]
    DocumentParseError(#[source] DocumentParseError),
}

#[derive(Error, Debug)]
pub enum CrawlerError {
    #[error(display = "actix client send: {}", _0)]
    SendRequest(#[source] SendRequestError),
    #[error(display = "acitx client payload: {}", _0)]
    Payload(#[source] PayloadError),
    #[error(display = "serde: {}", _0)]
    Serde(#[source] serde_json::Error),
    #[error(display = "fmt: {}", _0)]
    Fmt(#[source] core::fmt::Error),
    #[error(display = "utf8: {}", _0)]
    Utf8(#[source] std::str::Utf8Error),
    #[error(display = "fail to parse root page: {}", _0)]
    DocumentParseError(#[source] DocumentParseError),
    #[error(display = "fail to parse comment: {}", _0)]
    CommentParseError(#[source] CommentParseError),
    #[error(display = "fail to parse body: {}", _0)]
    DocumentBodyParseError(#[source] DocumentBodyParseError),
}

#[derive(Error, Debug)]
pub enum LiveDirectoryError {
    #[error(display = "crawler error")]
    Crawler(#[source] CrawlerError),
    #[error(display = "sled")]
    Sled(#[source] sled::Error),
}

#[derive(Error, Debug)]
pub enum BackOffError {
    #[error(display = "backoff error: {}", _0)]
    CrawlerError(#[source] CrawlerError),
    #[error(display = "backoff error(break): {}", _0)]
    Break(CrawlerError),
}
impl From<BackOffError> for CrawlerError {
    fn from(f: BackOffError) -> CrawlerError {
        match f {
            BackOffError::CrawlerError(e) => e,
            BackOffError::Break(e) => e,
        }
    }
}
