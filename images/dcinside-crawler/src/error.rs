use err_derive::Error;
use actix_web::client::{PayloadError, SendRequestError};


#[derive(Error, Debug)]
pub enum DocumentParseError {
    #[error(display = "fail to select `{}`", path)]
    Select { path: &'static str },
    #[error(display = "fail to parse `{}`", path)]
    NumberParse { path: &'static str },
    #[error(display = "fail to parse `{}`", path)]
    DatetimeParse { path: &'static str },
    #[error(display = "fail to parse" )]
    JsonParse(#[source] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum CommentParseError {
    #[error(display = "fail to select `{}`", path)]
    Select { path: &'static str },
    #[error(display = "fail to parse `{}`", path)]
    NumberParse { path: &'static str },
    #[error(display = "fail to parse `{}`", path)]
    DatetimeParse { path: &'static str },
    #[error(display = "fail to parse: {}.{}", gallery_id, doc_id )]
    JsonParse{
        source: serde_json::Error,
        doc_id: usize, 
        gallery_id: String,
    },
}

#[derive(Error, Debug)]
pub enum CrawlerError {
    #[error(display = "actix client send")]
    SendRequest(#[source] SendRequestError),
    #[error(display = "acitx client payload")]
    Payload(#[source] PayloadError),
    #[error(display = "serde")]
    Serde(#[source] serde_json::Error),
    #[error(display = "fmt")]
    Fmt(#[source] core::fmt::Error),
    #[error(display = "utf8")]
    Utf8(#[source] std::str::Utf8Error),
    #[error(display = "fail to parse root page")]
    DocumentParseError(#[source] DocumentParseError),
    #[error(display = "fail to parse root page")]
    CommentParseError(#[source] CommentParseError),
}

#[derive(Error, Debug)]
pub enum LiveDirectoryError {
    #[error(display = "crawler error")]
    Crawler(#[source] CrawlerError),
    #[error(display = "sled")]
    Sled(#[source] sled::Error),
}
