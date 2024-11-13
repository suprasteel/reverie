use back::{LogsStore, Page, ProjectLog};
use clap::Parser;
use itertools::Itertools;

// /// Six0One 601 > log
// #[derive(Debug, Parser)]
// #[command(name = "Six0One")]
// pub struct S0OArgs {
//     project: String,
//     content: String,
// }

#[derive(Debug, Parser)]
pub struct ProjectLogArg {
    project: String,
    content: String,
}

#[derive(Debug, Parser, Clone)]
pub struct PageArg {
    #[clap(short, long)]
    page: usize,
    #[clap(short, long)]
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

#[derive(Debug, Parser)]
pub struct ListArg {
    project: String,
    // page: PageArg,
}

#[derive(Debug, clap::Subcommand)]
pub enum CmdArgs {
    AddLog(ProjectLogArg),
    List(ListArg),
}

#[derive(Debug, Parser)]
pub struct CliArgs {
    #[clap(subcommand)]
    cmd: CmdArgs,
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let mut store = LogsStore::load();

    match CliArgs::parse().cmd {
        CmdArgs::AddLog(pl) => store.add(pl.into()),
        CmdArgs::List(ListArg {
            project, /*, page*/
        }) => {
            println!("{}", store.get(project, Page::new(1, 2)).iter().join("\n"))
        }
    }

    store.save();
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
