use std::future::Future;

use super::model::{Author, EntryId, Log, Project, ProjectId, UserId, Username};

pub struct CreateAuthorRequest {
    pub username: Username,
}

pub struct CreateAuthorError;
pub struct CreateLogError;

pub struct CreateProjectRequest {
    author: UserId,
    project_name: String,
}

pub struct CreateLogRequest {
    pub author: UserId,
    pub project: ProjectId,
    pub text: String,
}

pub struct UpdateLogRequest {
    log: EntryId,
    author: UserId,
    project: ProjectId,
    text: String,
}

pub trait AuthorRepository: Clone + Send + Sync + 'static {
    // todo define author repo err
    fn create_author(
        &self,
        request: CreateAuthorRequest,
    ) -> impl Future<Output = Result<Author, CreateAuthorError>> + Send;
    fn get_author_by_name(&self, username: Username)
        -> impl Future<Output = Option<Author>> + Send;
    fn get_author_by_id(&self, username: UserId) -> impl Future<Output = Option<Author>> + Send;
}

pub trait ProjectRepository: Clone + Send + Sync + 'static {
    // todo define author repo err
    fn create_project(
        &self,
        request: CreateProjectRequest,
    ) -> impl Future<Output = Result<Project, ()>> + Send;
}

pub trait LogRepository: Clone + Send + Sync + 'static {
    fn create_log(
        &self,
        request: CreateLogRequest,
    ) -> impl Future<Output = Result<Log, CreateLogError>> + Send;
    fn update_log(&self, request: UpdateLogRequest)
        -> impl Future<Output = Result<Log, ()>> + Send;
}
