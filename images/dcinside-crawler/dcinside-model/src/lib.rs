use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
pub enum DocumentKind {
    Text,
    Picture,
    Video,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
pub enum GalleryKind {
    Major,
    Minor,
    Mini,
}
impl Default for GalleryKind {
    fn default() -> Self {
        GalleryKind::Major
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Gallery {
    pub id: String,
    pub name: String,
    pub kind: GalleryKind,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Document {
    pub gallery: Gallery,

    pub gallery_id: String,
    pub id: usize,
    pub title: String,
    pub subject: Option<String>,
    pub author: User,
    pub comment_count: u32,
    pub like_count: u32,
    pub view_count: u32,
    pub kind: DocumentKind,
    pub is_recommend: bool,
    pub created_at: DateTime<Utc>,

    pub comments: Option<Vec<Comment>>,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum UserKind {
    Static,
    Dynamic,
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub id: Option<String>,
    pub ip: Option<String>,
    pub nickname: String,
    pub kind: UserKind,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum CommentKind {
    Text,
    Con,
    Voice,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Comment {
    pub id: usize,
    pub author: User,
    pub depth: usize,
    pub contents: String,
    pub kind: CommentKind,
    pub parent_id: Option<usize>,
    pub created_at: Option<DateTime<Utc>>,
}
