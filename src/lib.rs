use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use tracing::{info, instrument, warn};

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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LogsStore {
    store: HashMap<String, Vec<String>>,
}
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
    pub fn save<P: AsRef<Path>>(self, path: P) {
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
    #[instrument(level = "info")]
    pub fn get(&self, project: String, page: &Page) -> Vec<String> {
        self.store
            .get(&project)
            .map(|list| list.get(page))
            .unwrap_or_default()
    }
}
#[derive(Debug, Clone, Deserialize)]
pub struct Page {
    page: usize,
    size: usize,
}
impl Default for Page {
    fn default() -> Self {
        Self { page: 1, size: 10 }
    }
}
impl Page {
    pub fn new(page: usize, size: usize) -> Self {
        Self { page, size }
    }
    pub fn number(&self) -> usize {
        self.page
    }
}
pub trait Paginable {
    type Output;
    fn get(&self, page: &Page) -> Vec<Self::Output>;
}
impl<T> Paginable for Vec<T>
where
    T: Clone,
{
    type Output = T;
    fn get(&self, page: &Page) -> Vec<Self::Output> {
        let Page { page, size } = page;
        if self.len() < *size {
            return self.to_vec();
        }
        let pages_count = (self.len() / size) + 1;
        let page = if *page > pages_count {
            pages_count
        } else {
            *page
        };
        let offset = (page - 1) * size;
        self[offset..(offset + size)].to_vec()
    }
}
