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

use tracing::warn;

use crate::{Page, Paged};

use super::{
    model::{Log, Project, ProjectId, User, Username},
    repo::{
        AuthorRepository, CreateAuthorRequest, CreateProjectRequest, LogRepository,
        ProjectRepository,
    },
};

#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: AuthorRepository + ProjectRepository + LogRepository,
{
    repo: R,
    local_user: Option<User>,
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

impl<R> LocalService for Service<R>
where
    R: AuthorRepository + ProjectRepository + LogRepository,
{
    async fn new_user(&self, request: CreateAuthorRequest) -> Result<User, ()> {
        self.repo.create_author(request).await.map_err(|_| ())
    }

    async fn set_local_user(&self, name: Username) -> Result<(), ()> {
        let user = self.repo.get_author_by_name(&name).await;
        let _ = match user {
            Some(user) => user,
            None => self
                .repo
                .create_author(CreateAuthorRequest { username: name })
                .await
                .map_err(|_| ())?,
        };
        Ok(())
    }

    async fn project_info(&self, name: &str) -> Result<Project, ()> {
        self.repo.get_project_by_name(name).await.ok_or(())
    }

    async fn create_project(&self, request: CreateProjectRequest) -> Result<Project, ()> {
        self.repo.create_project(request).await.map_err(|_| ())
    }

    async fn add_log(&self, project: ProjectId, text: String) -> Result<Log, ()> {
        if self.local_user.is_none() {
            warn!("no local user");
            return Err(());
        }

        let request = super::repo::CreateLogRequest {
            author: self.local_user.as_ref().map(|user| user.id()).unwrap(),
            project,
            text,
        };

        self.repo.create_log(request).await.map_err(|_| ())
    }

    async fn get_logs(&self, project: ProjectId, page: Page) -> Result<Paged<Log>, ()> {
        self.repo.list_project_logs(project, page).await
    }
}

pub trait LocalService {
    fn new_user(
        &self,
        request: CreateAuthorRequest,
    ) -> impl Future<Output = Result<User, ()>> + Send;
    /// Set the local author for enabling default usage of user
    fn set_local_user(&self, name: Username) -> impl Future<Output = Result<(), ()>> + Send;
    /// Return informations about the project + stats
    fn project_info(&self, name: &str) -> impl Future<Output = Result<Project, ()>>;
    /// create a new project by name
    fn create_project(
        &self,
        request: CreateProjectRequest,
    ) -> impl Future<Output = Result<Project, ()>>;
    /// add a log to the project
    fn add_log(&self, project: ProjectId, content: String)
        -> impl Future<Output = Result<Log, ()>>;
    fn get_logs(
        &self,
        project: ProjectId,
        page: Page,
    ) -> impl Future<Output = Result<Paged<Log>, ()>>;
}
