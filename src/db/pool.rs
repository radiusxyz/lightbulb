use sqlx::{migrate::Migrator, Pool, Sqlite, SqlitePool};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct DbPool {
    pub pool: Pool<Sqlite>,
}

impl DbPool {
    pub async fn new(database_url: &str) -> Result<DbPool, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;

        // Run the migrations
        MIGRATOR.run(&pool).await?;

        Ok(DbPool { pool })
    }
}
