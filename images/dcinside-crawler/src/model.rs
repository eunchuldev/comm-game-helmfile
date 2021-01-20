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
    pub comments: Option<Vec<Comment>>,
    pub body: Option<String>,
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
        nickname: String, 
        #[serde(skip_deserializing)]
        ip: Option<bool>
    },
    Dynamic { 
        #[serde(deserialize_with = "skip_empty_str")]
        ip: String,
        #[serde(rename(deserialize = "name"))]
        nickname: String, 
        #[serde(skip_deserializing)]
        id: Option<bool>
    },
    Unknown {
        #[serde(rename(deserialize = "name"))]
        nickname: String, 
        #[serde(skip_deserializing)]
        ip: Option<bool>,
        #[serde(skip_deserializing)]
        id: Option<bool>
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
pub struct FromComment {
    #[serde(rename(deserialize = "no"), deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub id: usize,
    #[serde(flatten)]
    pub author: User,
    pub depth: usize,
    #[serde(rename(deserialize = "memo"))]
    pub contents: CommentContents,
    #[serde(rename(deserialize = "reg_date"), deserialize_with="comment_time")]
    pub created_at: Option<DateTime<Utc>>,
}
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(from="FromComment")]
pub struct Comment {
    pub id: usize,
    pub author: User,
    pub depth: usize,
    pub contents: CommentContents,
    pub parent_id: Option<usize>,
    pub created_at: Option<DateTime<Utc>>,
}
impl From<FromComment> for Comment {
    fn from(f: FromComment) -> Comment {
        Self {
            id: f.id,
            author: f.author,
            depth: f.depth,
            contents: f.contents,
            created_at: f.created_at,
            parent_id: None,
        }
    }
}


pub fn parse_document_body(
    body: &str,
    gallery_id: &str,
    document_id: usize,
) -> Result<String, DocumentBodyParseError> {
    let doc = HTMLDocument::from(body);
    Ok(doc
        .select(Class("write_div")).next().ok_or(DocumentParseError::Select { path: ".write_div", html: body.to_string() })?.inner_html())
        //.next().ok_or(DocumentParseError::Select { path: ".write_div" })?
        //.text().trim().to_string())
}


pub fn parse_document_indexes(
    body: &str,
    gallery_id: &str,
) -> Result<Vec<Result<DocumentIndex, DocumentParseError>>, DocumentParseError> {
    let doc = HTMLDocument::from(body);

    if body.starts_with("<script type=\"text/javascript\">location.replace(\"/error/adault") {
        return Err(DocumentParseError::AdultPage);
    }

    Ok(doc
        .select(Class("us-post"))
        .map(|node| -> Result<_, DocumentParseError> {
            let id = node
                .select(Class("gall_num"))
                .next().ok_or(DocumentParseError::Select { path: ".us-post .gall_num", html: body.to_string() })?
                .text().parse().map_err(|_| DocumentParseError::NumberParse { path: ".us-post .gall_num", })?;
            let title = node
                .select(Class("gall_tit").descendant(Name("a")))
                .next().ok_or(DocumentParseError::Select { path: ".us-post .gall_tit", html: body.to_string() })?
                .text();
            let subject = node.select(Class("gall_subject")).next().map(|n| n.text());
            let author = {
                let writer_node =
                    node.select(Class("gall_writer"))
                        .next().ok_or(DocumentParseError::Select { path: ".us-post .gall_writer", html: body.to_string() })?;
                let nickname = writer_node
                    .attr("data-nick").ok_or(DocumentParseError::Select { path: ".us-post .gall_writer@data-nick", html: body.to_string() })?;
                let ip = writer_node.attr("data-ip");
                let id = writer_node.attr("data-uid");
                match (id, ip) {
                    (Some(id), None) => Ok(User::Static { id: id.into(), nickname: nickname.into(), ip: None }),
                    (Some(id), Some(ip)) if ip.is_empty() => Ok(User::Static { id: id.into(), nickname: nickname.into(), ip: None }),
                    (None, Some(ip)) => Ok(User::Dynamic { ip: ip.into(), nickname: nickname.into(), id: None }),
                    (Some(id), Some(ip)) if id.is_empty() => Ok(User::Dynamic { ip: ip.into(), nickname: nickname.into(), id: None }),
                    _ => Err(DocumentParseError::Select { path: ".us-post .gall_writer@(data-ip | data-id)", html: body.to_string() }),
                }?
            };
            let comment_count = node
                .select(Class("reply_numbox"))
                .next()
                .map(|n| n.text().trim().trim_matches(|c| c == '[' || c == ']').parse().unwrap_or(0))
                .unwrap_or(0);
            let like_count = node
                .select(Class("gall_recommend"))
                .next().ok_or(DocumentParseError::Select { path: ".us-post .gall_recommned", html: body.to_string() })?
                .text().parse().map_err(|_| DocumentParseError::NumberParse { path: ".us-post .gall_recommend", })?;
            let view_count = node
                .select(Class("gall_count"))
                .next()
                .ok_or(DocumentParseError::Select { path: ".us-post .gall_count", html: body.to_string() })?
                .text()
                .parse()
                .map_err(|_| DocumentParseError::NumberParse { path: ".us-post .gall_count" })?;
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
                .next().ok_or(DocumentParseError::Select { path: ".us-post .gall_date", html: body.to_string() })?
                .attr("title").ok_or(DocumentParseError::Select { path: ".us-post .gall_date@title", html: body.to_string() })?;
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
    pagination: Option<String>,
}
pub fn parse_comments(
    body: &str,
    gallery_id: &str,
    document_id: usize,
    last_root_comment_id: Option<usize>,
) -> Result<(Vec<Comment>, usize), CommentParseError> {
    let body: _CommentsResponse = serde_json::from_str(body).map_err(|e| CommentParseError::JsonParse {
        source: e,
        target: body.to_string(),
        gallery_id: gallery_id.to_string(),
        doc_id: document_id,
    })?;
    match body.pagination {
        None => Ok((Vec::new(), 0)),
        Some(pagination) => {
            let doc = HTMLDocument::from(pagination.as_ref());
            let max_page = doc.select(Name("em").or(Name("a"))).map(|t| t.text().parse::<usize>().unwrap_or(0)).fold(0usize, |acc, x| if acc > x { acc } else { x });
            if let Some(mut comments) = body.comments {
                let mut last_root_comment_id = if let Some(id) = last_root_comment_id { id } else { 0usize };
                for c in comments.iter_mut() {
                    if c.depth == 0 {
                        last_root_comment_id = c.id;
                    } else if last_root_comment_id > 0 {
                        c.parent_id = Some(last_root_comment_id);
                    }
                }
                Ok((comments, max_page))
            } else {
                Ok((Vec::new(), max_page))
            }
        }
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
    fn it_parses_minor_document_indexes(){
        let res = parse_document_indexes(include_str!("../assets/minor_gallery.html"), "gallery_id").unwrap();
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
        let (res, max_page) = parse_comments(include_str!("../assets/comments.json"), "gallery_id", 1, None).unwrap();
        assert!(!res.is_empty());
        assert!(max_page == 10usize);
        assert!(res.len() >= 50);
        assert!(!res.iter().any(|c| match &c.author {
            User::Static { id, ..  } => id.is_empty(),
            User::Dynamic { ip, .. } => ip.is_empty(),
            User::Unknown { .. }=> { false }
        }));
    }
    #[test]
    fn it_deserializes_comments() {
        let (res, max_page) = parse_comments(include_str!("../assets/comments.json"), "gallery_id", 1, None).unwrap();
        let res = serde_json::to_string(&res[0]).unwrap();
        let expected = "{\"id\":13369033,\"author\":{\"ip\":\"119.195\",\"nickname\":\"ㅇㅇ\",\"id\":null},\"depth\":0,\"contents\":\"이제 뻑가,이슈왕 같은 랙카들이 청원하라고 한번 더 할듯ㅋㅋ  - dc App\",\"parent_id\":null,\"created_at\":\"2021-01-10T08:20:43Z\"}".to_string();
        assert_eq!(expected, res);
        //assert_eq!(!res[0], Comment{});
    }
    #[test]
    fn it_deserializes_minor_comments() {
        let (res, max_page) = parse_comments(include_str!("../assets/minor_comments.json"), "gallery_id", 1, None).unwrap();
        let res = serde_json::to_string(&res[0]).unwrap();
        let expected = "{\"id\":4649463,\"author\":{\"id\":\"nasdaqtrader\",\"nickname\":\"오함마의현인.\",\"ip\":null},\"depth\":0,\"contents\":\"개추\",\"parent_id\":null,\"created_at\":\"2020-12-31T07:44:47Z\"}".to_string();
        assert_eq!(expected, res);
        //assert_eq!(!res[0], Comment{});
    }

    #[test]
    fn it_parses_document_body() {
        let res = parse_document_body(include_str!("../assets/body.html"), "gallery_id", 1).unwrap();
        assert_eq!(res, "\n\t\t\t\t\t\t\t<p>\'올림픽\' 개최 위해 백신팀 극비리 가동<br><br>전국민 맞고 남을 만큼 확보했지만<br><br>\'국내 1~3차 임상 필수\' 규제에 발목<br><br>\"국산 백신은 왜 없나\" 비판도 일어<br><br>백신을 빨리 확보했음에도, 정작 접종시기는 2월말 - 한국과 차이 X<br><br>일본은 국내 임상 1,2,3차를 거쳐야 하는데, 모더나는 1월경 임상시험에 들어간다고 발표, 그러나 언제 접종할 지에 대해선 타임라인 제시 X</p><p><br></p><p><br></p><p style=\"text-align:left;\"><img src=\"https://dcimg8.dcinside.co.kr/viewimage.php?id=3dafdf21f7d335ab67b1d1&amp;no=24b0d769e1d32ca73dec87fa11d0283123a3619b5f9530e0a1306168e0dcca0e8d266e8bd0d7e56ed9364a956febc71a78fb28a8551648461846084596f34596bb9e7a7968b67c\" style=\"cursor:pointer;\" onclick=\"javascript:imgPop(\'https://image.dcinside.com/viewimagePop.php?id=3dafdf21f7d335ab67b1d1&amp;no=24b0d769e1d32ca73dec87fa11d0283123a3619b5f9530e0a1306168e0dcca0e8d266e8bd0d7e56ed9364ad33fbacc76da4d6aa29c4891b49fe3513541273923ef18\',\'image\',\'fullscreen=yes,scrollbars=yes,resizable=no,menubar=no,toolbar=no,location=no,status=no\');\" alt=\"viewimage.php?id=3dafdf21f7d335ab67b1d1&amp;no=24b0d769e1d32ca73dec87fa11d0283123a3619b5f9530e0a1306168e0dcca0e8d266e8bd0d7e56ed9364a956febc71a78fb28a8551648461846084596f34596bb9e7a7968b67c\"></p><p style=\"text-align:left;\">18시 현재 6,055명 (도쿄 1,494명 )<br><br>일요일 기준 최고치 갱신 중<br></p><p><br></p><p style=\"text-align:left;\"><img src=\"https://dcimg8.dcinside.co.kr/viewimage.php?id=3dafdf21f7d335ab67b1d1&amp;no=24b0d769e1d32ca73dec87fa11d0283123a3619b5f9530e0a1306168e0dcca0e8d266e8bd0d7e56ed9364a956febc71a78fb28a85516484618460810c0a219c177961ee0f408ae\" style=\"cursor:pointer;\" onclick=\"javascript:imgPop(\'https://image.dcinside.com/viewimagePop.php?id=3dafdf21f7d335ab67b1d1&amp;no=24b0d769e1d32ca73dec87fa11d0283123a3619b5f9530e0a1306168e0dcca0e8d266e8bd0d7e56ed9364ad33fbacc76da4d6aa29c48c4e2cebf0663432839231131\',\'image\',\'fullscreen=yes,scrollbars=yes,resizable=no,menubar=no,toolbar=no,location=no,status=no\');\" alt=\"viewimage.php?id=3dafdf21f7d335ab67b1d1&amp;no=24b0d769e1d32ca73dec87fa11d0283123a3619b5f9530e0a1306168e0dcca0e8d266e8bd0d7e56ed9364a956febc71a78fb28a85516484618460810c0a219c177961ee0f408ae\"></p><p><br></p>\t\t\t\t\t\t\t\t".to_string());
    }
}
