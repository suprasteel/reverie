use axum::{
    extract::Query,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use axum_macros::debug_handler;
use reverie::{LocalLogStoreService, LogService, Page, Paged, ProjectId, SqliteRepo, UserId};
use serde::Deserialize;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::signal;
use tracing::info;
#[debug_handler]
async fn add_log(store: axum::extract::State<AppContext>, Json(log): Json) -> StatusCode {
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
/// Should fetch project id from (projcetname, userid)
/// then fetch log
/// the user sees a call like : api/project/
/// - api/project/<id>/logs (get last logs (paged))
/// - api/project/<id>/tasks
/// - api/project/<id>/logs?by=me or by=coline
/// - api/project/<id>/blockers/all
/// - api/project/<id>/blockers/solved
/// - api/project/<id>/blockers
/// - api/project/<id>/blockers
/// - api/project/<id>/update -> returns last change time and version
/// the headers contain the user id (jwt?)
#[debug_handler]
async fn project_logs(
    app: axum::extract::State<AppContext>,
    axum::extract::Path(project): axum::extract::Path<String>,
    pagination: Option<Query<Pagination>>,
) -> (StatusCode, Json<Paged<String>>) {
    let Query(page) = pagination.unwrap_or_default();
    (StatusCode::OK, app.fetch_log(project, &page.into()).into())
}

#[derive(Clone)]
struct AppContext {
    service: Arc<Mutex<LogService<SqliteRepo>>>,
}
impl From<LogService<SqliteRepo>> for AppContext {
    fn from(service: LogService<SqliteRepo>) -> Self {
        Self {
            service: Arc::new(Mutex::new(service)),
        }
    }
}
impl AppContext {
    fn add_log(&self, log: String, user: UserId, project: ProjectId) {
        info!("add {log:?}");
        self.service.lock().unwrap().add_log(user, project, log);
    }
    fn fetch_log(&self, project: ProjectId, page: &Page) -> Paged<String> {
        info!("get {project:?}");
        self.service.lock().unwrap().get(project, page)
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
        .route("/project/:project/logs", get(project_logs))
        .route("/project/:project/add/log", post(add_log));

    let repo = SqliteRepo::new("/tmp/db.sqlite").await.unwrap();
    let service = LogService::new(repo);
    let store: AppContext = service.into();
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
