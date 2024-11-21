use std::str::FromStr;

use clap::{ArgGroup, Args, Parser};
use derive_more::derive::Display;
use itertools::Itertools;
use reverie::{
    LocalLogStoreService, LogService, Page, ProjectId, ProjectLog, ProjectName, SqliteRepo, UserId,
    Username,
};
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
}
#[derive(Debug, clap::Subcommand)]
pub enum NewArgs {
    Log(NewLogArgs),
    User(NewUserArgs),
    Project(NewProjectArgs),
}
#[derive(Debug, Args, Clone)]
pub struct NewLogArgs {
    #[clap(short, long)]
    author: UserIdOrNameArg,
    #[clap(short, long)]
    project: ProjectId,
    text: String,
}
#[derive(Debug, Args, Clone)]
pub struct NewUserArgs {
    username: Username,
}
#[derive(Debug, Args, Clone)]
pub struct NewProjectArgs {
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
    project: ProjectId,
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
        (_, Some(name)) => service.get_user_id(name).await.map(|u| u.id()),
        (_, _) => None,
    }
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
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
                if let Some(user_id) = get_user_id(author, &service).await {
                    match service.add_log(user_id, project, text).await {
                        Ok(log) => println!("{log}"),
                        Err(e) => println!("{e}"),
                    }
                } else {
                    println!("user not found");
                }
            }
            NewArgs::User(NewUserArgs { username }) => match service.new_user(username).await {
                Ok(user) => println!("created {user}"),
                Err(_) => println!("Could not create user"),
            },
            NewArgs::Project(NewProjectArgs {
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
            }) => service.logs(project, pagination.into()).await.display(),
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
