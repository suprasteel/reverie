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

// #[derive(Debug)]
// pub struct ContentId(u64);
// #[derive(Debug)]
// pub struct DocumentId(u64);
// #[derive(Debug)]
// pub struct BlockerId(EntryId);
// #[derive(Debug)]
// pub struct TaskId(EntryId);

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
