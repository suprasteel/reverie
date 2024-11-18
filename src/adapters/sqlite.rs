use std::str::FromStr;

use anyhow::Context;
use tracing::warn;

use crate::core::{
    model::{Author, Log, UserId, Username},
    repo::{
        AuthorRepository, CreateAuthorError, CreateAuthorRequest, CreateLogRequest, LogRepository,
    },
};

#[derive(Debug, Clone)]
pub struct Sqlite {
    pool: sqlx::SqlitePool,
}

impl Sqlite {
    pub async fn new(path: &str) -> anyhow::Result<Sqlite> {
        let pool = sqlx::SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::from_str(path)
                .with_context(|| format!("invalid database path {}", path))?
                .pragma("foreign_keys", "ON"),
        )
        .await
        .with_context(|| format!("failed to open database at {}", path))?;
        Ok(Sqlite { pool })
    }
}

impl AuthorRepository for Sqlite {
    async fn create_author(
        &self,
        request: CreateAuthorRequest,
    ) -> Result<Author, CreateAuthorError> {
        let new_author = Author::create(request.username);
        let _ = sqlx::query("INSERT INTO authors (id,name) VALUES ($1,$2)")
            .bind(new_author.id())
            .bind(new_author.name())
            .execute(&self.pool)
            .await
            .map_err(|e| {
                warn!("{e}");
                CreateAuthorError
            })?;
        Ok(new_author)
    }

    async fn get_author_by_name(&self, username: Username) -> Option<Author> {
        sqlx::query_as("SELECT (id,name) FROM authors WHERE name = ?")
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| warn!("{e}"))
            .ok()
    }

    fn get_author_by_id(
        &self,
        id: UserId,
    ) -> impl std::future::Future<Output = Option<Author>> + Send {
        std::future::ready(None)
    }
}

impl LogRepository for Sqlite {
    fn create_log(
        &self,
        request: CreateLogRequest,
    ) -> impl std::future::Future<Output = Result<Log, ()>> + Send {
        std::future::ready(Ok(Log::new("1st log".to_string(), UserId::new())))
    }

    fn update_log(
        &self,
        request: crate::core::repo::UpdateLogRequest,
    ) -> impl std::future::Future<Output = Result<Log, ()>> + Send {
        std::future::ready(Err(()))
    }
}
