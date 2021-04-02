use crate::error::*;

use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize, de::{MapAccess, Visitor, self, IgnoredAny}};

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
impl From<&str> for GalleryKind {
    fn from(s: &str) -> Self {
        match s {
            "major" => GalleryKind::Major,
            "minor" => GalleryKind::Minor,
            "mini" => GalleryKind::Mini,
            _ => panic!("unsupported gallery kind"),
        }
    }
}
impl From<String> for GalleryKind {
    fn from(s: String) -> Self {
        match s.as_ref() {
            "major" => GalleryKind::Major,
            "minor" => GalleryKind::Minor,
            "mini" => GalleryKind::Mini,
            _ => panic!("unsupported gallery kind"),
        }
    }
}
impl From<GalleryKind> for &'static str {
    fn from(k: GalleryKind) -> &'static str {
        match k {
            GalleryKind::Major => "major",
            GalleryKind::Minor => "minor",
            GalleryKind::Mini => "mini",
        }
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
    pub comment_count: u32,
    pub like_count: u32,
    pub view_count: u32,
    pub kind: DocumentKind,
    pub is_recommend: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Gallery {
    pub id: String,
    pub name: String,
    pub kind: GalleryKind,
}

impl From<GalleryIndex> for Gallery {
    fn from(o: GalleryIndex) -> Self {
        Self {
            id: o.id,
            name: o.name,
            kind: o.kind,
        }
    }
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

impl Document {
    pub fn from_indexes(gallery_index: GalleryIndex, doc_index: DocumentIndex, comments: Option<Vec<Comment>>, body: Option<String>) -> Self {
        Self{
            gallery: gallery_index.into(),

            gallery_id: doc_index.gallery_id,
            id: doc_index.id,
            title: doc_index.title,
            subject: doc_index.subject,
            author: doc_index.author,
            comment_count: doc_index.comment_count,
            like_count: doc_index.like_count,
            view_count: doc_index.view_count,
            kind: doc_index.kind,
            is_recommend: doc_index.is_recommend,
            created_at: doc_index.created_at,

            comments,
            body
        }
    }
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
pub enum UserKind { Static, Dynamic, Unknown }

impl UserKind {
    fn parse<T1: AsRef<str>, T2: AsRef<str>>(id: &Option<T1>, ip: &Option<T2>) -> Self {
        match (id, ip) {
            (Some(t), None) if !t.as_ref().is_empty() => Self::Static,
            (Some(t), Some(u)) if !t.as_ref().is_empty() && u.as_ref().is_empty() => Self::Static,
            (None, Some(t)) if !t.as_ref().is_empty() => Self::Dynamic,
            (Some(t), Some(u)) if t.as_ref().is_empty() && !u.as_ref().is_empty() => Self::Dynamic,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub id: Option<String>,
    pub ip: Option<String>,
    pub nickname: String,
    pub kind: UserKind,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Clone)]
pub struct GalleryIndex {
    pub id: String,
    #[serde(alias = "ko_name")]
    pub name: String,
    #[serde(default)]
    pub kind: GalleryKind,
    #[serde(
        default,
        deserialize_with = "serde_aux::field_attributes::deserialize_option_number_from_string"
    )]
    pub rank: Option<usize>,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum CommentKind { Text, Con, Voice }

impl CommentKind {
    fn from_contents(contents: &str) -> Self {
        if contents.starts_with("<img") {
            Self::Con
        } else if contents.starts_with("vr/") {
            Self::Voice
        } else {
            Self::Text
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Comment {
    pub id: usize,
    pub author: User,
    pub depth: usize,
    pub contents: String,
    pub kind: CommentKind,
    pub parent_id: Option<usize>,
    pub created_at: Option<DateTime<Utc>>,
}

impl<'de> Deserialize<'de> for Comment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrInt {
            String(String),
            Number(usize),
        }

        
        struct CommentVisitor;

        impl<'de> Visitor<'de> for CommentVisitor {
            type Value = Comment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Comment")
            }
            
            fn visit_map<V>(self, mut map: V) -> Result<Comment, V::Error>
                where
                    V: MapAccess<'de>,
                {
                    let mut id: Option<StringOrInt> = None;
                    let mut author_id: Option<String> = None;
                    let mut author_ip: Option<String> = None;
                    let mut author_nickname = None;
                    let mut depth = None;
                    let mut contents = None;
                    let mut kind = None;
                    let mut created_at = None;
                    while let Some(key) = map.next_key()? {
                        match key {
                            "no" => { id = Some(map.next_value()?); }
                            "user_id" => { author_id = Some(map.next_value()?); }
                            "ip" => { author_ip = Some(map.next_value()?); }
                            "name" => { author_nickname = Some(map.next_value()?); }
                            "depth" => { depth = Some(map.next_value()?); }
                            "memo" => {
                                let val = map.next_value::<String>()?;
                                kind = Some(CommentKind::from_contents(&val));
                                contents = Some(val);
                            }
                            "reg_date" => {
                                created_at = Some(map.next_value::<String>()?);
                            }
                            "comment_date" => {
                                created_at = Some(map.next_value::<String>()?);
                            }
                            _ => { map.next_value::<IgnoredAny>()?; }
                        }
                    }
                    let created_at = created_at.and_then(|s| {
                        NaiveDateTime::parse_from_str(&s, "%Y.%m.%d %H:%M:%S")
                            .or_else(|_| {
                                NaiveDateTime::parse_from_str(
                                    &format!(
                                        "{}.{}",
                                        Utc::now().with_timezone(&chrono_tz::Asia::Seoul).year(),
                                        s
                                    ),
                                    "%Y.%m.%d %H:%M:%S",
                                )
                            })
                            .map(|created_at_without_tz|
                                chrono_tz::Asia::Seoul
                                    .from_local_datetime(&created_at_without_tz)
                                    .unwrap()
                                    .with_timezone(&Utc))
                            .ok()
                        });
                    Ok(Comment {
                        id: match id.ok_or_else(|| de::Error::missing_field("no"))? {
                            StringOrInt::String(s) => s.parse().map_err(|_| de::Error::custom("field `no` should number_string"))?,
                            StringOrInt::Number(u) => u,
                        },
                        author: User {
                            kind: UserKind::parse(&author_id, &author_ip),
                            nickname: author_nickname.ok_or_else(|| de::Error::missing_field("name"))?,
                            ip: author_ip.and_then(|i| if i.is_empty() { None } else { Some(i) }),
                            id: author_id.and_then(|i| if i.is_empty() { None } else { Some(i) }),
                        },
                        depth: depth.ok_or_else(|| de::Error::missing_field("depth"))?,
                        contents: contents.ok_or_else(|| de::Error::missing_field("memo"))?,
                        kind: kind.ok_or_else(|| de::Error::missing_field("memo"))?,
                        created_at,
                        parent_id: None,
                    })
                }
        }
        const FIELDS: &[&str] = &["id", "author", "depth", "contents", "kind", "created_at", "parent_id"];
        deserializer.deserialize_struct("Comment", FIELDS, CommentVisitor)
    }
}


pub fn parse_document_body(
    body: &str,
    _gallery_id: &str,
    _document_id: usize,
) -> Result<String, DocumentBodyParseError> {
    let doc = HTMLDocument::from(body);
    Ok(doc
        .select(Class("write_div"))
        .next()
        .ok_or(DocumentParseError::Select {
            path: ".write_div",
            html: body.to_string(),
        })?
        .inner_html())
    //.next().ok_or(DocumentParseError::Select { path: ".write_div" })?
    //.text().trim().to_string())
}

pub fn parse_document_indexes(
    body: &str,
    gallery_id: &str,
) -> Result<Vec<Result<DocumentIndex, DocumentParseError>>, DocumentParseError> {
    let doc = HTMLDocument::from(body);

    if body.starts_with(r#"<script type="text/javascript">location.replace("/error/adult"#) {
        return Err(DocumentParseError::AdultPage);
    } else if body.starts_with("<script type=\"text/javascript\">alert(\"해당 마이너 갤러리는 매니저의 요청으로 폐쇄되었습니다.") {
        return Err(DocumentParseError::MinorGalleryClosed);
    } else if body.starts_with("<script type=\"text/javascript\">alert(\"해당 마이너 갤러리는 운영원칙 위반") {
        if body.contains("폐쇄") {
            return Err(DocumentParseError::MinorGalleryClosed);
        } else {
            return Err(DocumentParseError::MinorGalleryAccessNotAllowed);
        }
    } else if body.starts_with("<script type=\"text/javascript\">location.replace(\"https://gall.dcinside.com/board/lists?") {
        return Err(DocumentParseError::MinorGalleryPromoted);
    } else if let Some(node) = doc.select(Class("migall_state")).next() {
        if let Some(c) = node.attr("class") {
            if c.contains("restriction") {
                return Err(DocumentParseError::MinorGalleryAccessNotAllowed);
            }
        }
    }

    Ok(doc
        .select(Class("us-post"))
        .map(|node| -> Result<_, DocumentParseError> {
            let id = node
                .select(Class("gall_num"))
                .next()
                .ok_or(DocumentParseError::Select {
                    path: ".us-post .gall_num",
                    html: body.to_string(),
                })?
                .text()
                .parse()
                .map_err(|_| DocumentParseError::NumberParse {
                    path: ".us-post .gall_num",
                })?;
            let title = node
                .select(Class("gall_tit").descendant(Name("a")))
                .next()
                .ok_or(DocumentParseError::Select {
                    path: ".us-post .gall_tit",
                    html: body.to_string(),
                })?
                .text();
            let subject = node.select(Class("gall_subject")).next().map(|n| n.text());
            let author = {
                let writer_node =
                    node.select(Class("gall_writer"))
                        .next()
                        .ok_or(DocumentParseError::Select {
                            path: ".us-post .gall_writer",
                            html: body.to_string(),
                        })?;
                let nickname = writer_node
                    .attr("data-nick")
                    .ok_or(DocumentParseError::Select {
                        path: ".us-post .gall_writer@data-nick",
                        html: body.to_string(),
                    })?;
                let ip = writer_node.attr("data-ip");
                let id = writer_node.attr("data-uid");
                User {
                    nickname: nickname.into(),
                    kind: UserKind::parse(&id, &ip),
                    id: id.and_then(|i| if i.is_empty() { None } else { Some(i.to_owned()) }),
                    ip: ip.and_then(|i| if i.is_empty() { None } else { Some(i.to_owned()) }),
                }
            };
            let comment_count = node
                .select(Class("reply_numbox"))
                .next()
                .map(|n| {
                    n.text()
                        .trim()
                        .trim_matches(|c| c == '[' || c == ']')
                        .parse()
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            let like_count = node
                .select(Class("gall_recommend"))
                .next()
                .ok_or(DocumentParseError::Select {
                    path: ".us-post .gall_recommned",
                    html: body.to_string(),
                })?
                .text()
                .parse()
                .map_err(|_| DocumentParseError::NumberParse {
                    path: ".us-post .gall_recommend",
                })?;
            let view_count = node
                .select(Class("gall_count"))
                .next()
                .ok_or(DocumentParseError::Select {
                    path: ".us-post .gall_count",
                    html: body.to_string(),
                })?
                .text()
                .parse()
                .map_err(|_| DocumentParseError::NumberParse {
                    path: ".us-post .gall_count",
                })?;
            let has_picture = node
                .select(Class("icon_pic"))
                .next()
                .map(|_| true)
                .unwrap_or(false);
            let has_video = node
                .select(Class("icon_movie"))
                .next()
                .map(|_| true)
                .unwrap_or(false);
            let kind = match (has_video, has_picture) {
                (true, false) => DocumentKind::Video,
                (false, true) => DocumentKind::Picture,
                _ => DocumentKind::Text,
            };
            let is_recommend = node
                .select(Class("icon_recom"))
                .next()
                .map(|_| true)
                .unwrap_or(false);
            let created_at_text = node
                .select(Class("gall_date"))
                .next()
                .ok_or(DocumentParseError::Select {
                    path: ".us-post .gall_date",
                    html: body.to_string(),
                })?
                .attr("title")
                .ok_or(DocumentParseError::Select {
                    path: ".us-post .gall_date@title",
                    html: body.to_string(),
                })?;
            let created_at_without_tz =
                NaiveDateTime::parse_from_str(created_at_text.trim(), "%Y-%m-%d %H:%M:%S")
                    .map_err(|_| DocumentParseError::DatetimeParse {
                        path: ".us-post .gall_date@title",
                    })?;
            let created_at = chrono_tz::Asia::Seoul
                .from_local_datetime(&created_at_without_tz)
                .unwrap()
                .with_timezone(&Utc);
            Ok(DocumentIndex {
                id,
                title,
                subject,
                author,
                comment_count,
                like_count,
                view_count,
                kind,
                is_recommend,
                created_at,
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
    let body: _CommentsResponse =
        serde_json::from_str(body).map_err(|e| CommentParseError::JsonParse {
            source: e,
            target: body.to_string(),
            gallery_id: gallery_id.to_string(),
            doc_id: document_id,
        })?;
    match body.pagination {
        None => Ok((Vec::new(), 0)),
        Some(pagination) => {
            let doc = HTMLDocument::from(pagination.as_ref());
            let max_page = doc
                .select(Name("em").or(Name("a")))
                .map(|t| t.text().parse::<usize>().unwrap_or(0))
                .fold(0usize, |acc, x| if acc > x { acc } else { x });
            if let Some(mut comments) = body.comments {
                let mut last_root_comment_id = if let Some(id) = last_root_comment_id {
                    id
                } else {
                    0usize
                };
                for c in comments.iter_mut() {
                    if c.depth == 0 && c.id > 0{
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

fn default_as_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GalleryState {
    pub index: GalleryIndex,
    pub last_ranked: DateTime<Utc>,
    pub last_crawled_at: Option<DateTime<Utc>>,
    pub last_crawled_document_id: Option<usize>,
    #[serde(default = "default_as_true")]
    pub visible: bool,
    #[serde(default)]
    pub last_error: Option<CrawlerErrorReport>,
    #[serde(default)]
    pub publish_duration_in_seconds: Option<f64>,
    #[serde(default)]
    pub last_published_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub registered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GalleryCrawlReportForm {
    pub worker_part: u64,
    pub id: String,
    pub last_crawled_at: Option<DateTime<Utc>>,
    pub last_crawled_document_id: Option<usize>,
    pub crawled_document_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CrawlerErrorReport {
    Unknown,
    AdultPage,
    MinorGalleryAccessNotAllowed,
    MinorGalleryClosed,
    MinorGalleryPromoted,
    PageNotFound,
}

impl From<&CrawlerError> for CrawlerErrorReport {
    fn from(err: &CrawlerError) -> Self {
        match err {
            CrawlerError::DocumentParseError(DocumentParseError::MinorGalleryPromoted) => {
                CrawlerErrorReport::MinorGalleryPromoted
            }
            CrawlerError::DocumentParseError(DocumentParseError::AdultPage) => {
                CrawlerErrorReport::AdultPage
            }
            CrawlerError::DocumentParseError(DocumentParseError::MinorGalleryClosed) => {
                CrawlerErrorReport::MinorGalleryClosed
            }
            CrawlerError::DocumentParseError(DocumentParseError::MinorGalleryAccessNotAllowed) => {
                CrawlerErrorReport::MinorGalleryAccessNotAllowed
            }
            CrawlerError::PageNotFound => CrawlerErrorReport::PageNotFound,
            _ => CrawlerErrorReport::Unknown,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GalleryCrawlErrorReportForm {
    pub worker_part: u64,
    pub id: String,
    pub last_crawled_at: Option<DateTime<Utc>>,
    pub error: CrawlerErrorReport,
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_err {
        ($expression:expr, $($pattern:tt)+) => {
            match $expression {
                $($pattern)+ => (),
                ref e => panic!("expected `{}` but got `{:?}`", stringify!($($pattern)+), e),
            }
        }
    }

    #[test]
    fn it_parses_document_indexes() {
        let res =
            parse_document_indexes(include_str!("../assets/gallery.html"), "gallery_id").unwrap();
        let res: Vec<_> = res.into_iter().map(|d| d.unwrap()).collect();
        assert!(!res.is_empty());
        assert!(res.len() >= 20);
        assert!(res.iter().any(|d| d.comment_count > 0));
        assert!(res.iter().any(|d| DocumentKind::Picture == d.kind));
    }
    #[test]
    fn it_parses_problmetic_gallery1() {
        let res =
            parse_document_indexes(include_str!("../assets/gallery-problemtic.html"), "gallery_id").unwrap();
        let res: Vec<_> = res.into_iter().map(|d| d.unwrap()).collect();
        assert!(!res.is_empty());
        assert!(res.len() >= 20);
        assert!(res.iter().any(|d| d.comment_count > 0));
        assert!(res.iter().any(|d| DocumentKind::Picture == d.kind));
    }

    #[test]
    fn it_parses_minor_document_indexes() {
        let res =
            parse_document_indexes(include_str!("../assets/minor_gallery.html"), "gallery_id")
                .unwrap();
        let res: Vec<_> = res.into_iter().map(|d| d.unwrap()).collect();
        assert!(!res.is_empty());
        assert!(res.len() >= 20);
        assert!(res.iter().any(|d| d.comment_count > 0));
        assert!(res.iter().any(|d| DocumentKind::Picture == d.kind));
    }
    #[test]
    fn it_parses_comments() {
        let (res, max_page) = parse_comments(
            include_str!("../assets/comments.json"),
            "gallery_id",
            1,
            None,
        )
        .unwrap();
        assert!(!res.is_empty());
        assert!(max_page == 10usize);
        assert!(res.len() >= 50);
        for c in res {
            match c.author.kind {
                UserKind::Static => assert!(c.author.id.is_some()),
                UserKind::Dynamic  => assert!(c.author.ip.is_some()),
                UserKind::Unknown  => assert_eq!(c.author.nickname, "댓글돌이".to_string()),
            }
        }
    }
    #[test]
    fn it_deserializes_comments() {
        let (res, _max_page) = parse_comments(
            include_str!("../assets/comments.json"),
            "gallery_id",
            1,
            None,
        )
        .unwrap();
        let res = serde_json::to_string(&res[0]).unwrap();
        let expected = "{\"id\":13369033,\"author\":{\"id\":null,\"ip\":\"119.195\",\"nickname\":\"ㅇㅇ\",\"kind\":\"Dynamic\"},\"depth\":0,\"contents\":\"이제 뻑가,이슈왕 같은 랙카들이 청원하라고 한번 더 할듯ㅋㅋ  - dc App\",\"kind\":\"Text\",\"parent_id\":null,\"created_at\":\"2021-01-10T08:20:43Z\"}".to_string();
        assert_eq!(expected, res);
        //assert_eq!(!res[0], Comment{});
    }
    #[test]
    fn it_deserializes_minor_comments() {
        let (res, _max_page) = parse_comments(
            include_str!("../assets/minor_comments.json"),
            "gallery_id",
            1,
            None,
        )
        .unwrap();
        let res = serde_json::to_string(&res[0]).unwrap();
        let expected = "{\"id\":4649463,\"author\":{\"id\":\"nasdaqtrader\",\"ip\":null,\"nickname\":\"오함마의현인.\",\"kind\":\"Static\"},\"depth\":0,\"contents\":\"개추\",\"kind\":\"Text\",\"parent_id\":null,\"created_at\":\"2020-12-31T07:44:47Z\"}".to_string();
        assert_eq!(expected, res);
        //assert_eq!(!res[0], Comment{});
    }

    #[test]
    fn it_parses_document_body() {
        let res =
            parse_document_body(include_str!("../assets/body.html"), "gallery_id", 1).unwrap();
        assert_eq!(res, "\n\t\t\t\t\t\t\t<p>\'올림픽\' 개최 위해 백신팀 극비리 가동<br><br>전국민 맞고 남을 만큼 확보했지만<br><br>\'국내 1~3차 임상 필수\' 규제에 발목<br><br>\"국산 백신은 왜 없나\" 비판도 일어<br><br>백신을 빨리 확보했음에도, 정작 접종시기는 2월말 - 한국과 차이 X<br><br>일본은 국내 임상 1,2,3차를 거쳐야 하는데, 모더나는 1월경 임상시험에 들어간다고 발표, 그러나 언제 접종할 지에 대해선 타임라인 제시 X</p><p><br></p><p><br></p><p style=\"text-align:left;\"><img src=\"https://dcimg8.dcinside.co.kr/viewimage.php?id=3dafdf21f7d335ab67b1d1&amp;no=24b0d769e1d32ca73dec87fa11d0283123a3619b5f9530e0a1306168e0dcca0e8d266e8bd0d7e56ed9364a956febc71a78fb28a8551648461846084596f34596bb9e7a7968b67c\" style=\"cursor:pointer;\" onclick=\"javascript:imgPop(\'https://image.dcinside.com/viewimagePop.php?id=3dafdf21f7d335ab67b1d1&amp;no=24b0d769e1d32ca73dec87fa11d0283123a3619b5f9530e0a1306168e0dcca0e8d266e8bd0d7e56ed9364ad33fbacc76da4d6aa29c4891b49fe3513541273923ef18\',\'image\',\'fullscreen=yes,scrollbars=yes,resizable=no,menubar=no,toolbar=no,location=no,status=no\');\" alt=\"viewimage.php?id=3dafdf21f7d335ab67b1d1&amp;no=24b0d769e1d32ca73dec87fa11d0283123a3619b5f9530e0a1306168e0dcca0e8d266e8bd0d7e56ed9364a956febc71a78fb28a8551648461846084596f34596bb9e7a7968b67c\"></p><p style=\"text-align:left;\">18시 현재 6,055명 (도쿄 1,494명 )<br><br>일요일 기준 최고치 갱신 중<br></p><p><br></p><p style=\"text-align:left;\"><img src=\"https://dcimg8.dcinside.co.kr/viewimage.php?id=3dafdf21f7d335ab67b1d1&amp;no=24b0d769e1d32ca73dec87fa11d0283123a3619b5f9530e0a1306168e0dcca0e8d266e8bd0d7e56ed9364a956febc71a78fb28a85516484618460810c0a219c177961ee0f408ae\" style=\"cursor:pointer;\" onclick=\"javascript:imgPop(\'https://image.dcinside.com/viewimagePop.php?id=3dafdf21f7d335ab67b1d1&amp;no=24b0d769e1d32ca73dec87fa11d0283123a3619b5f9530e0a1306168e0dcca0e8d266e8bd0d7e56ed9364ad33fbacc76da4d6aa29c48c4e2cebf0663432839231131\',\'image\',\'fullscreen=yes,scrollbars=yes,resizable=no,menubar=no,toolbar=no,location=no,status=no\');\" alt=\"viewimage.php?id=3dafdf21f7d335ab67b1d1&amp;no=24b0d769e1d32ca73dec87fa11d0283123a3619b5f9530e0a1306168e0dcca0e8d266e8bd0d7e56ed9364a956febc71a78fb28a85516484618460810c0a219c177961ee0f408ae\"></p><p><br></p>\t\t\t\t\t\t\t\t".to_string());
    }

    #[test]
    fn it_parses_adults() {
        let html = r#"<script type="text/javascript">location.replace("/error/adult/?s_url=https%3A%2F%2Fgall.dcinside.com%2Fmgallery%2Fboard%2Flists%3Fid%3Donahole%26list_num%3D100%26page%3D2");</script>"#;
        let res = parse_document_indexes(html, "gallery_id");
        assert_err!(res, Err(DocumentParseError::AdultPage));
    }

    #[test]
    fn it_parses_prohibited() {
        let res = parse_document_indexes(include_str!("../assets/prohibited.html"), "gallery_id");
        assert_err!(res, Err(DocumentParseError::MinorGalleryAccessNotAllowed));
    }

    #[test]
    fn it_parses_promoted() {
        let res = parse_document_indexes(
            r#"<script type="text/javascript">location.replace("https://gall.dcinside.com/board/lists?id=wln");</script>"#,
            "gallery_id",
        );
        assert_err!(res, Err(DocumentParseError::MinorGalleryPromoted));
    }

    #[test]
    fn it_pareses_closed() {
        let res = parse_document_indexes(
            r#"<script type="text/javascript">alert("해당 마이너 갤러리는 매니저의 요청으로 폐쇄되었습니다.\n마이너 갤러리 메인으로 돌아갑니다.");</script><script type="text/javascript">location.replace("https://gall.dcinside.com/m");</script>"#,
            "gallery_id",
        );
        assert_err!(res, Err(DocumentParseError::MinorGalleryClosed));
    }

    #[test]
    fn it_pareses_closed2() {
        let res = parse_document_indexes(
            r#"<script type="text/javascript">alert("해당 마이너 갤러리는 운영원칙 위반(사유: )으로 폐쇄되었습니다.\n마이너 갤러리 메인으로 돌아갑니다.");</script><script type="text/javascript">location.replace("https://gall.dcinside.com/m");</script>"#,
            "gallery_id",
        );
        assert_err!(res, Err(DocumentParseError::MinorGalleryClosed));
    }

}
