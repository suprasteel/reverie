use std::str::FromStr;

use anyhow::Context;
use serde::de::Error;
use tracing::warn;

use crate::core::{
    model::{Author, UserId, Username},
    repo::{AuthorRepository, CreateAuthorError, CreateAuthorRequest},
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
        let _ = sqlx::query("INSERT INTO AUTHOR (id, name) VALUES ($1,$2)")
            .bind(new_author.id())
            .bind(&new_author.name().to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| {
                warn!("{e}");
                CreateAuthorError
            })?;
        Ok(new_author)
    }

    fn get_author_by_name(
        &self,
        username: Username,
    ) -> impl std::future::Future<Output = Option<Author>> + Send {
        // let query = sqlx::query("SELECT * FROM AUTHOR WHERE ")
        //     .execute(&pool)
        //     .await
        //     .unwrap();
        std::future::ready(None)
    }

    fn get_author_by_id(
        &self,
        id: UserId,
    ) -> impl std::future::Future<Output = Option<Author>> + Send {
        std::future::ready(None)
    }
}
