use back::{
    CreateAuthorRequest, CreateLogRequest, CreateProjectRequest, LocalLogStoreService, LogService,
    Page, ProjectLog, SqliteRepo, Username,
};
use clap::{Args, Parser};
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
    #[clap(short, long, default_value = "default")]
    project: String,
    #[clap(short, long, default_value = "me")]
    author: Username,
    #[clap(flatten)]
    page: PageArg,
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
    Log(LogArg),
    User,
    Project,
}
#[derive(Debug, clap::Subcommand)]
pub enum ListArgs {
    Log,
    User,
    Project,
}
#[derive(Debug, Args, Clone)]
pub struct LogArg {
    #[clap(trailing_var_arg = true, allow_hyphen_values = false)]
    value: Vec<String>,
}
impl From<LogArg> for String {
    fn from(LogArg { value }: LogArg) -> Self {
        value.join(" ").to_string()
    }
}
#[derive(Debug, Parser)]
pub struct ProjectLogArg {
    project: String,
    content: String,
}
#[derive(Debug, Args, Clone)]
pub struct PageArg {
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
impl From<PageArg> for Page {
    fn from(PageArg { page, size }: PageArg) -> Self {
        Self::new(page, size)
    }
}
#[tokio::main]
async fn main() {
    // load config

    // set loging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    // setup db
    let db = home::home_dir().unwrap().join("s0O.db");
    // let mut store = LogsStore::load(&db);
    // set remote logs
    // load service and inject store
    let repo = SqliteRepo::new(db.to_str().unwrap()).await.unwrap();
    let service = LogService::new(repo);

    let CliArgs {
        cmd,
        project,
        author,
        page,
    } = CliArgs::parse();

    match cmd {
        CmdArgs::New(new) => match new {
            NewArgs::Log(log) => {
                let project = service.project(&project).await;
                if project.is_none() {
                    return println!("project not found");
                }
                let author = service.user(&author).await;
                if author.is_none() {
                    return println!("author not found");
                }
                match service
                    .add_log(CreateLogRequest {
                        author: author.unwrap().id(),
                        project: project.unwrap().id(),
                        text: log.into(),
                    })
                    .await
                {
                    Ok(log) => println!("{log:?}"),
                    Err(()) => println!("failed to add log"),
                }
            }
            NewArgs::User => {
                match service
                    .new_user(CreateAuthorRequest {
                        username: author.to_owned(),
                    })
                    .await
                {
                    Ok(user) => println!("user {user:?} created"),
                    Err(_) => println!("Could not create user"),
                }
            }
            NewArgs::Project => {
                let author = match service.user(&author).await {
                    None => return println!("author not found"),
                    Some(author) => author,
                };
                match service
                    .create_project(CreateProjectRequest {
                        author: author.id(),
                        project_name: project,
                    })
                    .await
                    .ok()
                {
                    Some(project) => {
                        println!("created project {} ({})", project.name(), project.id());
                    }
                    _ => {
                        println!("failed");
                    }
                }
            }
        },
        CmdArgs::List(list_arg) => {
            let project = match service.project(&project).await {
                None => return println!("project not found"),
                Some(p) => p,
            };
            match list_arg {
                ListArgs::Log => println!(
                    "{}",
                    service
                        .logs(project.id(), page.into())
                        .await
                        .map(|l| format!("{:?}", l))
                        .unwrap()
                ),
                ListArgs::User => {
                    println!(
                        "{:?}",
                        service
                            .user(&author)
                            .await
                            .map(|u| format!("{:?}", u))
                            .unwrap_or("no user".into())
                    );
                }
                ListArgs::Project => println!("{:?}", project),
            }
        }
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
