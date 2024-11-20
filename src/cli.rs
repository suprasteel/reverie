use clap::{Args, Parser};
use derive_more::derive::Display;
use itertools::Itertools;
use reverie::{
    LocalLogStoreService, LogService, Page, ProjectId, ProjectLog, ProjectName, SqliteRepo, UserId,
    Username,
};
// /// Six0One 601 > log
// #[derive(Debug, Parser)]
// #[command(name = "Six0One")]
// pub struct S0OArgs {
//     project: String,
//     content: String,
// }
#[derive(Debug, Parser)]
pub struct CliArgs {
    #[clap(subcommand)]
    cmd: CmdArgs,
    // #[clap(short, long, default_value = "default")]
    // project: String,
    // #[clap(short, long, default_value = "me")]
    // author: String,
    // #[clap(flatten)]
    // page: PageArg,
}
#[derive(Debug, clap::Subcommand)]
pub enum CmdArgs {
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
    author: UserId,
    project: ProjectId,
    text: String,
}
#[derive(Debug, Args, Clone)]
pub struct NewUserArgs {
    username: Username,
}
#[derive(Debug, Args, Clone)]
pub struct NewProjectArgs {
    owner: UserId,
    name: ProjectName,
}
#[derive(Debug, clap::Subcommand)]
pub enum ListArgs {
    Logs(ListLogsArgs),
    Projects(ListProjectsArgs),
    #[cfg(feature = "admin")]
    Users(PageArgs),
}
#[derive(Debug, Args, Clone)]
pub struct ListLogsArgs {
    project: ProjectId,
    #[clap(flatten)]
    pagination: PageArgs,
}
#[derive(Debug, Args, Clone)]
pub struct ListProjectsArgs {
    user: UserId,
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
#[tokio::main]
async fn main() {
    // load config
    // let config = Config::default();

    // set loging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let repo = SqliteRepo::new("/tmp/db.sqlite").await.unwrap();
    let service = LogService::new(repo);

    let CliArgs {
        cmd,
        // project,
        // author,
        // page,
    } = CliArgs::parse();

    match cmd {
        CmdArgs::New(new) => match new {
            NewArgs::Log(NewLogArgs {
                author,
                project,
                text,
            }) => match service.add_log(author, project, text).await {
                Ok(log) => println!("{log:?}"),
                Err(e) => println!("{e}"),
            },
            NewArgs::User(NewUserArgs { username }) => match service.new_user(username).await {
                Ok(user) => println!("created {user}"),
                Err(_) => println!("Could not create user"),
            },
            NewArgs::Project(NewProjectArgs { owner, name }) => {
                match service.new_project(name, owner).await {
                    Ok(project) => println!("created {project}"),
                    Err(e) => print!("{e}"),
                }
            }
        },
        CmdArgs::List(list) => match list {
            ListArgs::Logs(ListLogsArgs {
                project,
                pagination,
            }) => match service.logs(project, pagination.into()).await {
                Ok(logs) => println!("{logs}"),
                Err(e) => print!("{e}"),
            },
            ListArgs::Projects(ListProjectsArgs { user }) => {
                println!("{:?}", service.projects_of(user).await);
            }
            ListArgs::Users(page) => {
                println!("{:?}", service.list_users(page.into()).await);
            }
        },
    }
    // store.save(&db);
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
