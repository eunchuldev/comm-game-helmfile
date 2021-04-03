use crate::error::*;
use crate::parse::GalleryIndex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
