use axum::{
    extract::Query,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use axum_macros::debug_handler;
use back::{LogsStore, Page, Paged, ProjectLog};
use serde::Deserialize;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::signal;
use tracing::info;
#[debug_handler]
async fn add_log(
    store: axum::extract::State<LogsService>,
    Json(log): Json<ProjectLog>,
) -> StatusCode {
    store.add_log(log);
    StatusCode::CREATED
}
#[derive(Deserialize, Debug)]
struct Pagination {
    page: usize,
    size: usize,
}
impl From<Pagination> for Page {
    fn from(Pagination { page, size }: Pagination) -> Self {
        Self::new(page, size)
    }
}
impl Default for Pagination {
    fn default() -> Self {
        Self { page: 1, size: 10 }
    }
}
#[debug_handler]
async fn project_logs(
    store: axum::extract::State<LogsService>,
    axum::extract::Path(project): axum::extract::Path<String>,
    pagination: Option<Query<Pagination>>,
) -> (StatusCode, Json<Paged<String>>) {
    let Query(page) = pagination.unwrap_or_default();
    (StatusCode::OK, store.get(project, &page.into()).into())
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
        info!("add {log:?}");
        self.store.lock().unwrap().add(log);
    }
    fn get(&self, project: String, page: &Page) -> Paged<String> {
        info!("get {project:?}");
        self.store.lock().unwrap().get(project, page)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let s0o_bind_ip: String = std::env::var("S0O_BIND_IP").unwrap_or("127.0.0.1".to_string());
    let s0o_bind_port: String = std::env::var("S0O_BIND_PORT").unwrap_or("3000".to_string());

    let app = Router::new()
        .route("/project/:project", get(project_logs))
        .route("/new/log", post(add_log));

    let db = home::home_dir().unwrap().join("s0O.db");
    let logstore = LogsStore::load(&db);
    let store: LogsService = logstore.into();
    let store_clone = store.clone();
    {
        let addr: SocketAddr = format!("{}:{}", s0o_bind_ip, s0o_bind_port)
            .parse()
            .expect("ip:port binding invalid");
        info!("listening on {}", addr);
        let handle = axum_server::Handle::new();
        let shutdown_signal_fut = shutdown_signal();
        let server = axum_server::bind(addr)
            .handle(handle.clone())
            .serve(app.with_state(store).into_make_service());

        tokio::select! {
        () = shutdown_signal_fut =>
            handle.graceful_shutdown(Some(Duration::from_secs(5))),
            res = server => res.unwrap(),
        }
        info!("Server is stopping");
    }
    store_clone.store.lock().unwrap().save(db);
}

// https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    info!("Terminate signal received");
}
