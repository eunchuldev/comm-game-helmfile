use model::State;
use std::net::{IpAddr, SocketAddr, AddrParseError};
use std::str::FromStr;
use thiserror::Error;
use warp::reject::Reject;
use model::Error as ModelError;

#[derive(Error, Debug)]
pub enum WebError {
    #[error("io err: {0}")]
    IoError(#[from] std::io::Error),
    #[error("addr parse err: {0}")]
    AddrParseError(#[from] AddrParseError),
    #[error("model err: {0}")]
    ModelError(#[from] ModelError),
}

impl Reject for WebError {} 


pub async fn serve(db_url: &str, host: &str, port: u16) -> Result<(), WebError> {
    let state = State::connect(db_url).await?;
    warp::serve(filters::routes(state))
        .run(SocketAddr::new(IpAddr::from_str(host)?, port))
        .await;
    Ok(())
}

pub mod filters {
    use super::{handlers, WebError};
    use warp::Filter;
    use model::{State, Schema, schema};
    use http::StatusCode;
    use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
    use async_graphql_warp::{BadRequest, Response};

    pub fn routes(state: State) 
        -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        graphql_playground()
            .or(graphql_post(schema(state)))
            .or(front())
            /*.recover(|err: warp::Rejection| async move {
                if let Some(BadRequest(err)) = err.find() {
                    return Ok(warp::reply::with_status(
                            err.to_string(),
                            StatusCode::BAD_REQUEST,
                    ));
                }

                Ok(warp::reply::with_status(
                        "INTERNAL_SERVER_ERROR".to_string(),
                        StatusCode::INTERNAL_SERVER_ERROR,
                ))
            })*/
    }


    pub fn front() 
        -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
         warp::path("static")
            .and(warp::fs::dir("svelte-app/public"))
    }

    pub fn graphql_post(schema: Schema) 
        -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("graphql").and(async_graphql_warp::graphql(schema).and_then(
            |(schema, request): (
                Schema, async_graphql::Request,
            )| async move { Ok::<_, warp::Rejection>(Response::from(schema.execute(request).await)) },
        ))
    }

    pub fn graphql_playground()
        -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("graphiql").and(warp::get()).map(|| {
            warp::http::Response::builder()
                .header("content-type", "text/html")
                .body(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
        })
    }

}

pub mod handlers {
    use model::{State, Schema, schema};
}

#[cfg(test)]
mod tests {
    use super::filters::*;
    use super::*;
    use model::{State, Gallery, Document, PoolOptions};
    use warp::Filter;
    use serial_test::serial;
    pub async fn setup() -> (State, Vec<Gallery>, Vec<Document>) {
        let galleries = vec![
            Gallery{ 
                id: "gallery_id1".to_string(),
                name: "gallery_name1".to_string(),
                kind: "gallery_kind1".to_string(),
            },
            Gallery{ 
                id: "gallery_id2".to_string(),
                name: "gallery_name2".to_string(),
                kind: "gallery_kind2".to_string(),
            }
        ];
        let docs = vec![
            Document{
                gallery_id: "gallery_id1".to_string(),
                gallery_name: "gallery_name1".to_string(),
                id: 1,
                title: "title1".to_string(),
                subject: Some("subject1".to_string()),
                author_nickname: "nickname1".to_string(),
                author_ip: Some("ip1".to_string()),
                author_id: Some("id1".to_string()),
                comment_count: 2,
                like_count: 3,
                view_count: 4,
                kind: "kind1".to_string(),
                is_recommend: true,
                created_at: chrono::Utc::now(),
            },
            Document{
                gallery_id: "gallery_id2".to_string(),
                gallery_name: "gallery_name2".to_string(),
                id: 2,
                title: "title2".to_string(),
                subject: Some("subject2".to_string()),
                author_nickname: "nickname2".to_string(),
                author_ip: Some("ip2".to_string()),
                author_id: Some("id2".to_string()),
                comment_count: 3,
                like_count: 4,
                view_count: 5,
                kind: "kind2".to_string(),
                is_recommend: false,
                created_at: chrono::Utc::now(),
            }
        ];
        let url = std::env::var("DATABASE_URL").unwrap();
        let pool = PoolOptions::new().connect(&url).await.unwrap();
        sqlx::query!("DELETE FROM dcinside_gallery").execute(&pool).await.unwrap();
        for g in galleries.iter() {
            sqlx::query!(
                r#"
                INSERT INTO dcinside_gallery ( id, name, kind )
                VALUES ( $1, $2, $3 )"#,
                g.id, g.name, g.kind
            )
                .execute(&pool)
                .await.unwrap();
            }
        sqlx::query!("DELETE FROM dcinside_document").execute(&pool).await.unwrap();
        for d in docs.iter() {
            sqlx::query!(
                r#"
                INSERT INTO dcinside_document ( 
                gallery_id, id, title, subject, 
                author_nickname, author_ip, author_id, 
                comment_count, like_count, view_count, 
                kind, is_recommend, created_at)
                VALUES ( $1, $2, $3, $4,
                         $5, $6, $7,
                         $8, $9, $10, 
                         $11, $12, $13)"#,
                         d.gallery_id, d.id, d.title, d.subject,
                         d.author_nickname, d.author_ip, d.author_id,
                         d.comment_count, d.like_count, d.view_count,
                         d.kind, d.is_recommend, d.created_at
            )
                .execute(&pool)
                .await.unwrap();
            }

        let state = State::connect(&url).await.unwrap();
        (state, galleries, docs)
    }

    #[tokio::test]
    #[serial]
    async fn test_gallery_name_autocomplete() {
        let (state, galleries, docs) = setup().await;
        let filter = routes(state);

        let res = warp::test::request()
            .path("/graphql")
            .method("POST")
            .body(r#"{ "query": "{ galleries(namePart: id2) { id, name } }"}"#)
            .reply(&filter)
            .await;

        assert_eq!(
            format!("{:?}", res),
            "Response { status: 200, version: HTTP/1.1, headers: {\"content-type\": \"application/json\"}, body: b\"{\\\"data\\\":{\\\"galleries\\\":[{\\\"id\\\":\\\"gallery_id2\\\",\\\"name\\\":\\\"gallery_name2\\\"}]}}\" }".to_string()
        );
    }
}
