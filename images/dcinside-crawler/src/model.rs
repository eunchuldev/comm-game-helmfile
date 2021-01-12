use crate::error::*;


use chrono::{DateTime, NaiveDateTime, TimeZone, Utc, Datelike};
use serde::{Deserialize, Serialize, Deserializer};

use select::document::Document as HTMLDocument;
use select::predicate::{Class, Name, Predicate};



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

#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
pub enum DocumentKind {
    Text,
    Picture,
    Video,
}
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct DocumentIndex {
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
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Document {
    #[serde(flatten)]
    pub index: DocumentIndex,
    pub comments: Vec<Comment>,
}

pub fn skip_empty_str<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Err(serde::de::Error::custom("empty str"))
    } else {
        Ok(s)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum User {
    Static { 
        #[serde(rename(deserialize = "user_id"), deserialize_with = "skip_empty_str")]
        id: String, 
        #[serde(rename(deserialize = "name"))]
        nickname: String 
    },
    Dynamic { 
        #[serde(deserialize_with = "skip_empty_str")]
        ip: String,
        #[serde(rename(deserialize = "name"))]
        nickname: String, 
    },
    Unknown {
        #[serde(rename(deserialize = "name"))]
        nickname: String, 
    },
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Clone)]
pub struct GalleryIndex {
    pub id: String,
    #[serde(alias = "ko_name")]
    pub name: String,
    #[serde(default)]
    pub kind: GalleryKind,
    pub rank: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum CommentContents {
    Text(String),
    Con(String),
    Voice(String),
}

impl<'de> Deserialize<'de> for CommentContents {
    fn deserialize<D>(des: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
        {
            let s = String::deserialize(des)?;
            Ok(if s.starts_with("<img") {
                CommentContents::Con(s)
            } else if s.starts_with("vr/") {
                CommentContents::Voice(s)
            } else {
                CommentContents::Text(s)
            })
        }
}


pub fn comment_time<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    let created_at_without_tz =
        NaiveDateTime::parse_from_str(&s, "%Y.%m.%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(&format!("{}.{}", Utc::now().with_timezone(&chrono_tz::Asia::Seoul).year(), s), "%Y.%m.%d %H:%M:%S"))
            .map_err(serde::de::Error::custom)?;
    Ok(Some(chrono_tz::Asia::Seoul
        .from_local_datetime(&created_at_without_tz)
        .unwrap().with_timezone(&Utc)))
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Comment {
    #[serde(rename(deserialize = "no"), deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub id: usize,
    #[serde(flatten)]
    pub author: User,
    pub depth: usize,
    #[serde(rename(deserialize = "memo"))]
    pub contents: CommentContents,
    #[serde(skip_deserializing)]
    pub parent_id: Option<usize>,
    #[serde(rename(deserialize = "reg_date"), deserialize_with="comment_time")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_deserializing)]
    pub document_id: usize,
    #[serde(skip_deserializing)]
    pub gallery_id: String,
}




pub fn parse_document_indexes(
    body: &str,
    gallery_id: &str,
) -> Result<Vec<Result<DocumentIndex, DocumentParseError>>, DocumentParseError> {
    let doc = HTMLDocument::from(body);
    Ok(doc
        .select(Class("us-post"))
        .map(|node| -> Result<_, DocumentParseError> {
            let id = node
                .select(Class("gall_num"))
                .next().ok_or(DocumentParseError::Select { path: ".us-post .gall_num", })?
                .text().parse().map_err(|_| DocumentParseError::NumberParse { path: ".us-post .gall_num", })?;
            let title = node
                .select(Class("gall_tit").descendant(Name("a")))
                .next().ok_or(DocumentParseError::Select { path: ".us-post .gall_tit", })?
                .text();
            let subject = node.select(Class("gall_subject")).next().map(|n| n.text());
            let author = {
                let writer_node =
                    node.select(Class("gall_writer"))
                        .next().ok_or(DocumentParseError::Select { path: ".us-post .gall_writer", })?;
                let nickname = writer_node
                    .attr("data-nick").ok_or(DocumentParseError::Select { path: ".us-post .gall_writer@data-nick", })?;
                let ip = writer_node.attr("data-ip");
                let id = writer_node.attr("data-uid");
                match (id, ip) {
                    (Some(id), None) => Ok(User::Static { id: id.into(), nickname: nickname.into(), }),
                    (Some(id), Some(ip)) if ip.is_empty() => Ok(User::Static { id: id.into(), nickname: nickname.into(), }),
                    (None, Some(ip)) => Ok(User::Dynamic { ip: ip.into(), nickname: nickname.into(), }),
                    (Some(id), Some(ip)) if id.is_empty() => Ok(User::Dynamic { ip: ip.into(), nickname: nickname.into(), }),
                    _ => Err(DocumentParseError::Select { path: ".us-post .gall_writer@(data-ip | data-id)", }),
                }?
            };
            let comment_count = node
                .select(Class("reply_numbox"))
                .next()
                .map(|n| n.text().trim().trim_matches(|c| c == '[' || c == ']').parse().unwrap_or(0))
                .unwrap_or(0);
            let like_count = node
                .select(Class("gall_recommend"))
                .next().ok_or(DocumentParseError::Select { path: ".us-post .gall_recommned", })?
                .text().parse().map_err(|_| DocumentParseError::NumberParse { path: ".us-post .gall_recommend", })?;
            let view_count = node
                .select(Class("gall_count"))
                .next()
                .ok_or(DocumentParseError::Select { path: ".us-post .gall_count", })?
                .text()
                .parse()
                .map_err(|_| DocumentParseError::NumberParse { path: ".us-post .gall_count", })?;
            let has_picture = node
                .select(Class("icon_pic"))
                .next().map(|_| true).unwrap_or(false);
            let has_video = node
                .select(Class("icon_movie"))
                .next().map(|_| true).unwrap_or(false);
            let kind = match (has_video, has_picture) {
                (true, false) => DocumentKind::Video,
                (false, true) => DocumentKind::Picture,
                _ => DocumentKind::Text,
            };
            let is_recommend = node
                .select(Class("icon_recom"))
                .next().map(|_| true).unwrap_or(false);
            let created_at_text = node
                .select(Class("gall_date"))
                .next().ok_or(DocumentParseError::Select { path: ".us-post .gall_date", })?
                .attr("title").ok_or(DocumentParseError::Select { path: ".us-post .gall_date@title", })?;
            let created_at_without_tz =
                NaiveDateTime::parse_from_str(created_at_text.trim(), "%Y-%m-%d %H:%M:%S")
                    .map_err(|_| DocumentParseError::DatetimeParse { path: ".us-post .gall_date@title", })?;
            let created_at = chrono_tz::Asia::Seoul
                .from_local_datetime(&created_at_without_tz)
                .unwrap().with_timezone(&Utc);
            Ok(DocumentIndex {
                id, title, subject, author, comment_count, 
                like_count, view_count, kind, is_recommend, created_at,
                gallery_id: gallery_id.to_string(),
            })
        })
        .collect())
}


#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct _CommentsResponse {
    comments: Option<Vec<Comment>>,
}
pub fn parse_comments(
    body: &str,
    gallery_id: &str,
    document_id: usize,
    last_root_comment_id: Option<usize>,
) -> Result<Vec<Comment>, CommentParseError> {
    let body: _CommentsResponse = serde_json::from_str(body).map_err(|e| CommentParseError::JsonParse {
        source: e,
        gallery_id: gallery_id.to_string(),
        doc_id: document_id,
    })?;
    if let Some(mut comments) = body.comments {
        let mut last_root_comment_id = if let Some(id) = last_root_comment_id { id } else { 0usize };
        for c in comments.iter_mut() {
            c.document_id = document_id;
            c.gallery_id = gallery_id.to_string();
            if c.depth == 0 {
                last_root_comment_id = c.id;
            } else if last_root_comment_id > 0 {
                c.parent_id = Some(last_root_comment_id);
            }
        }
        Ok(comments)
    } else {
        Ok(Vec::new())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GalleryState {
    pub index: GalleryIndex,
    pub last_ranked: DateTime<Utc>,
    pub last_crawled_at: Option<DateTime<Utc>>,
    pub last_crawled_document_id: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GalleryCrawlReportForm {
    pub id: String,
    pub last_crawled_at: Option<DateTime<Utc>>,
    pub last_crawled_document_id: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_parses_document_indexes() {
        let res = parse_document_indexes(include_str!("../assets/gallery.html"), "gallery_id").unwrap();
        let res: Vec<_> = res.into_iter().map(|d| d.unwrap()).collect();
        assert!(!res.is_empty());
        assert!(res.len() >= 20);
        assert!(res.iter().any(|d| d.comment_count > 0));
        assert!(res.iter().any(|d| if DocumentKind::Picture == d.kind {
            true
        } else {
            false
        }));
    }
    #[test]
    fn it_parses_comments() {
        let res = parse_comments(include_str!("../assets/comments.json"), "gallery_id", 1, None).unwrap();
        assert!(!res.is_empty());
        assert!(res.len() >= 50);
        assert!(!res.iter().any(|c| match &c.author {
            User::Static { id, ..  } => id.is_empty(),
            User::Dynamic { ip, .. } => ip.is_empty(),
            User::Unknown { .. }=> { false }
        }));
    }
}
