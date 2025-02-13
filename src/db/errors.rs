use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

impl From<sqlx::migrate::MigrateError> for DatabaseError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        Self::DatabaseError(err.to_string())
    }
}
