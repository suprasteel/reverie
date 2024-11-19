use std::str::FromStr;

use anyhow::Context;
use tracing::warn;

use crate::{
    core::{
        model::{Log, Project, ProjectId, User, UserId, Username},
        repo::{
            AuthorRepository, CreateAuthorError, CreateAuthorRequest, CreateLogError,
            CreateLogRequest, CreateProjectRequest, LogRepository, ProjectRepository,
        },
    },
    Page, Paged,
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
    async fn create_author(&self, request: CreateAuthorRequest) -> Result<User, CreateAuthorError> {
        let new_author = User::create(request.username);
        let _ = sqlx::query("INSERT INTO author (id,name) VALUES ($1,$2)")
            .bind(new_author.id())
            .bind(&new_author.name)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                warn!("{e}");
                CreateAuthorError
            })?;
        Ok(new_author)
    }

    async fn get_author_by_name(&self, username: &Username) -> Option<User> {
        sqlx::query_as("SELECT (id,name) FROM author WHERE name = ?")
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| warn!("{e}"))
            .ok()
    }

    async fn get_author_by_id(&self, id: UserId) -> Option<User> {
        sqlx::query_as("SELECT (id,name) FROM author WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| warn!("{e}"))
            .ok()
    }
}

impl LogRepository for Sqlite {
    async fn create_log(&self, request: CreateLogRequest) -> Result<Log, CreateLogError> {
        // let tx = self.pool.begin().await.map_err(|e| {
        //     warn!("{e}");
        //     CreateLogError
        // })?;
        let CreateLogRequest {
            author,
            project,
            text,
        } = request;
        let log = Log::new(text, author);
        let _ = sqlx::query(
            "INSERT INTO log (id,project,author,created,version,revision,text) VALUES ($1,$2,$3,$4,$5,$6,$7)",
        )
        .bind(log.id())
        .bind(project)
        .bind(author)
        .bind(log.meta.created.as_i64())
        .bind(log.meta.version)
        .bind(log.meta.revision)
        .bind(&log.text)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            warn!("{e}");
            CreateLogError
        })?;
        // tx.commit().await.map_err(|e| {
        //     warn!("{e}");
        //     CreateLogError
        // })?;
        Ok(log)
    }

    /// Fetches all rows. Not streaming
    async fn list_project_logs(&self, project: ProjectId, page: Page) -> Result<Paged<Log>, ()> {
        let logs: Vec<Log> = sqlx::query_as("SELECT (id,author,created,version,revision,text) FROM log WHERE project = ? LIMIT ? OFFSET ?")
            .bind(project)
            .bind(page.page_size() as i32)
            .bind(page.offset() as i32)
            .bind(project)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| warn!("{e}"))?;
        use crate::Paginable;
        Ok(logs.to_paged(page))
    }
}

impl ProjectRepository for Sqlite {
    async fn create_project(&self, request: CreateProjectRequest) -> Result<Project, ()> {
        let CreateProjectRequest {
            author,
            project_name,
        } = request;
        let project = Project::new(project_name, author);
        let _ = sqlx::query(
            "INSERT INTO project (id,author,created,version,revision,name) VALUES ($1,$2,$3,$4,$5,$6,$7)",
        )
        .bind(project.id())
        .bind(project.meta.author)
        .bind(project.meta.created.as_i64())
        .bind(project.meta.version)
        .bind(project.meta.revision)
        .bind(&project.name)
        .execute(&self.pool)
        .await
        .map_err(|e| warn!("{e}"))?;
        Ok(project)
    }
    async fn get_project_by_name(&self, name: &str) -> Option<Project> {
        sqlx::query_as(
            "SELECT (id,author,created,version,revision,name) FROM project WHERE name = ?",
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| warn!("{e}"))
        .ok()
    }

    async fn get_project_by_id(&self, id: ProjectId) -> Option<Project> {
        sqlx::query_as("SELECT (id,author,created,version,revision,name) FROM author WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| warn!("{e}"))
            .ok()
    }
}
