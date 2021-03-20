mod error;

use serde::{Serialize, Deserialize};
use juniper::{graphql_object, EmptyMutation, EmptySubscription, GraphQLObject, RootNode};

pub use error::Error;

type Pool = sqlx::postgres::PgPool;
type PoolOptions = sqlx::postgres::PgPoolOptions;

type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize, PartialEq, GraphQLObject)]
pub struct Gallery {
    pub id: String,
    pub name: String,
    pub kind: String,
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize, PartialEq, GraphQLObject)]
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

impl juniper::Context for State {}


pub struct Query;

#[graphql_object(context = State)]
impl Query {
    fn apiVersion() -> String {
        "0.1".to_string()
    }
    async fn galleries(
        state: &State,
        name_part: Option<String>,
        id: Option<String>,
        ) -> Result<Vec<Gallery>, Error> {
        Ok(match (id, name_part) {
            (Some(id), None) => state.get_gallery_by_id(&id).await?,
            (None, Some(name_part)) => state.get_gallery_by_name_part(&name_part).await?,
            (None, None) => state.get_gallery().await?,
            _ => return Err(Error::BadRequest("galleries", "valid form: { [id,] [name_part,] }")),
        })
    }
    async fn documents(
        state: &State, 
        gallery_id: Option<String>,
        title: Option<String>,
        author_nickname: Option<String>,
        author_ip: Option<String>,
        author_id: Option<String>,
        last_created_at: Option<DateTime>,
        ) -> Result<Vec<Document>, Error> {
        Ok(match (gallery_id, title, author_nickname, author_ip, author_id) {
            (Some(gallery_id), Some(title), _, _, _) => state.get_docs_by_gallery_and_title(&gallery_id, &title, last_created_at).await?,
            (None, Some(title), _, _, _) => state.get_docs_by_title(&title, last_created_at).await?,
            (None, _, Some(author_nickname), None, None) => state.get_docs_by_nickname(&author_nickname, last_created_at).await?,
            (None, _, None, Some(author_ip), None) => state.get_docs_by_ip(&author_ip, last_created_at).await?,
            (None, _, None, None, Some(author_id)) => state.get_docs_by_id(&author_id, last_created_at).await?,
            (Some(gallery_id), _, Some(author_nickname), None, None) => state.get_docs_by_gallery_and_nickname(&gallery_id, &author_nickname, last_created_at).await?,
            (Some(gallery_id), _, None, Some(author_ip), None) => state.get_docs_by_gallery_and_ip(&gallery_id, &author_ip, last_created_at).await?,
            (Some(gallery_id), _, None, None, Some(author_id)) => state.get_docs_by_gallery_and_id(&gallery_id, &author_id, last_created_at).await?,
            _ => return Err(Error::BadRequest("documents", "valid form: { [gallery_id,] [last_created_at,] ONE_OF(title, author_nickname, authir_ip, author_id)")),
        })
    }
}

pub type Schema = RootNode<'static, Query, EmptyMutation<State>, EmptySubscription<State>>;

pub fn schema() -> Schema {
    Schema::new(
        Query,
        EmptyMutation::<State>::new(),
        EmptySubscription::<State>::new(),
    )
}
