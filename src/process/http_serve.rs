use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::fs;

use tower_http::services::ServeDir;
use tracing::info;

#[derive(Debug)]
struct HtpServeState {
    path: PathBuf,
}
pub async fn process_http_serve(path: PathBuf, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Serving {:?} on {}", path, addr);
    let state = HtpServeState { path: path.clone() };
    let dir_service = ServeDir::new(path);
    let router = Router::new()
        .nest_service("/tower", dir_service)
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
) -> Result<impl IntoResponse, HttpError> {
    let p = std::path::Path::new(&state.path).join(path.clone());
    info!("Reading file: {:?}", p);
    if !p.exists() {
        return Err(HttpError::NotFound(path.clone()));
    }
    // if p is a directory, generate a directory listing
    if p.is_dir() {
        match process_dir(p).await {
            Ok(content) => {
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(content)
                    .map_err(|_| HttpError::Internal));
            }
            Err(_) => {
                return Err(HttpError::Internal);
            }
        }
    }

    // return (StatusCode::OK, content);
    match tokio::fs::read_to_string(p).await {
        Ok(content) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(content)
                .map_err(|_| HttpError::Internal)?;

            Ok(Ok(response))
        }
        Err(_) => Err(HttpError::Internal),
    }
}

async fn process_dir(path: impl AsRef<std::path::Path>) -> Result<String> {
    let mut content = String::new();
    content.push_str("<html><body><ul>");
    let mut entries = fs::read_dir(path).await?;
    // Iterate over directory entries using StreamExt
    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        let name = entry_path.file_name().unwrap().to_str().unwrap();
        content.push_str(&format!(
            "<li><a href=\"{}\">{}</a></li>",
            entry_path.display().to_string().trim_start_matches('.'),
            name
        ));
    }

    content.push_str("</ul></body></html>");

    Ok(content)
}

#[derive(Debug)]
enum HttpError {
    NotFound(String),
    Internal,
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        let (code, msg) = match self {
            HttpError::NotFound(resource) => (
                StatusCode::NOT_FOUND,
                format!("{} not found", resource).to_string(),
            ),
            HttpError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
        };
        (code, msg).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_file_handler() {
        let state = Arc::new(HtpServeState {
            path: PathBuf::from("."),
        });
        let result = file_handler(State(state), Path("Cargo.toml".to_string())).await;
        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
