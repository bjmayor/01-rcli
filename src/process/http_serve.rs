use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};

use tracing::info;

#[derive(Debug)]
struct HtpServeState {
    path: PathBuf,
}
pub async fn process_http_serve(path: PathBuf, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Serving {:?} on {}", path, addr);
    let state = HtpServeState { path };
    let router = Router::new()
        .route("/*path", get(file_handler))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;
    // let server = rouille::Server::new(format!("
    Ok(())
}

async fn file_handler(
    State(state): State<Arc<HtpServeState>>,
    Path(path): Path<String>,
) -> (StatusCode, String) {
    let p = std::path::Path::new(&state.path).join(path);
    info!("Reading file: {:?}", p);
    if !p.exists() {
        return (StatusCode::NOT_FOUND, format!("File Not found:{:?}", p));
    }
    match tokio::fs::read_to_string(p).await {
        Ok(content) => (StatusCode::OK, content),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error reading file: {:?}", e),
        ),
    }
}
