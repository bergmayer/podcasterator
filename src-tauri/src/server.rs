use crate::files::get_mime_type;
use crate::state::AppState;
use axum::{
    body::Body,
    extract::{Path as AxumPath, State as AxumState},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use chrono::Utc;
use rss::{ChannelBuilder, EnclosureBuilder, GuidBuilder, ImageBuilder, ItemBuilder};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ServerState {
    pub app_state: AppState,
    pub cache_dir: PathBuf,
    pub base_url: String,
}

/// Detect the local IP address (non-loopback IPv4)
pub fn get_local_ip() -> Result<String, String> {
    local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .map_err(|e| format!("Failed to get local IP: {}", e))
}

/// Generate RSS feed XML
fn generate_feed(state: &ServerState) -> String {
    let mut items = Vec::new();

    for (i, file) in state.app_state.files.iter().enumerate() {
        let path = PathBuf::from(&file.temp_path);
        let file_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let mime_type = get_mime_type(&path);

        // Calculate pub date (first file = newest)
        let pub_date = Utc::now() - chrono::Duration::minutes(i as i64);

        let file_url = format!(
            "{}/files/{}/{}",
            state.base_url,
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
            .pub_date(Some(pub_date.to_rfc2822()))
            .enclosure(Some(enclosure))
            .build();

        items.push(item);
    }

    let mut channel_builder = ChannelBuilder::default();
    channel_builder
        .title(&state.app_state.podcast_name)
        .link(&state.base_url)
        .description("Local podcast feed")
        .last_build_date(Some(Utc::now().to_rfc2822()))
        .items(items);

    // Add artwork if available
    if let Some(ref artwork_path) = state.app_state.artwork_path {
        if PathBuf::from(artwork_path).exists() {
            let image = ImageBuilder::default()
                .url(format!("{}/artwork.jpg", state.base_url))
                .title(&state.app_state.podcast_name)
                .link(&state.base_url)
                .build();
            channel_builder.image(Some(image));
        }
    }

    let channel = channel_builder.build();
    channel.to_string()
}

/// Serve the RSS feed
async fn feed_handler(AxumState(state): AxumState<Arc<RwLock<ServerState>>>) -> impl IntoResponse {
    let state = state.read().await;
    let feed = generate_feed(&state);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")
        .body(Body::from(feed))
        .unwrap()
}



/// Serve audio files
async fn file_handler(
    AxumState(state): AxumState<Arc<RwLock<ServerState>>>,
    AxumPath((uuid, filename)): AxumPath<(String, String)>,
) -> impl IntoResponse {
    let state = state.read().await;

    // Security: Clean the path to prevent directory traversal
    let clean_uuid = path_clean::clean(&uuid);
    let clean_filename = path_clean::clean(&filename);

    if clean_uuid.to_string_lossy().contains("..") || clean_filename.to_string_lossy().contains("..") {
        return (StatusCode::FORBIDDEN, "Invalid path").into_response();
    }

    let file_path = state.cache_dir.join(clean_uuid).join(clean_filename);

    serve_file(&file_path).await
}

async fn serve_file(file_path: &PathBuf) -> Response {
    let file = match tokio::fs::File::open(file_path).await {
        Ok(file) => file,
        Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };

    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mime_type = get_mime_type(file_path);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .body(body)
        .unwrap()
        .into_response()
}

/// Serve artwork
async fn artwork_handler(
    AxumState(state): AxumState<Arc<RwLock<ServerState>>>,
) -> impl IntoResponse {
    let state = state.read().await;

    let artwork_path = match &state.app_state.artwork_path {
        Some(p) => PathBuf::from(p),
        None => return (StatusCode::NOT_FOUND, "No artwork").into_response(),
    };

    serve_file(&artwork_path).await
}

/// Start the HTTP server
pub async fn launch_server(
    app_state: AppState,
    cache_dir: PathBuf,
) -> Result<(String, tokio::sync::oneshot::Sender<()>), String> {
    let ip = get_local_ip()?;
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("Failed to bind to a port: {}", e))?;
    
    let port = listener.local_addr().unwrap().port();
    let base_url = format!("http://{}:{}", ip, port);

    let server_state = Arc::new(RwLock::new(ServerState {
        app_state,
        cache_dir,
        base_url: base_url.clone(),
    }));

    let app = Router::new()
        .route("/feed.xml", get(feed_handler))
        .route("/files/{uuid}/{filename}", get(file_handler))
        .route("/artwork.jpg", get(artwork_handler))
        .with_state(server_state);

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            })
            .await
            .ok();
    });

    Ok((format!("{}/feed.xml", base_url), shutdown_tx))
}
