use std::str::FromStr;

use anyhow::Context;
use tracing::{instrument, warn};

use crate::{
    core::{
        model::{Log, Project, ProjectId, User, UserId, Username},
        repo::{
            AuthorRepository, CreateAuthorError, CreateAuthorRequest, CreateLogError,
            CreateLogRequest, CreateProjectError, CreateProjectRequest, LogRepository,
            ProjectRepository, RepoQueryError,
        },
    },
    Page, Paged, ProjectName,
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
                CreateAuthorError(e.to_string())
            })?;
        Ok(new_author)
    }

    async fn get_user_by_name(&self, username: &Username) -> Option<User> {
        sqlx::query_as("SELECT id,name FROM author WHERE name = ?")
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| warn!("{e}"))
            .ok()
    }

    async fn get_user_by_id(&self, id: UserId) -> Option<User> {
        sqlx::query_as("SELECT id,name FROM author WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| warn!("{e}"))
            .ok()
    }

    #[instrument]
    async fn list_users(&self, page: Page) -> Paged<User> {
        use crate::Paginable;
        // let users: Vec<([u8; 15], String)> = sqlx::query_as("SELECT (id,name) FROM author")
        sqlx::query_as("SELECT id,name FROM author")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| warn!("{e}"))
            .ok()
            .unwrap_or_default()
            // .unwrap_or_default();
            .to_paged(page)
        // users
        //     .into_iter()
        //     .map(|u| u.into())
        //     .collect::<Vec<User>>()
        //     .to_paged(page)
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
            CreateLogError(e.to_string())
        })?;
        // tx.commit().await.map_err(|e| {
        //     warn!("{e}");
        //     CreateLogError
        // })?;
        Ok(log)
    }

    /// Fetches all rows. Not streaming
    async fn list_project_logs(
        &self,
        project: ProjectId,
        page: Page,
    ) -> Result<Paged<Log>, RepoQueryError> {
        let logs: Vec<Log> = sqlx::query_as("SELECT (id,author,created,version,revision,text) FROM log WHERE project = ? LIMIT ? OFFSET ?")
            .bind(project)
            .bind(page.page_size() as i32)
            .bind(page.offset() as i32)
            .bind(project)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {warn!("{e}"); RepoQueryError(e.to_string())})?;
        use crate::Paginable;
        Ok(logs.to_paged(page))
    }
}

impl ProjectRepository for Sqlite {
    async fn create_project(
        &self,
        request: CreateProjectRequest,
    ) -> Result<Project, CreateProjectError> {
        let CreateProjectRequest {
            owner: author,
            project_name,
        } = request;
        let project = Project::new(project_name, author);
        let _ = sqlx::query(
            "INSERT INTO project (id,author,created,version,revision,name) VALUES ($1,$2,$3,$4,$5,$6)",
        )
        .bind(project.id())
        .bind(project.meta.author)
        .bind(project.meta.created.as_i64())
        .bind(project.meta.version)
        .bind(project.meta.revision)
        .bind(&project.name)
        .execute(&self.pool)
        .await
        .map_err(|e| {
                warn!("{e}"); CreateProjectError(format!("{e:?}"))})?;
        Ok(project)
    }
    async fn get_project_by_name(&self, name: &ProjectName) -> Option<Project> {
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
        sqlx::query_as("SELECT (id,author,created,version,revision,name) FROM project WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| warn!("{e}"))
            .ok()
    }
    async fn list_projects_for_user(&self, id: UserId) -> Vec<Project> {
        sqlx::query_as("SELECT (id,author,created,version,revision,name) FROM project WHERE id = ?")
            .bind(id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| warn!("{e}"))
            .ok()
            .unwrap_or_default()
    }
}
