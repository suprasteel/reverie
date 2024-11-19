use std::future::Future;

use crate::{Page, Paged};

use super::model::{EntryId, Log, Project, ProjectId, User, UserId, Username};

pub struct CreateAuthorRequest {
    pub username: Username,
}

pub struct CreateAuthorError;
pub struct CreateLogError;

pub struct CreateProjectRequest {
    pub author: UserId,
    pub project_name: String,
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
    ) -> impl Future<Output = Result<User, CreateAuthorError>> + Send;
    fn get_author_by_name(&self, username: &Username) -> impl Future<Output = Option<User>> + Send;
    fn get_author_by_id(&self, id: UserId) -> impl Future<Output = Option<User>> + Send;
}

pub trait ProjectRepository: Clone + Send + Sync + 'static {
    // todo define author repo err
    fn create_project(
        &self,
        request: CreateProjectRequest,
    ) -> impl Future<Output = Result<Project, ()>> + Send;
    fn get_project_by_name(&self, name: &str) -> impl Future<Output = Option<Project>> + Send;
    fn get_project_by_id(&self, id: ProjectId) -> impl Future<Output = Option<Project>> + Send;
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
    ) -> impl Future<Output = Result<Paged<Log>, ()>> + Send;
}
