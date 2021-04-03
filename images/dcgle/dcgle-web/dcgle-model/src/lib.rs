mod error;

use serde::{Serialize, Deserialize};
use async_graphql::{EmptyMutation, EmptySubscription, Schema as GraphqlSchema, SimpleObject, Context, Error as GraphqlError, Object};
//use juniper::{graphql_object, EmptyMutation, EmptySubscription, GraphQLObject, RootNode};

pub use error::Error;

pub type Pool = sqlx::postgres::PgPool;
pub type PoolOptions = sqlx::postgres::PgPoolOptions;

type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize, PartialEq, SimpleObject)]
pub struct Gallery {
    pub id: String,
    pub name: String,
    pub kind: String,
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize, PartialEq, SimpleObject)]
pub struct Document {
    pub gallery_id: String,
    pub gallery_name: String,
    pub id: i32,
    pub title: String,
    pub subject: Option<String>,
    pub author_nickname: String,
    pub author_ip: Option<String>,
    pub author_id: Option<String>,
    pub comment_count: i32,
    pub like_count: i32,
    pub view_count: i32,
    pub kind: String,
    pub is_recommend: bool,
    pub created_at: DateTime,
}

#[derive(Clone)]
pub struct State {
    pool: Pool,
}

impl State {
    pub async fn migrate(db_url: &str) -> Result<(), Error> {
        let pool = PoolOptions::new().connect(db_url).await?;
        let mut migrator = sqlx::migrate!();
        migrator.migrations.to_mut().retain(|migration| !migration.description.ends_with(".down"));
        migrator.run(&pool).await?;
        Ok(())
    }
    pub async fn connect(db_url: &str) -> Result<Self, Error> {
        let pool = PoolOptions::new().connect(db_url).await?;
        Ok(State{
            pool,
            })
    }
    pub async fn health(&self) -> Result<(), Error> {
        let _ = sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }

    pub async fn get_gallery(&self) -> Result<Vec<Gallery>, Error> {
        let res = sqlx::query_as::<_, Gallery>("SELECT * FROM dcinside_gallery LIMIT 30")
            .fetch_all(&self.pool)
            .await?;
        Ok(res)
    }
    pub async fn get_gallery_by_id(&self, id: &str) -> Result<Vec<Gallery>, Error> {
        let res = sqlx::query_as::<_, Gallery>("SELECT * FROM dcinside_gallery WHERE id = ?")
            .bind(id)
            .fetch_all(&self.pool)
            .await?;
        Ok(res)
    }
    pub async fn get_gallery_by_name_part(&self, name_part: &str) -> Result<Vec<Gallery>, Error> {
        let res = sqlx::query_as::<_, Gallery>("SELECT * FROM dcinside_gallery WHERE name ILIKE '%' || ? || '%' LIMIT 30")
            .bind(name_part)
            .fetch_all(&self.pool)
            .await?;
        Ok(res)
    }



    pub async fn get_docs_by_title(&self, title: &str, last_created_at: Option<DateTime>) -> Result<Vec<Document>, Error> {
        Err(Error::NotImplemented("get_docs_by_title", "title get(fts) is not implemented yet"))
        /*let res = sqlx::query_as::<_, Document>("SELECT * FROM users WHERE _title @@ ? ORDER BY created_at <=| ? LIMIT 100")
            .bind(title)
            .bind(last_created_at.unwrap_or(chrono::Utc::now()))
            .fetch_all(&self.pool)
            .await?;
        Ok(res)*/
    }
    pub async fn get_docs_by_gallery_and_title(&self, gallery: &str, title: &str, last_created_at: Option<DateTime>) -> Result<Vec<Document>, Error> {
        Err(Error::NotImplemented("get_docs_by_gallery_and_title", "gallery with title get(fts) is not implemented yet"))
        /*let res = sqlx::query_as::<_, Document>("SELECT * FROM users WHERE _title @@ ? ORDER BY _gallery_id_and_created_at <=| cat_and_time(?, ?) LIMIT 100")
            .bind(title)
            .bind(gallery)
            .bind(last_created_at.unwrap_or(chrono::Utc::now()))
            .fetch_all(&self.pool)
            .await?;
        Ok(res)*/
    }
    pub async fn get_docs_by_nickname(&self, nickname: &str, last_created_at: Option<DateTime>) -> Result<Vec<Document>, Error> {
        let res = sqlx::query_as::<_, Document>(r#"
            SELECT 
                d.*,  
                g.name AS gallery_name
            FROM dcinside_document d 
            INNER JOIN gallery g ON g.id = d.gallery_id 
            WHERE author_nickname = ? AND created_at <= ? 
            ORDER BY last_created_at DESC 
            LIMIT 30"#)
            .bind(nickname)
            .bind(last_created_at.unwrap_or(chrono::Utc::now()))
            .fetch_all(&self.pool)
            .await?;
        Ok(res)
    }
    pub async fn get_docs_by_ip(&self, ip: &str, last_created_at: Option<DateTime>) -> Result<Vec<Document>, Error> {
        let res = sqlx::query_as::<_, Document>(r#"
            SELECT  
                d.*,  
                g.name AS gallery_name
            FROM dcinside_document d 
            INNER JOIN gallery g ON g.id = d.gallery_id 
            WHERE author_ip = ? AND created_at <= ? 
            ORDER BY last_created_at DESC 
            LIMIT 30"#)
            .bind(ip)
            .bind(last_created_at.unwrap_or(chrono::Utc::now()))
            .fetch_all(&self.pool)
            .await?;
        Ok(res)
    }
    pub async fn get_docs_by_id(&self, id: &str, last_created_at: Option<DateTime>) -> Result<Vec<Document>, Error> {
        let res = sqlx::query_as::<_, Document>(r#"
            SELECT 
                d.*,  
                g.name AS gallery_name
            FROM dcinside_document d 
            INNER JOIN gallery g ON g.id = d.gallery_id 
            WHERE author_id = ? AND created_at <= ? 
            ORDER BY last_created_at DESC 
            LIMIT 30"#)
            .bind(id)
            .bind(last_created_at.unwrap_or(chrono::Utc::now()))
            .fetch_all(&self.pool)
            .await?;
        Ok(res)
    }


    pub async fn get_docs_by_gallery_and_nickname(&self, gallery: &str, nickname: &str, last_created_at: Option<DateTime>) -> Result<Vec<Document>, Error> {
        let res = sqlx::query_as::<_, Document>(r#"
            SELECT 
                d.*,  
                g.name AS gallery_name
            FROM dcinside_document d 
            INNER JOIN gallery g ON g.id = d.gallery_id 
            WHERE author_nickname = ? AND created_at <= ? AND gallery_id = ?
            ORDER BY last_created_at DESC 
            LIMIT 30"#)
            .bind(nickname)
            .bind(last_created_at.unwrap_or(chrono::Utc::now()))
            .bind(gallery)
            .fetch_all(&self.pool)
            .await?;
        Ok(res)
    }
    pub async fn get_docs_by_gallery_and_ip(&self, gallery: &str, ip: &str, last_created_at: Option<DateTime>) -> Result<Vec<Document>, Error> {
        let res = sqlx::query_as::<_, Document>(r#"
            SELECT 
                d.*,  
                g.name AS gallery_name
            FROM dcinside_document d 
            INNER JOIN gallery g ON g.id = d.gallery_id 
            WHERE author_ip = ? AND created_at <= ? AND gallery_id = ?
            ORDER BY last_created_at DESC 
            LIMIT 30"#)
            .bind(ip)
            .bind(last_created_at.unwrap_or(chrono::Utc::now()))
            .bind(gallery)
            .fetch_all(&self.pool)
            .await?;
        Ok(res)
    }
    pub async fn get_docs_by_gallery_and_id(&self, gallery: &str, id: &str, last_created_at: Option<DateTime>) -> Result<Vec<Document>, Error> {
        let res = sqlx::query_as::<_, Document>(r#"
            SELECT 
                d.*,  
                g.name AS gallery_name
            FROM dcinside_document d 
            INNER JOIN gallery g ON g.id = d.gallery_id 
            WHERE author_id = ? AND created_at <= ? AND gallery_id = ?
            ORDER BY last_created_at DESC 
            LIMIT 30"#)
            .bind(id)
            .bind(last_created_at.unwrap_or(chrono::Utc::now()))
            .bind(gallery)
            .fetch_all(&self.pool)
            .await?;
        Ok(res)
    }

}


pub struct Query;

#[Object]
impl Query {
    async fn api_version(&self) -> String {
        "0.1".to_string()
    }
    async fn galleries(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search term of galelry")] name_part: Option<String>,
        id: Option<String>,
        ) -> Result<Vec<Gallery>, GraphqlError> {
        let state: &State = ctx.data::<State>()?;
        Ok(match (id, name_part) {
            (Some(id), None) => state.get_gallery_by_id(&id).await?,
            (None, Some(name_part)) => state.get_gallery_by_name_part(&name_part).await?,
            //(None, None) => state.get_gallery().await?,
            _ => Err(Error::BadRequest("galleries", "valid form: { ONE_OF(id, name_part) }"))?,
        })
    }
    async fn documents(
        &self,
        ctx: &Context<'_>,
        gallery_id: Option<String>,
        title: Option<String>,
        author_nickname: Option<String>,
        author_ip: Option<String>,
        author_id: Option<String>,
        last_created_at: Option<DateTime>,
        ) -> Result<Vec<Document>, GraphqlError> {
        let state: &State = ctx.data::<State>()?;
        Ok(match (gallery_id, title, author_nickname, author_ip, author_id) {
            (Some(gallery_id), Some(title), _, _, _) => state.get_docs_by_gallery_and_title(&gallery_id, &title, last_created_at).await?,
            (None, Some(title), _, _, _) => state.get_docs_by_title(&title, last_created_at).await?,
            (None, _, Some(author_nickname), None, None) => state.get_docs_by_nickname(&author_nickname, last_created_at).await?,
            (None, _, None, Some(author_ip), None) => state.get_docs_by_ip(&author_ip, last_created_at).await?,
            (None, _, None, None, Some(author_id)) => state.get_docs_by_id(&author_id, last_created_at).await?,
            (Some(gallery_id), _, Some(author_nickname), None, None) => state.get_docs_by_gallery_and_nickname(&gallery_id, &author_nickname, last_created_at).await?,
            (Some(gallery_id), _, None, Some(author_ip), None) => state.get_docs_by_gallery_and_ip(&gallery_id, &author_ip, last_created_at).await?,
            (Some(gallery_id), _, None, None, Some(author_id)) => state.get_docs_by_gallery_and_id(&gallery_id, &author_id, last_created_at).await?,
            _ => Err(Error::BadRequest("documents", "valid form: { [gallery_id,] [last_created_at,] ONE_OF(title, author_nickname, authir_ip, author_id)"))?,
        })
    }
}

//pub type Schema = RootNode<'static, Query, EmptyMutation<State>, EmptySubscription<State>>;
pub type Schema = GraphqlSchema<Query, EmptyMutation, EmptySubscription>;


pub fn schema(state: State) -> GraphqlSchema<Query, EmptyMutation, EmptySubscription> {
    GraphqlSchema::build(
        Query,
        EmptyMutation,
        EmptySubscription,
    ).data(state).finish()
}

#[cfg(test)]
pub mod tests {
    use super::*;
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


    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn healthcheck() {
        let (state, galleries, docs) = setup().await;
        state.health().await.unwrap();
        let res = state.get_gallery().await.unwrap();
        assert_eq!(res, galleries);
    }

    #[tokio::test]
    #[serial]
    async fn get_galleries() {
        let (state, galleries, docs) = setup().await;
        let res = state.get_gallery().await.unwrap();
        assert_eq!(res, galleries);
    }
}
