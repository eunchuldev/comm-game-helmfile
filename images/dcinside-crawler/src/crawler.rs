use crate::model::*;
use crate::error::*;

use actix_web::client::{Client, ClientBuilder};

use serde::{Deserialize, Serialize};


use std::time::Duration;
use chrono::Utc;


use select::document::Document as HTMLDocument;
use select::predicate::{Attr};






use exponential_backoff::Backoff;


#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct CommentsPostForm<'a> {
    id: &'a str,
    no: usize,
    cmt_id: &'a str,
    cmt_no: usize,
    e_s_n_o: Option<&'a str>,
    comment_page: usize,
    sort: &'a str,
    prevCnt: usize,
    _GALLTYPE_: &'a str,
}


#[derive(Clone)]
pub struct Crawler {
    pub client: Client,
    host: String,
    backoff: Backoff,
    e_s_n_o: Option<String>,
    delay: Duration,
}
impl<'a> Crawler {
    pub fn new() -> Self {
        let client = ClientBuilder::new()
            .header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:84.0) Gecko/20100101 Firefox/84.0",
            )
            .finish();
        Crawler {
            client,
            host: String::from("https://gall.dcinside.com"),
            backoff: Backoff::new(8, Duration::from_millis(100), Duration::from_secs(10)),
            e_s_n_o: None,
            delay: Duration::from_millis(300),
        }
    }
    pub async fn realtime_hot_galleries(&self) -> Result<Vec<GalleryIndex>, CrawlerError> {
        let jsonp_callback_func = format!(
            "jQuery3210837750950307798_{}",
            Utc::now().timestamp_millis()
        );
        let path = format!(
            "https://json2.dcinside.com/json1/ranking_gallery.php?jsoncallback={}&_={}",
            jsonp_callback_func,
            Utc::now().timestamp_millis()
        );
        let bytes = backoff::tokio::retry(backoff::ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .get(path.as_str())
                .header("Referer", "https://gall.dcinside.com/")
                .send().await.map_err(CrawlerError::from)?
                .body().await.map_err(CrawlerError::from)?)
        })
        .await?;
        let text = std::str::from_utf8(&bytes)?;
        let trimed = text.trim();
        let jsonp_contents = &trimed[jsonp_callback_func.len() + 1..trimed.len() - 1];
        let mut galleries: Vec<GalleryIndex> = serde_json::from_str(&jsonp_contents)?;
        for g in galleries.iter_mut() {
            g.kind = GalleryKind::Major;
        }
        Ok(galleries)
    }
    pub async fn document_indexes_after(
        &mut self,
        gallery: &GalleryIndex,
        last_document_id: usize,
        start_page: usize,
    ) -> Result<Vec<Result<DocumentIndex, DocumentParseError>>, CrawlerError> {
        let mut docs = Vec::new();
        for i in start_page..1000 {
            let next_docs = self.document_indexes(gallery, i).await?;
            if next_docs.is_empty() {
                break;
            } 
            if !next_docs.iter().any(|d| d.is_ok()) {
                break;
            }
            docs.extend(next_docs);
            if docs.iter().rev().find_map(|d| d.as_ref().ok()).unwrap().id <= last_document_id {
                break;
            }
            actix::clock::delay_for(self.delay).await;
        }
        Ok(docs.into_iter().filter(|t| match t { Ok(d) => d.id > last_document_id, Err(_e) => true } ).collect())
    }
    pub async fn comments(
        &mut self,
        gallery: &GalleryIndex,
        doc_id: usize,
    ) -> Result<Vec<Comment>, CrawlerError> {
        let mut comms = Vec::new();
        for i in 1..1000 {
            let next_comms = self._comments(&gallery, doc_id, i, None).await?;
            if next_comms.is_empty() {
                break;
            }
            actix::clock::delay_for(self.delay).await;
            comms.extend(next_comms.into_iter().rev());
        }
        Ok(comms.into_iter().rev().collect())
    }
    pub async fn documents(
        &mut self,
        gallery: &GalleryIndex,
        page: usize
    ) -> Result<Vec<Result<Document, CrawlerError>>, CrawlerError> {
        let mut documents = Vec::new();
        for res in self.document_indexes(&gallery, page).await? {
            match res {
                Ok(index) => {
                    let id = index.id;
                    if index.comment_count > 0 {
                        match self.comments(&gallery, id).await {
                            Ok(comms) => documents.push(Ok(Document { index: index, comments: comms })),
                            Err(err) => documents.push(Err(err.into())),
                        };
                    } else {
                        documents.push(Ok(Document { index: index, comments: Vec::new() }));
                    }
                },
                Err(err) => documents.push(Err(err.into())),
            };
        }
        Ok(documents)
    }
    pub async fn documents_after(
        &mut self,
        gallery: &GalleryIndex,
        last_document_id: usize,
        start_page: usize,
    ) -> Result<Vec<Result<Document, CrawlerError>>, CrawlerError> {
        let mut documents = Vec::new();
        for res in self.document_indexes_after(&gallery, last_document_id, start_page).await? {
            match res {
                Ok(index) => {
                    let id = index.id;
                    if index.comment_count > 0 {
                        match self.comments(&gallery, id).await {
                            Ok(comms) => documents.push(Ok(Document { index: index, comments: comms })),
                            Err(err) => documents.push(Err(err.into())),
                        };
                    } else {
                        documents.push(Ok(Document { index: index, comments: Vec::new() }));
                    }
                },
                Err(err) => documents.push(Err(err.into())),
            };
        }
        Ok(documents)
    }
    async fn _comments(
        &mut self,
        gallery: &GalleryIndex,
        doc_id: usize,
        page: usize,
        last_root_comment_id: Option<usize>,
    ) -> Result<Vec<Comment>, CrawlerError> {
        let path = format!("{}/board/comment", self.host);
        if self.e_s_n_o.is_none() {
            self.document_indexes(&gallery, 1).await?;
        }
        let form = CommentsPostForm {
            id: &gallery.id,
            no: doc_id,
            cmt_id: &gallery.id,
            cmt_no: doc_id,
            e_s_n_o: self.e_s_n_o.as_deref(),
            comment_page: page,
            sort: "D",
            prevCnt: 0,
            _GALLTYPE_: "G",
        };
        let bytes = backoff::tokio::retry(backoff::ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .post(path.as_str())
                .header("Referer", format!("https://gall.dcinside.com/board/view/?id={}&no={}&_rk=tDl&page=1", gallery.id, doc_id))
                .header("X-Requested-With", "XMLHttpRequest")
                .send_form(&form).await.map_err(CrawlerError::from)?
                .body().await.map_err(CrawlerError::from)?)
        }).await?;
        let text = std::str::from_utf8(&bytes)?;
        Ok(parse_comments(text, &gallery.id, doc_id, last_root_comment_id)?)
    }
    pub async fn document_indexes(
        &mut self,
        gallery: &GalleryIndex,
        page: usize,
    ) -> Result<Vec<Result<DocumentIndex, DocumentParseError>>, CrawlerError> {
        let path = match gallery.kind {
            GalleryKind::Major => format!(
                "{}/board/lists?id={}&list_num=100&page={}",
                self.host, gallery.id, page
            ),
            GalleryKind::Minor => format!(
                "{}/mgallery/board/lists?id={}&list_num=100&page={}",
                self.host, gallery.id, page
            ),
            _ => panic!("What's this?"),
        };
        let bytes = backoff::tokio::retry(backoff::ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .get(path.as_str())
                .header("Referer", "https://gall.dcinside.com/")
                .send().await.map_err(CrawlerError::from)?
                .body().await.map_err(CrawlerError::from)?)
        }).await?;
        let text = std::str::from_utf8(&bytes)?;
        self.e_s_n_o = Some(HTMLDocument::from(text)
            .select(Attr("id", "e_s_n_o"))
            .next()
            .ok_or(DocumentParseError::Select { path: ".e_s_n_o" })?
            .attr("value")
            .ok_or(DocumentParseError::Select { path: ".e_s_n_o@value" })?.to_string());
        /*parse_document_indexes(text)
        .map_err(|e| CrawlerError::DocumentParseError{
            source: e,
            gallery: gallery.id.clone(),
            page: page,
        })?.into_iter().collect::<Result<Vec<_>, DocumentParseError>>()
        .map_err(|e| CrawlerError::DocumentParseError{
            source: e,
            gallery: gallery.id.clone(),
            page: page,
        })*/
        Ok(parse_document_indexes(text, &gallery.id)?)
        /*Ok(parse_document_indexes(text, &gallery.id)
            .map_err(|e| CrawlerError::DocumentParseError {
                source: e,
                gallery: gallery.id.clone(),
                page: page,
            })?
            .into_iter()
            .filter_map(|res| match res {
                Ok(res) => Some(res),
                Err(err) => {
                    warn!("{}", err);
                    None
                }
            })
            .collect())*/
    }
}
impl Default for Crawler {
    fn default() -> Self {
        Crawler::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[actix_rt::test]
    async fn realtime_hot_galleries() {
        let crawler = Crawler::new();
        let res = crawler.realtime_hot_galleries().await.unwrap();
        assert!(!res.is_empty());
        assert!(!res[0].id.is_empty());
        assert!(!res[0].name.is_empty());
        assert_eq!(res[0].kind, GalleryKind::Major);
    }
    #[actix_rt::test]
    async fn document_indexes() {
        let mut crawler = Crawler::new();
        let gallery = GalleryIndex {
            id: String::from("programming"),
            name: String::from("프로그래밍"),
            kind: GalleryKind::Major,
            rank: None,
        };
        let res = crawler.document_indexes(&gallery, 1).await.unwrap();
        assert!(!res.is_empty());
        assert!(res.len() >= 90);
        assert!(res.iter().any(|d| match d { Ok(d) => d.comment_count > 0, Err(_) => false}));
        assert!(res.iter().any(|d| match d { Ok(d) => if DocumentKind::Picture == d.kind {
            true
        } else {
            false
        }, Err(_) => false }));
    }
    #[actix_rt::test]
    async fn comments() {
        let mut crawler = Crawler::new();
        let gallery = GalleryIndex {
            id: String::from("programming"),
            name: String::from("프로그래밍"),
            kind: GalleryKind::Major,
            rank: None,
        };
        let res = crawler.comments(&gallery, 1586201).await.unwrap();
        assert!(!res.is_empty());
        assert!(res.len() >= 2);
        assert!(!res.iter().any(|c| match &c.author {
            User::Static { id, ..  } => id.is_empty(),
            User::Dynamic { ip, .. } => ip.is_empty(),
            _ => false,
        }));
        /*assert!(res.iter().any(|c| d.comment_count > 0));
        assert!(res.iter().any(|c| if DocumentKind::Picture == d.kind {
            true
        } else {
            false
        }));*/
    }
}
