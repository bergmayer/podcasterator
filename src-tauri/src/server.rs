use crate::files::{get_mime_type, ARTWORK_FILENAME};
use crate::state::AppState;
use axum::{
    extract::{Path as AxumPath, Request, State as AxumState},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use rss::{ChannelBuilder, EnclosureBuilder, GuidBuilder, ImageBuilder, ItemBuilder};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::watch;
use tower::util::ServiceExt;
use tower_http::services::ServeFile;

pub struct ServerState {
    pub state_rx: watch::Receiver<AppState>,
    pub base_url: String,
}

/// Detect the local IP address (non-loopback IPv4)
pub fn get_local_ip() -> Result<String, String> {
    local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .map_err(|e| format!("Failed to get local IP: {}", e))
}

/// Generate RSS feed XML
fn generate_feed(app_state: &AppState, base_url: &str) -> String {
    let mut items = Vec::new();

    for file in &app_state.files {
        let path = PathBuf::from(&file.temp_path);
        let file_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let mime_type = get_mime_type(&path);

        let pub_date = (!file.added_at.is_empty()).then(|| file.added_at.clone());

        let file_url = format!(
            "{}/files/{}/{}",
            base_url,
            file.id,
            urlencoding::encode(&file.display_name)
        );

        let enclosure = EnclosureBuilder::default()
            .url(&file_url)
            .mime_type(mime_type)
            .length(file_size.to_string())
            .build();

        let guid = GuidBuilder::default()
            .value(&file.id)
            .permalink(false)
            .build();

        let item = ItemBuilder::default()
            .title(Some(file.display_name.clone()))
            .link(Some(file_url.clone()))
            .guid(Some(guid))
            .pub_date(pub_date)
            .enclosure(Some(enclosure))
            .build();

        items.push(item);
    }

    let mut channel_builder = ChannelBuilder::default();
    channel_builder
        .title(&app_state.podcast_name)
        .link(base_url)
        .description("Local podcast feed")
        .last_build_date(Some(chrono::Utc::now().to_rfc2822()))
        .items(items);

    // Add artwork if available
    if let Some(ref artwork_path) = app_state.artwork_path {
        if PathBuf::from(artwork_path).exists() {
            let image = ImageBuilder::default()
                .url(format!("{}/{}", base_url, ARTWORK_FILENAME))
                .title(&app_state.podcast_name)
                .link(base_url)
                .build();
            channel_builder.image(Some(image));
        }
    }

    let channel = channel_builder.build();
    channel.to_string()
}

/// Serve the RSS feed
async fn feed_handler(AxumState(state): AxumState<Arc<ServerState>>) -> impl IntoResponse {
    let app_state = state.state_rx.borrow().clone();
    let feed = generate_feed(&app_state, &state.base_url);

    (
        [(axum::http::header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        feed,
    )
}

/// Serve audio files.
///
/// The file is looked up by ID in the app state rather than by joining
/// request input onto the cache directory, so a request can never address
/// anything outside the current file list. The filename path segment only
/// exists to give podcast apps a nice download name.
async fn file_handler(
    AxumState(state): AxumState<Arc<ServerState>>,
    AxumPath((uuid, _filename)): AxumPath<(String, String)>,
    req: Request,
) -> Response {
    let temp_path = state
        .state_rx
        .borrow()
        .files
        .iter()
        .find(|f| f.id == uuid)
        .map(|f| PathBuf::from(&f.temp_path));

    match temp_path {
        Some(path) => serve_file(&path, req).await,
        None => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

/// Serve artwork
async fn artwork_handler(
    AxumState(state): AxumState<Arc<ServerState>>,
    req: Request,
) -> Response {
    let artwork_path = state.state_rx.borrow().artwork_path.clone();

    match artwork_path {
        Some(p) => serve_file(Path::new(&p), req).await,
        None => (StatusCode::NOT_FOUND, "No artwork").into_response(),
    }
}

/// Serve a file with Range, HEAD, and Content-Length support.
async fn serve_file(path: &Path, req: Request) -> Response {
    let mime = get_mime_type(path)
        .parse::<mime::Mime>()
        .unwrap_or(mime::APPLICATION_OCTET_STREAM);

    match ServeFile::new_with_mime(path, &mime).oneshot(req).await {
        Ok(res) => res.into_response(),
        Err(e) => {
            log::error!("Failed to serve {}: {}", path.display(), e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
        }
    }
}

/// Start the HTTP server. Returns (feed_url, shutdown_sender, state_sender).
pub async fn launch_server(
    app_state: AppState,
) -> Result<(String, tokio::sync::oneshot::Sender<()>, watch::Sender<AppState>), String> {
    let ip = get_local_ip()?;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("Failed to bind to a port: {}", e))?;

    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();
    let base_url = format!("http://{}:{}", ip, port);

    let (state_tx, state_rx) = watch::channel(app_state);

    let server_state = Arc::new(ServerState {
        state_rx,
        base_url: base_url.clone(),
    });

    let app = Router::new()
        .route("/feed.xml", get(feed_handler))
        .route("/files/{uuid}/{filename}", get(file_handler))
        .route(&format!("/{}", ARTWORK_FILENAME), get(artwork_handler))
        .with_state(server_state);

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            })
            .await
        {
            log::error!("HTTP server error: {}", e);
        }
    });

    Ok((format!("{}/feed.xml", base_url), shutdown_tx, state_tx))
}
