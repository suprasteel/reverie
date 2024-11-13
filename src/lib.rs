use std::{collections::HashMap, fs::File, io::Read};

use serde::{Deserialize, Serialize};
use tracing::instrument;

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

#[derive(Debug, Default)]
pub struct LogsStore {
    store: HashMap<String, Vec<String>>,
}
pub fn db() -> File {
    let path = home::home_dir().unwrap().join("s0O.db");

    match std::fs::File::options()
        .append(true)
        .truncate(false)
        .read(true)
        .open(&path)
    {
        Ok(file) => file,
        _ => std::fs::File::create_new(path).unwrap(),
    }
}
impl LogsStore {
    pub fn load() -> Self {
        let mut data = Vec::new();
        db().read_to_end(&mut data).unwrap();
        let data = serde_cbor::from_slice(&data);
        let mut store = Self::default();
        for plog in data.into_iter() {
            store.add(plog);
        }
        store
    }

    pub fn save(self) {
        let data = self.store.into_iter().collect::<Vec<_>>();
        serde_cbor::to_writer(&mut db(), &data).unwrap();
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
    pub fn get(&self, project: String, page: Page) -> Vec<String> {
        self.store
            .get(&project)
            .map(|list| list.get(page))
            .unwrap_or_default()
    }
}
#[derive(Debug, Clone)]
pub struct Page {
    page: usize,
    size: usize,
}
impl Page {
    pub fn new(page: usize, size: usize) -> Self {
        Self { page, size }
    }
}
pub trait Paginable {
    type Output;
    fn get(&self, page: Page) -> Vec<Self::Output>;
}
impl<T> Paginable for Vec<T>
where
    T: Clone,
{
    type Output = T;
    fn get(&self, page: Page) -> Vec<Self::Output> {
        let Page { page, size } = page;
        if self.len() < size {
            return self.to_vec();
        }
        let pages_count = (self.len() / size) + 1;
        let page = if page > pages_count {
            pages_count
        } else {
            page
        };
        let offset = (page - 1) * size;
        self[offset..(offset + size)].to_vec()
    }
}
