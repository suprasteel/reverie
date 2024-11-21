use std::str::FromStr;

use clap::{ArgGroup, Args, Parser};
use derive_more::derive::Display;
use itertools::Itertools;
use reverie::{
    LocalLogStoreService, LogService, Page, ProjectId, ProjectLog, ProjectName, SqliteRepo, UserId,
    Username,
};
use tracing_subscriber::{filter, fmt, layer::SubscriberExt};
#[derive(Debug, Parser)]
pub struct CliArgs {
    #[clap(subcommand)]
    cmd: CmdArgs,
}
#[derive(Debug, clap::Subcommand)]
enum CmdArgs {
    #[clap(subcommand)]
    New(NewArgs),
    #[clap(subcommand)]
    List(ListArgs),
    #[clap(subcommand)]
    Id(IdArgs),
    #[clap(subcommand)]
    Search(SearchArgs),
}
#[derive(Debug, clap::Subcommand)]
pub enum NewArgs {
    Log(NewLogArgs),
    User(UsernameArg),
    Project(UserProjectArgs),
}
#[derive(Debug, clap::Subcommand)]
pub enum IdArgs {
    User(UsernameArg),
    Project(UserProjectArgs),
}
#[derive(Debug, clap::Subcommand)]
pub enum SearchArgs {
    Logs(SearchLogsArgs),
    Projects(SearchProjectsArgs),
}
#[derive(Debug, Args, Clone)]
pub struct SearchLogsArgs {
    #[clap(short, long)]
    user: Username,
    #[clap(trailing_var_arg = true, allow_hyphen_values = true)]
    desc: String,
    // before: Date,
    // after: Date,
}
#[derive(Debug, Args, Clone)]
pub struct SearchProjectsArgs {}
#[derive(Debug, Args, Clone)]
pub struct LogDescriptionArgs {
    #[clap(trailing_var_arg = true, allow_hyphen_values = false)]
    desc: String,
    #[clap(short, long)]
    user: Username,
}
#[derive(Debug, Args, Clone)]
pub struct NewLogArgs {
    #[clap(short, long)]
    author: UserIdOrNameArg,
    #[clap(short, long)]
    project: ProjectIdOrNameArg,
    text: String,
}
#[derive(Debug, Args, Clone)]
pub struct UsernameArg {
    username: Username,
}
#[derive(Debug, Args, Clone)]
pub struct UserProjectArgs {
    name: ProjectName,
    owner: UserIdOrNameArg,
}
#[derive(Debug, clap::Subcommand)]
enum ListArgs {
    Logs(ListLogsArgs),
    Projects(ListProjectsArgs),
    #[cfg(feature = "admin")]
    Users(PageArgs),
}
#[derive(Debug, Args, Clone)]
pub struct ListProjectsArgs {
    user: UserIdOrNameArg,
    #[clap(flatten)]
    page: PageArgs,
}
#[derive(Debug, Args, Clone)]
pub struct ListLogsArgs {
    project: ProjectIdOrNameArg,
    #[clap(flatten)]
    pagination: PageArgs,
}
#[derive(Debug, Clone, clap::Args)]
#[clap(group(
    ArgGroup::new("user")
        .required(true)
        .multiple(false)
        .args(&["id","name"])))]
struct UserIdOrNameArg {
    id: Option<UserId>,
    name: Option<Username>,
}
impl FromStr for UserIdOrNameArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = UserId::from_str(s) {
            Ok(Self {
                id: Some(id),
                name: None,
            })
        } else if let Ok(name) = Username::from_str(s) {
            Ok(Self {
                id: None,
                name: Some(name),
            })
        } else {
            Err("not an id not a name".into())
        }
    }
}
#[derive(Debug, Clone, clap::Args)]
#[clap(group(
    ArgGroup::new("project")
        .required(true)
        .multiple(false)
        .args(&["id","name"])))]
struct ProjectIdOrNameArg {
    id: Option<ProjectId>,
    name: Option<ProjectName>,
}
impl FromStr for ProjectIdOrNameArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = ProjectId::from_str(s) {
            Ok(Self {
                id: Some(id),
                name: None,
            })
        } else if let Ok(name) = ProjectName::from_str(s) {
            Ok(Self {
                id: None,
                name: Some(name),
            })
        } else {
            Err("not an id nor a valid name".into())
        }
    }
}
#[derive(Debug, Args, Clone, Display)]
#[display("{}", value.iter().join(" "))]
pub struct LogArg {
    #[clap(trailing_var_arg = true, allow_hyphen_values = false)]
    value: Vec<String>,
}
#[derive(Debug, Parser)]
pub struct ProjectLogArg {
    project: String,
    content: String,
}
#[derive(Debug, Args, Clone)]
pub struct PageArgs {
    #[clap(long, default_value = "1")]
    page: usize,
    #[clap(long, default_value = "100")]
    size: usize,
}
impl From<ProjectLogArg> for ProjectLog {
    fn from(ProjectLogArg { project, content }: ProjectLogArg) -> Self {
        Self::new(project, content)
    }
}
impl From<PageArgs> for Page {
    fn from(PageArgs { page, size }: PageArgs) -> Self {
        Self::new(page, size)
    }
}

async fn get_user_id<T>(
    UserIdOrNameArg { id, name }: UserIdOrNameArg,
    service: &T,
) -> Option<UserId>
where
    T: LocalLogStoreService,
{
    match (id, name) {
        (Some(id), _) => Some(id),
        (_, Some(name)) => service.get_user(name).await.map(|u| u.id()),
        (_, _) => None,
    }
}
async fn get_project_id<T>(
    ProjectIdOrNameArg { id, name }: ProjectIdOrNameArg,
    service: &T,
) -> Option<ProjectId>
where
    T: LocalLogStoreService,
{
    match (id, name) {
        (Some(id), _) => Some(id),
        (_, Some(name)) => service.get_project(name).await.map(|p| p.id()),
        (_, _) => None,
    }
}
#[tokio::main]
async fn main() {
    use tracing_subscriber::util::SubscriberInitExt;
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter::EnvFilter::from_default_env())
        .init();

    let repo = SqliteRepo::new("/tmp/db.sqlite").await.unwrap();
    let service = LogService::new(repo);

    let CliArgs { cmd } = CliArgs::parse();

    match cmd {
        CmdArgs::New(new) => match new {
            NewArgs::Log(NewLogArgs {
                author,
                project,
                text,
            }) => {
                let project_id = get_project_id(project, &service).await;
                let user_id = get_user_id(author, &service).await;
                if project_id.is_none() {
                    return println!("project not found");
                }
                if user_id.is_none() {
                    return println!("user not found");
                }
                match service
                    .add_log(user_id.unwrap(), project_id.unwrap(), text)
                    .await
                {
                    Ok(log) => println!("{log}"),
                    Err(e) => println!("{e}"),
                }
            }
            NewArgs::User(UsernameArg { username }) => match service.new_user(username).await {
                Ok(user) => println!("created {user}"),
                Err(_) => println!("Could not create user"),
            },
            NewArgs::Project(UserProjectArgs {
                name: project,
                owner,
            }) => {
                if let Some(user_id) = get_user_id(owner, &service).await {
                    match service.new_project(project, user_id).await {
                        Ok(project) => println!("created {project}"),
                        Err(e) => print!("{e}"),
                    }
                } else {
                    println!("user not found");
                }
            }
        },
        CmdArgs::List(list) => match list {
            ListArgs::Logs(ListLogsArgs {
                project,
                pagination,
            }) => {
                if let Some(project_id) = get_project_id(project, &service).await {
                    service.logs(project_id, pagination.into()).await.display()
                } else {
                    println!("project not found");
                }
            }
            ListArgs::Projects(ListProjectsArgs {
                page,
                user: UserIdOrNameArg { id, name },
            }) => match (id, name) {
                (Some(id), _) => println!("{}", service.projects_of(id, page.into()).await),
                (None, Some(name)) => service.projects_of_named(name, page.into()).await.display(),
                (_, _) => println!("oops"),
            },
            ListArgs::Users(page) => {
                println!("{}", service.list_users(page.into()).await);
            }
        },
        CmdArgs::Id(subarg) => match subarg {
            IdArgs::User(UsernameArg { username }) => {}
            IdArgs::Project(UserProjectArgs { name, owner }) => {}
        },
        CmdArgs::Search(subarg) => match subarg {
            SearchArgs::Logs(_) => {}
            SearchArgs::Projects(_) => {}
        },
    }
    // store.save(&db);
}

trait DisplayMonad {
    fn display(&self);
}

impl<T, E> DisplayMonad for Result<T, E>
where
    T: std::fmt::Display,
    E: std::fmt::Display,
{
    fn display(&self) {
        match self {
            Ok(t) => println!("{t}"),
            Err(e) => println!("{e}"),
        }
    }
}

impl<T> DisplayMonad for Option<T>
where
    T: std::fmt::Display,
{
    fn display(&self) {
        match self {
            Some(t) => println!("{t}"),
            None => println!(" ø "),
        }
    }
}

// we have projects
// a project is a string

// a project has a list of logs

// log can be a
// - trace (simple string)
// - associate (tag user on projecct with a message)
// - disociate user
// - resource (external like a link, internal)
// - update (update or replace a previous entry)
// - deletion (previous log)
// - expectation (waiting for something [from user] [before date] [])
// - task [deadline]
// - trigger (memo) [on event|date]

// lot 1: Rust store + api (project + log)
// lot.1.b: Many types

// lot 2: Rust api + postgres

// lot 3: Filter by predefined types (memo starting today)
