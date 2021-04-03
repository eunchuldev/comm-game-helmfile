//use crate::model::{Document};
//use dcinside_document_consumer::*;
//use std::io;
pub use dcinside_model::*;
//use chrono::{DateTime, Datelike, Duration, NaiveDateTime, TimeZone, Utc};
use nats::jetstream::{Consumer, StreamConfig};

use postgres::{Client, NoTls};

fn upsert_gallery(client: &mut Client, gallery: &Gallery) -> anyhow::Result<()> {
    client.execute(
        r#"
        INSERT INTO dcinside_gallery (id, name, kind) 
        VALUES ($1, $2, $3)
        ON CONFLICT DO NOTHING"#,
        &[&gallery.id, &gallery.name, &gallery.kind.name()],
    )?;
    Ok(())
}

fn upsert_document(client: &mut Client, doc: &Document) -> anyhow::Result<()> {
    client.execute(
        r#"
        INSERT INTO dcinside_documents 
            (gallery_id, id, title, subject, 
            author_nickname, author_ip, author_id, 
            comment_count, like_count, view_count, 
            kind, is_recommend, created_at) 
        VALUES 
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        ON CONFLICT DO NOTHING"#,
        &[
            &doc.gallery_id,
            &(doc.id as u32),
            &doc.title,
            &doc.subject,
            &doc.author.nickname,
            &doc.author.ip,
            &doc.author.id,
            &doc.comment_count,
            &doc.like_count,
            &doc.view_count,
            &doc.kind.name(),
            &doc.is_recommend,
            &doc.created_at,
        ],
    )?;
    Ok(())
}

/*fn upsert_comment(client: &mut Client, doc: &Comment) -> anyhow::Result<()> {
    client.execute(r#"
        INSERT INTO dcinside_documents
            (gallery_id, id, title, subject,
            author_nickname, author_ip, author_id,
            comment_count, like_count, view_count,
            kind, is_recommend, created_at)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        ON CONFLICT DO NOTHING"#,
        &[&doc.gallery_id, &(doc.id as u32), &doc.title, &doc.subject,
        &doc.author.nickname, &doc.author.ip, &doc.author.id,
        &doc.comment_count, &doc.like_count, &doc.view_count,
        &doc.kind.name(), &doc.is_recommend, &doc.created_at])?;
    Ok(())
}*/

pub fn subscribe(url: &str, subject: &str, consumer: &str) -> anyhow::Result<Consumer> {
    let nc = nats::connect(&url)?;
    let _ = nc.create_stream(StreamConfig {
        name: subject.to_owned(),
        num_replicas: 0,
        max_age: 7 * 24 * 60 * 60 * 1000,
        ..Default::default()
    })?;
    Ok(Consumer::create_or_open(nc, subject, consumer)?)
}

fn main() -> anyhow::Result<()> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let nats_url = std::env::var("NATS_URL").expect("NATS_URL");
    let nats_subject =
        std::env::var("NATS_SUBJECT").unwrap_or_else(|_| "crawled.dcinside.documents".to_string());

    let mut db_conn = Client::connect(&db_url, NoTls)?;

    let mut consumer = subscribe(&nats_url, &nats_subject, "dcgle_document_writer")?;

    loop {
        let msg = consumer.pull()?;
        let doc: Document = bincode::deserialize(&msg.data)?;
        upsert_gallery(&mut db_conn, &doc.gallery)?;
        upsert_document(&mut db_conn, &doc)?;
        msg.ack()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
