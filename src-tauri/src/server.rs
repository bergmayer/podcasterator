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
use tokio::sync::{watch, Semaphore};
use tower::util::ServiceExt;
use tower_http::services::ServeFile;

pub struct ServerState {
    pub state_rx: watch::Receiver<AppState>,
    pub base_url: String,
    pub feed_permit: Arc<Semaphore>,
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
    let publish_anchor = app_state
        .files
        .iter()
        .filter_map(|file| chrono::DateTime::parse_from_rfc2822(&file.added_at).ok())
        .max();

    for (index, file) in app_state.files.iter().enumerate() {
        let path = PathBuf::from(&file.temp_path);
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        let file_size = metadata.len();
        let mime_type = get_mime_type(&path);

        // Podcast clients commonly sort items newest-first by pubDate instead
        // of respecting XML order. Give each item a unique date based on its
        // current list position so the feed order always matches the UI.
        let pub_date = publish_anchor
            .as_ref()
            .map(|anchor| (*anchor - chrono::Duration::seconds(index as i64)).to_rfc2822());

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
async fn feed_handler(AxumState(state): AxumState<Arc<ServerState>>) -> Response {
    let Ok(permit) = state.feed_permit.clone().try_acquire_owned() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Feed is busy; retry shortly",
        )
            .into_response();
    };
    let app_state = state.state_rx.borrow().clone();
    let base_url = state.base_url.clone();
    match tokio::task::spawn_blocking(move || {
        let _permit = permit;
        generate_feed(&app_state, &base_url)
    })
    .await
    {
        Ok(feed) => (
            [(
                axum::http::header::CONTENT_TYPE,
                "application/rss+xml; charset=utf-8",
            )],
            feed,
        )
            .into_response(),
        Err(error) => {
            log::error!("Feed generation failed: {error}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Feed unavailable").into_response()
        }
    }
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
async fn artwork_handler(AxumState(state): AxumState<Arc<ServerState>>, req: Request) -> Response {
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
) -> Result<
    (
        String,
        tokio::sync::oneshot::Sender<()>,
        watch::Sender<AppState>,
    ),
    String,
> {
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
        feed_permit: Arc::new(Semaphore::new(1)),
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

#[cfg(test)]
mod tests {
    use super::generate_feed;
    use crate::state::{AppState, AudioFile};
    use std::io::Cursor;

    fn audio_file(root: &std::path::Path, id: &str, name: &str, added_at: &str) -> AudioFile {
        let path = root.join(id);
        std::fs::write(&path, b"fixture").unwrap();
        AudioFile {
            id: id.to_string(),
            original_path: format!("/original/{name}"),
            temp_path: path.to_string_lossy().into_owned(),
            display_name: name.to_string(),
            added_at: added_at.to_string(),
        }
    }

    #[test]
    fn feed_dates_preserve_the_displayed_file_order() {
        let root = tempfile::tempdir().unwrap();
        let state = AppState {
            files: vec![
                audio_file(
                    root.path(),
                    "one",
                    "Episode 01.mp3",
                    "Wed, 01 Jul 2026 12:00:00 +0000",
                ),
                audio_file(
                    root.path(),
                    "two",
                    "Episode 02.mp3",
                    "Thu, 02 Jul 2026 12:00:00 +0000",
                ),
                audio_file(
                    root.path(),
                    "three",
                    "Episode 03.mp3",
                    "Fri, 03 Jul 2026 12:00:00 +0000",
                ),
            ],
            podcast_name: "Test Podcast".to_string(),
            artwork_path: None,
        };

        let xml = generate_feed(&state, "http://127.0.0.1:1234");
        let channel = rss::Channel::read_from(Cursor::new(xml)).unwrap();
        let items = channel.items();

        assert_eq!(
            items
                .iter()
                .map(|item| item.title().unwrap())
                .collect::<Vec<_>>(),
            vec!["Episode 01.mp3", "Episode 02.mp3", "Episode 03.mp3"]
        );

        let dates = items
            .iter()
            .map(|item| chrono::DateTime::parse_from_rfc2822(item.pub_date().unwrap()).unwrap())
            .collect::<Vec<_>>();
        assert!(dates.windows(2).all(|pair| pair[0] > pair[1]));
    }
    #[tokio::test]
    async fn feed_work_is_bounded_and_observes_updated_state() {
        use super::*;
        use axum::body::to_bytes;
        let (tx, rx) = watch::channel(AppState::default());
        let server = Arc::new(ServerState {
            state_rx: rx,
            base_url: "http://localhost".into(),
            feed_permit: Arc::new(Semaphore::new(1)),
        });
        let permit = server.feed_permit.clone().acquire_owned().await.unwrap();
        assert_eq!(
            feed_handler(AxumState(server.clone())).await.status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        drop(permit);
        let mut state = AppState::default();
        state.podcast_name = "Updated".into();
        tx.send(state).unwrap();
        let response = feed_handler(AxumState(server)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 65536).await.unwrap();
        assert!(String::from_utf8(body.to_vec())
            .unwrap()
            .contains("Updated"));
    }

    #[test]
    fn missing_audio_is_excluded_from_the_feed() {
        let root = tempfile::tempdir().unwrap();
        let file = audio_file(
            root.path(),
            "missing",
            "Gone.mp3",
            "Wed, 01 Jul 2026 12:00:00 +0000",
        );
        std::fs::remove_file(&file.temp_path).unwrap();
        let state = AppState {
            files: vec![file],
            ..AppState::default()
        };
        let xml = generate_feed(&state, "http://localhost");
        assert!(rss::Channel::read_from(Cursor::new(xml))
            .unwrap()
            .items()
            .is_empty());
    }
}
