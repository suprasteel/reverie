use std::future::Future;

use derive_more::derive::Display;

use crate::{Page, Paged, ProjectName};

use super::model::{Log, Project, ProjectId, User, UserId, Username};

pub struct CreateAuthorRequest {
    pub username: Username,
}
#[derive(Debug, Display)]
#[display("Could not create author: {}", 0)]
pub struct CreateAuthorError(pub String);
impl std::error::Error for CreateAuthorError {}

#[derive(Debug, Display)]
#[display("Could not create project: {}", 0)]
pub struct CreateProjectError(pub String);
impl std::error::Error for CreateProjectError {}

#[derive(Debug, Display)]
#[display("Could not create log: {}", 0)]
pub struct CreateLogError(pub String);
impl std::error::Error for CreateLogError {}

#[derive(Debug, Display)]
#[display("Could not process query: {}", 0)]
pub struct RepoQueryError(pub String);
impl std::error::Error for RepoQueryError {}

pub struct CreateProjectRequest {
    pub owner: UserId,
    pub project_name: ProjectName,
}
pub struct CreateLogRequest {
    pub author: UserId,
    pub project: ProjectId,
    pub text: String,
}

pub trait AuthorRepository: Clone + Send + Sync + 'static {
    // todo define author repo err
    fn create_author(
        &self,
        request: CreateAuthorRequest,
    ) -> impl Future<Output = Result<User, CreateAuthorError>> + Send;
    fn get_user_by_name(&self, username: &Username) -> impl Future<Output = Option<User>> + Send;
    fn get_user_by_id(&self, id: UserId) -> impl Future<Output = Option<User>> + Send;
    #[cfg(feature = "admin")]
    fn list_users(&self, page: Page) -> impl Future<Output = Paged<User>> + Send;
}

pub trait ProjectRepository: Clone + Send + Sync + 'static {
    // todo define author repo err
    fn create_project(
        &self,
        request: CreateProjectRequest,
    ) -> impl Future<Output = Result<Project, CreateProjectError>> + Send;
    fn get_project_by_name(
        &self,
        name: &ProjectName,
    ) -> impl Future<Output = Option<Project>> + Send;
    fn get_project_by_id(&self, id: ProjectId) -> impl Future<Output = Option<Project>> + Send;
    fn list_user_projects(&self, id: UserId) -> impl Future<Output = Vec<Project>> + Send;
}

pub trait LogRepository: Clone + Send + Sync + 'static {
    fn create_log(
        &self,
        request: CreateLogRequest,
    ) -> impl Future<Output = Result<Log, CreateLogError>> + Send;
    // fn update_log(&self, request: UpdateLogRequest)
    //     -> impl Future<Output = Result<Log, ()>> + Send;
    fn list_project_logs(
        &self,
        project: ProjectId,
        page: Page,
    ) -> impl Future<Output = Result<Paged<Log>, RepoQueryError>> + Send;
}
