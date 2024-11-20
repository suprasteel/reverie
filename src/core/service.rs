// pub async fn create_author<AR: AuthorRepository>(
//     State(state): State<AppState<AR>>,
//     Json(body): Json<CreateAuthorHttpRequestBody>,
// ) -> Result<ApiSuccess<CreateAuthorResponseData>, ApiError> {
// 	let domain_req = body.try_into_domain()?;
//     state
//         .author_repo
//         .create_author(&domain_req)
//         .await
//         .map_err(ApiError::from)
//         .map(|ref author| ApiSuccess::new(StatusCode::CREATED, author.into()))
// }

use std::future::Future;

use crate::{Page, Paged};

use super::{
    model::{
        InvalidProjectName, InvalidUsername, Log, Project, ProjectId, ProjectName, User, UserId,
        Username,
    },
    repo::{
        AuthorRepository, CreateAuthorError, CreateAuthorRequest, CreateLogError, CreateLogRequest,
        CreateProjectError, CreateProjectRequest, LogRepository, ProjectRepository, RepoQueryError,
    },
};

#[derive(Debug, Clone)]
pub struct LogService<R>
where
    R: AuthorRepository + ProjectRepository + LogRepository,
{
    repo: R,
}

impl<R> LogService<R>
where
    R: AuthorRepository + ProjectRepository + LogRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

// impl<R, M, N> AuthorService for Service<R, M, N>
// where
//     R: AuthorRepository,
//     // S: MergeService,
// {
//     async fn create_author(&self, req: &CreateAuthorRequest) -> Result<Author, CreateAuthorError> {
//         let result = self.repo.create_author(req).await;

//         result
//     }
// }

#[derive(Debug, thiserror::Error)]
pub enum LogServiceError {
    #[error("Project not found")]
    ProjectNotFound,
    #[error("Project exists")]
    ProjectExists,
    #[error("User not found")]
    UserNotFound,
    #[error("User exists")]
    UserExists,
    #[error("{0} has no read access on {1}")]
    NoReadAccess(Username, ProjectName),
    #[error("{0} has no write access on {1}")]
    NoWriteAccess(Username, ProjectName),
    #[error("{0}")]
    InvalidUsername(InvalidUsername),
    #[error("{0}")]
    InvalidProjectName(InvalidProjectName),
    #[error("error: {0}")]
    TechnicalError(Box<dyn std::error::Error>),
}
impl From<InvalidUsername> for LogServiceError {
    fn from(value: InvalidUsername) -> Self {
        Self::InvalidUsername(value)
    }
}
impl From<CreateAuthorError> for LogServiceError {
    fn from(value: CreateAuthorError) -> Self {
        Self::TechnicalError(Box::new(value))
    }
}
impl From<CreateProjectError> for LogServiceError {
    fn from(value: CreateProjectError) -> Self {
        Self::TechnicalError(Box::new(value))
    }
}
impl From<CreateLogError> for LogServiceError {
    fn from(value: CreateLogError) -> Self {
        Self::TechnicalError(Box::new(value))
    }
}
impl From<RepoQueryError> for LogServiceError {
    fn from(value: RepoQueryError) -> Self {
        Self::TechnicalError(Box::new(value))
    }
}

impl<R> LocalLogStoreService for LogService<R>
where
    R: AuthorRepository + ProjectRepository + LogRepository,
{
    async fn new_user(&self, username: Username) -> Result<User, LogServiceError> {
        let request = CreateAuthorRequest { username };
        Ok(self.repo.create_author(request).await?)
    }
    // async fn project_info(&self, name: &str) -> Result<ProjectDetails, ()> {
    //     self.repo.get_project_by_name(name).await.ok_or(())
    // }

    async fn new_project(
        &self,
        name: ProjectName,
        owner: UserId,
    ) -> Result<Project, LogServiceError> {
        let request = CreateProjectRequest {
            owner,
            project_name: name,
        };
        Ok(self.repo.create_project(request).await?)
    }

    async fn add_log(
        &self,
        by: UserId,
        on: ProjectId,
        text: String,
    ) -> Result<Log, LogServiceError> {
        let request = CreateLogRequest {
            author: by,
            project: on,
            text,
        };
        Ok(self.repo.create_log(request).await?)
    }
    async fn logs(&self, project: ProjectId, page: Page) -> Result<Paged<Log>, LogServiceError> {
        Ok(self.repo.list_project_logs(project, page).await?)
    }
    async fn projects_of(&self, user: UserId) -> Vec<Project> {
        self.repo.list_projects_for_user(user).await
    }
    #[cfg(feature = "admin")]
    async fn list_users(&self, page: Page) -> Paged<User> {
        self.repo.list_users(page).await
    }
}

pub trait LocalLogStoreService {
    fn new_user(
        &self,
        username: Username,
    ) -> impl Future<Output = Result<User, LogServiceError>> + Send;

    /// Return informations about the project + stats
    // fn project_info(&self, name: &str) -> impl Future<Output = Result<Project, ()>> + Send;

    /// create a new project by name
    fn new_project(
        &self,
        name: ProjectName,
        owner: UserId,
    ) -> impl Future<Output = Result<Project, LogServiceError>>;
    /// add a log to the project
    fn add_log(
        &self,
        by_user: UserId,
        on_project: ProjectId,
        text: String,
    ) -> impl Future<Output = Result<Log, LogServiceError>>;
    fn logs(
        &self,
        project: ProjectId,
        page: Page,
    ) -> impl Future<Output = Result<Paged<Log>, LogServiceError>> + Send;
    fn projects_of(&self, user: UserId) -> impl Future<Output = Vec<Project>> + Send;
    fn list_users(&self, page: Page) -> impl Future<Output = Paged<User>> + Send;
}
