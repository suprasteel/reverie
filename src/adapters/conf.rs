use std::{path::PathBuf, str::FromStr};

use tracing::info;

#[derive(Debug)]
pub struct Config {
    pub database: Database,
    pub preferences: Option<PathBuf>,
}
impl Config {
    pub fn from_env() -> Self {
        Self {
            database: Database::from_env(),
            preferences: std::env::var("REVERIE_USER_PREFS").map(PathBuf::from).ok(),
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            database: Database::default(),
            preferences: Some("./.reverie_user_prefs.conf".into()),
        }
    }
}

#[derive(Debug)]
pub enum Database {
    /// "sqlite:/path/db.sqlite"
    Sqlite(PathBuf),
    None,
}
impl Default for Database {
    fn default() -> Self {
        Self::Sqlite(PathBuf::from_str("/tmp/db.sqlite").unwrap())
    }
}

impl Database {
    pub fn from_env() -> Self {
        let url = std::env::var("REVERIE_DB").unwrap_or_default();
        if url.is_empty() {
            Self::None
        } else {
            let mut parts = url.split(":");
            let typ = parts.next();
            let res = parts.next();
            match (typ, res) {
                (Some("sqlite"), Some(file)) => match PathBuf::from_str(file) {
                    Ok(file) => Self::Sqlite(file),
                    Err(_) => {
                        info!("Database configuartion path invalid");
                        Self::None
                    }
                },
                (_, _) => Self::None,
            }
        }
    }
}
