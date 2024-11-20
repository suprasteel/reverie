use std::{collections::HashMap, path::Path};

mod adapters;
mod core;
// make pagination public
pub use adapters::Config;
pub use adapters::Database;
pub use adapters::SqliteRepo;
pub use core::model::Project;
pub use core::model::ProjectId;
pub use core::model::ProjectName;
pub use core::model::UserId;
pub use core::model::Username;
pub use core::pagination::{Page, Paged, Paginable};
pub use core::repo::{CreateAuthorRequest, CreateLogRequest, CreateProjectRequest};
pub use core::service::{LocalLogStoreService, LogService};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument, warn};

// #[derive(Debug)]
// pub struct ContentId(u64);
// #[derive(Debug)]
// pub struct DocumentId(u64);
// #[derive(Debug)]
// pub struct BlockerId(EntryId);
// #[derive(Debug)]
// pub struct TaskId(EntryId);

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectLog {
    project: String,
    content: String,
}
impl ProjectLog {
    pub fn new(project: String, content: String) -> Self {
        Self { project, content }
    }
}
// pub struct Task {
//     id: TaskId,
//     meta: Metadata,
//     content: ContentId,
//     done: bool,
// }
// pub struct Blocker {
//     id: BlockerId,
//     meta: Metadata,
//     content: ContentId,
//     solved: bool,
// }
// pub struct Reminder {
//     id: BlockerId,
//     meta: Metadata,
//     content: ContentId,
//     solved: bool,
// }
// pub struct Document {
//     id: EntryId,
//     revision: Revision,
//     meta: Metadata,
//     document: DocumentId,
// }
// pub struct Share {
//     id: EntryId,
//     meta: Metadata,
//     user: UserId,
//     revoked: bool,
// }
// pub struct Unshare {
//     id: EntryId,
//     share_id: EntryId,
//     meta: Metadata,
// }
// pub enum Condition {
//     Date(u64),
//     Completion(TaskId),
//     Solved(BlockerId),
//     // Every(...
// }
// pub struct Trigger {
//     on: Condition,
//     desc: String,
//     /// maximum number of times to trigger
//     times: u32,
//     action: (), // todo
// }
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LogsStore {
    store: HashMap<String, Vec<String>>,
}
pub type StoreResult<T> = Result<T, ()>;
impl LogsStore {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        if !path.exists() {
            return LogsStore::default();
        }
        let db = std::fs::File::options()
            .truncate(false)
            .read(true)
            .open(path)
            .unwrap();
        let store = serde_json::from_reader(db)
            .map_err(|e| warn!("{e}"))
            .unwrap_or_default();
        info!("{:?}", store);
        store
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        if self.store.is_empty() {
            return;
        }
        let path = path.as_ref();
        let mut file = match std::fs::File::options()
            .write(true)
            .truncate(true)
            .open(path)
        {
            Ok(file) => file,
            Err(_) => std::fs::File::create_new(path).expect("Cannot create db"),
        };
        serde_json::to_writer(&mut file, &self)
            .map_err(|e| warn!("{e}"))
            .unwrap_or_default();
        info!("{:?}", self);
    }
    #[instrument(level = "info")]
    pub fn add(&mut self, log: ProjectLog) {
        let ProjectLog { project, content } = log;
        if let Some(list) = self.store.get_mut(&project) {
            list.push(content);
        } else {
            self.store.insert(project, vec![content]);
        }
    }
    // #[instrument(level = "info")]
    // pub fn log(&mut self, project: ProjectId, log: Text) {
    //     if let Some(list) = self.store.get_mut(&project) {
    //         list.push(content);
    //     } else {
    //         self.store.insert(project, vec![content]);
    //     }
    // }
    // pub fn upd(&mut self, log: Log);
    // pub fn share(&mit self, log: LogId, user: UserId) -> StoreResult<()> {
    // }
    #[instrument(level = "info")]
    pub fn get(&self, project: String, page: &Page) -> Paged<String> {
        use core::pagination::Paginable;
        self.store
            .get(&project)
            .map(|list| list.get_page(page))
            .unwrap_or_default()
    }
}
