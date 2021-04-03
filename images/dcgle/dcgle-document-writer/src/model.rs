use serde::Deserialize;
use dcgle_model::{Gallery, Document};
use chrono::{DateTime, Datelike, Duration, NaiveDateTime, TimeZone, Utc};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub struct User {
    id: Option<String>,
    nickname: Option<String>,
    ip: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Comment {
    pub id: usize,
    pub author: User,
    pub depth: usize,
    pub contents: CommentContents,
    pub parent_id: Option<usize>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Document {
    pub gallery_id: String,
    pub id: usize,
    pub title: String,
    pub subject: Option<String>,
    pub author: User,
    pub comment_count: usize,
    pub like_count: usize,
    pub view_count: usize,
    pub kind: DocumentKind,
    pub is_recommend: bool,
    pub created_at: DateTime<Utc>,

    pub comments: Option<Vec<Comment>>,
    pub body: Option<String>,
}
