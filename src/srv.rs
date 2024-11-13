use axum::{
    extract::Query,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use axum_macros::debug_handler;
use back::{LogsStore, Page, ProjectLog};
use serde::Serialize;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
#[debug_handler]
async fn add_log(
    store: axum::extract::State<LogsService>,
    Json(log): Json<ProjectLog>,
) -> StatusCode {
    // Access the shared store and add the new log entry
    store.add_log(log);
    StatusCode::CREATED
}
#[debug_handler]
async fn project_logs(
    store: axum::extract::State<LogsService>,
    axum::extract::Path(project): axum::extract::Path<String>,
    Query(page): Query<Option<Page>>,
) -> (StatusCode, Json<Paged<String>>) {
    let page = page.unwrap_or_default();
    (StatusCode::OK, store.get(project, &page).into())
}

#[derive(Clone)]
struct LogsService {
    store: Arc<Mutex<LogsStore>>,
}
impl From<LogsStore> for LogsService {
    fn from(store: LogsStore) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
        }
    }
}
impl LogsService {
    fn add_log(&self, log: ProjectLog) {
        self.store.lock().unwrap().add(log);
    }
    fn get(&self, project: String, page: &Page) -> Paged<String> {
        let list = self.store.lock().unwrap().get(project, page);
        Paged {
            page: page.number(),
            data: list,
        }
    }
}
#[derive(Serialize)]
struct Paged<T> {
    page: usize,
    data: Vec<T>,
}

#[tokio::main]
async fn main() {
    let s0o_bind_ip: String = std::env::var("S0O_BIND_IP").unwrap_or("127.0.0.1".to_string());
    let s0o_bind_port: String = std::env::var("S0O_BIND_PORT").unwrap_or("3000".to_string());

    let app = Router::new()
        .route("/project/{project}", get(project_logs))
        .route("/new/log", post(add_log));

    let db = home::home_dir().unwrap().join("s0O.db");
    let store = LogsStore::load(db).into();

    let addr: SocketAddr = format!("{}:{}", s0o_bind_ip, s0o_bind_port)
        .parse()
        .expect("ip:port binding invalid");
    println!("listening on {}", addr);
    axum_server::bind(addr)
        .serve(app.with_state(store).into_make_service())
        .await
        .unwrap();
}
