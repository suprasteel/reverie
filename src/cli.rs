use back::{LogsStore, Page, ProjectLog};
use clap::{Args, Parser};
use itertools::Itertools;
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
}
#[derive(Debug, clap::Subcommand)]
pub enum CmdArgs {
    #[clap(subcommand)]
    New(NewArgs),
    List(PageArg),
}
#[derive(Debug, clap::Subcommand)]
pub enum NewArgs {
    Log(LogArg),
    Other(LogArg),
}
#[derive(Debug, Args, Clone)]
pub struct LogArg {
    #[clap(trailing_var_arg = true, allow_hyphen_values = true)]
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
    #[clap(short, long, default_value = "1")]
    page: usize,
    #[clap(short, long, default_value = "5")]
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
fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();
    let db = home::home_dir().unwrap().join("s0O.db");
    let mut store = LogsStore::load(&db);
    let CliArgs { cmd, project } = CliArgs::parse();
    match cmd {
        CmdArgs::New(new) => match new {
            NewArgs::Log(log) => store.add(ProjectLog::new(project, log.into())),
            _ => print!("do nothing"),
        },
        CmdArgs::List(page) => {
            println!(
                "{}",
                store.get(project, &page.into()).data.iter().join("\n")
            )
        }
    }
    store.save(&db);
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
