use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    database: Database,
    userfile: PathBuf,
}

#[derive(Debug)]
pub enum Database {
    /// "sqlite:/path/db.sqlite"
    Sqlite(PathBuf),
}
