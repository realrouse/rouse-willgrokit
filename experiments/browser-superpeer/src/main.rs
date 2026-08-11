//! Browser superpeer experiment: LBRY-shaped blobs over Iroh.
//!
//! Commands:
//!   pack        — build a demo blob directory from a media file
//!   superpeer   — serve blobs from a directory over Iroh
//!   fetch       — CLI assemble stream via Iroh (Slice B)
//!   companion   — localhost HTTP bridge + static web UI (Slice C)

mod lbry_blob;
mod protocol;
mod ticket;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use clap::{Parser, Subcommand};
use iroh::{Endpoint, EndpointAddr, endpoint::presets};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::{error, info};

use crate::lbry_blob::{load_blob_file, pack_file, parse_sd_blob, verify_blob_hash};
use crate::protocol::{client_get_blob, client_have, serve_one, ALPN};
use crate::ticket::{decode_ticket, encode_ticket};

#[derive(Parser, Debug)]
#[command(name = "browser_superpeer", about = "LBRY blob download superpeer over Iroh (MVP)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Pack a file into LBRY-shaped encrypted blobs + sd blob.
    Pack {
        /// Input media/file path.
        #[arg(long)]
        input: PathBuf,
        /// Output blob directory.
        #[arg(long, default_value = "fixtures/demo")]
        out: PathBuf,
    },
    /// Run download superpeer (serves existing blobs; no upload reflector).
    Superpeer {
        /// Directory of blob files named by SHA-384 hex.
        #[arg(long, default_value = "fixtures/demo")]
        blobs: PathBuf,
    },
    /// Fetch and assemble a stream over Iroh to an output file.
    Fetch {
        #[arg(long)]
        ticket: String,
        #[arg(long)]
        sd_hash: String,
        /// Override stream key hex (defaults to key inside sd for demo packs).
        #[arg(long)]
        key: Option<String>,
        #[arg(long, default_value = "assembled.bin")]
        out: PathBuf,
    },
    /// Localhost companion: Iroh client + HTTP API + web UI.
    Companion {
        #[arg(long, default_value = "127.0.0.1:8787")]
        listen: SocketAddr,
        /// Optional directory of static web assets (defaults to ./web).
        #[arg(long, default_value = "web")]
        web: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Pack { input, out } => cmd_pack(input, out).await,
        Cmd::Superpeer { blobs } => cmd_superpeer(blobs).await,
        Cmd::Fetch {
            ticket,
            sd_hash,
            key,
            out,
        } => cmd_fetch(ticket, sd_hash, key, out).await,
        Cmd::Companion { listen, web } => cmd_companion(listen, web).await,
    }
}

async fn cmd_pack(input: PathBuf, out: PathBuf) -> Result<()> {
    let packed = pack_file(&input, &out)?;
    println!("Packed OK");
    println!("  blob_dir   = {}", packed.blob_dir.display());
    println!("  sd_hash    = {}", packed.sd_hash);
    println!("  stream_key = {}", packed.stream_key_hex);
    println!("  filename   = {}", packed.filename);
    println!("  meta       = {}/DEMO.json", packed.blob_dir.display());
    Ok(())
}

async fn cmd_superpeer(blobs: PathBuf) -> Result<()> {
    if !blobs.is_dir() {
        bail!("blob directory does not exist: {}", blobs.display());
    }
    let blobs = Arc::new(blobs.canonicalize().context("canonicalize blobs dir")?);

    let endpoint = Endpoint::builder(presets::N0)
        .alpns(vec![ALPN.to_vec()])
        .bind()
        .await
        .map_err(|e| anyhow!("endpoint bind: {e}"))?;

    endpoint.online().await;
    let addr = endpoint.addr();
    let t = encode_ticket(&addr)?;
    println!("========================================");
    println!("LBRY download superpeer (Iroh)");
    println!("  blobs   = {}", blobs.display());
    println!("  endpoint_id = {}", addr.id);
    println!("  ticket  = {t}");
    println!("========================================");
    println!("Paste the ticket into the companion web UI or `fetch` command.");
    println!("Ctrl+C to stop.");

    let ep = endpoint.clone();
    let accept = tokio::spawn(async move {
        while let Some(incoming) = ep.accept().await {
            let blobs = Arc::clone(&blobs);
            tokio::spawn(async move {
                if let Err(e) = handle_incoming(incoming, blobs).await {
                    error!("connection error: {e:#}");
                }
            });
        }
    });

    tokio::signal::ctrl_c().await.ok();
    info!("shutting down superpeer");
    endpoint.close().await;
    accept.abort();
    Ok(())
}

async fn handle_incoming(
    incoming: iroh::endpoint::Incoming,
    blobs: Arc<PathBuf>,
) -> Result<()> {
    let conn = incoming.await.map_err(|e| anyhow!("accept: {e}"))?;
    let remote = conn.remote_id();
    info!("accepted connection from {remote}");
    loop {
        match conn.accept_bi().await {
            Ok((mut send, mut recv)) => {
                let blobs = Arc::clone(&blobs);
                let res = serve_one(&mut send, &mut recv, |hash| {
                    match load_blob_file(blobs.as_path(), hash) {
                        Ok(data) => Ok(Some(data)),
                        Err(_) => {
                            // Try without re-verify failure paths: missing file
                            let p = blobs.join(hash.to_lowercase());
                            if p.exists() {
                                let data = std::fs::read(&p)?;
                                Ok(Some(data))
                            } else if blobs.join(hash).exists() {
                                let data = std::fs::read(blobs.join(hash))?;
                                Ok(Some(data))
                            } else {
                                Ok(None)
                            }
                        }
                    }
                })
                .await;
                if let Err(e) = res {
                    error!("request error: {e:#}");
                }
            }
            Err(e) => {
                // Connection closed.
                info!("connection closed ({remote}): {e}");
                break;
            }
        }
    }
    Ok(())
}

async fn connect_to_ticket(ticket: &str) -> Result<(Endpoint, iroh::endpoint::Connection)> {
    let addr: EndpointAddr = decode_ticket(ticket)?;
    let endpoint = Endpoint::bind(presets::N0)
        .await
        .map_err(|e| anyhow!("client bind: {e}"))?;
    let conn = endpoint
        .connect(addr, ALPN)
        .await
        .map_err(|e| anyhow!("connect: {e}"))?;
    Ok((endpoint, conn))
}

async fn cmd_fetch(
    ticket: String,
    sd_hash: String,
    key_override: Option<String>,
    out: PathBuf,
) -> Result<()> {
    let (endpoint, conn) = connect_to_ticket(&ticket).await?;
    info!("connected; fetching sd_hash={sd_hash}");

    let sd_raw = client_get_blob(&conn, &sd_hash).await?;
    verify_blob_hash(&sd_raw, &sd_hash)?;
    let mut sd = parse_sd_blob(&sd_raw)?;
    if let Some(k) = key_override {
        sd.key = k;
    }

    let mut out_bytes = Vec::new();
    for entry in &sd.blobs {
        let raw = client_get_blob(&conn, &entry.blob_hash).await?;
        verify_blob_hash(&raw, &entry.blob_hash)?;
        let plain = lbry_blob::decrypt_content_blob(&raw, &sd.key, &entry.iv)?;
        out_bytes.extend_from_slice(&plain);
        info!(
            "blob {} ok ({} ciphertext bytes)",
            &entry.blob_hash[..16.min(entry.blob_hash.len())],
            raw.len()
        );
    }

    std::fs::write(&out, &out_bytes)?;
    println!(
        "Wrote {} bytes to {} ({} content blobs)",
        out_bytes.len(),
        out.display(),
        sd.blobs.len()
    );

    conn.close(0u32.into(), b"done");
    endpoint.close().await;
    Ok(())
}

// --- Companion HTTP ---

#[derive(Clone)]
struct AppState {
    // Keep one endpoint; reconnect per job for simplicity in MVP.
    inner: Arc<Mutex<()>>,
}

#[derive(Deserialize)]
struct PlayBody {
    ticket: String,
    sd_hash: String,
    #[serde(default)]
    key: Option<String>,
}

#[derive(Serialize)]
struct PlayResponse {
    ok: bool,
    filename: String,
    bytes: usize,
    content_blobs: usize,
    media_path: String,
    message: String,
}

#[derive(Deserialize)]
struct HaveQuery {
    ticket: String,
    hash: String,
}

async fn cmd_companion(listen: SocketAddr, web: PathBuf) -> Result<()> {
    let state = AppState {
        inner: Arc::new(Mutex::new(())),
    };

    // Ensure media cache dir
    std::fs::create_dir_all("cache")?;

    let app = Router::new()
        .route("/", get(index_fallback))
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/have", get(api_have))
        .route("/api/play", post(api_play))
        .route("/media/{name}", get(serve_media))
        .nest_service("/static", ServeDir::new(&web))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Prefer index from web/
    let index_path = web.join("index.html");
    if index_path.exists() {
        info!("web UI: {}", index_path.display());
    } else {
        info!("web/index.html missing; using embedded fallback HTML");
    }

    println!("Companion listening on http://{listen}");
    println!("Open that URL in a browser. Superpeer ticket + sd_hash go in the form.");

    let listener = tokio::net::TcpListener::bind(listen).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index_fallback() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}

async fn api_have(
    State(_st): State<AppState>,
    Query(q): Query<HaveQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (endpoint, conn) = connect_to_ticket(&q.ticket)
        .await
        .map_err(AppError::from)?;
    let have = client_have(&conn, &q.hash).await.map_err(AppError::from)?;
    conn.close(0u32.into(), b"bye");
    endpoint.close().await;
    Ok(Json(serde_json::json!({ "have": have, "hash": q.hash })))
}

async fn api_play(
    State(st): State<AppState>,
    Json(body): Json<PlayBody>,
) -> Result<Json<PlayResponse>, AppError> {
    let _guard = st.inner.lock().await;
    let (endpoint, conn) = connect_to_ticket(&body.ticket)
        .await
        .map_err(AppError::from)?;

    let sd_raw = client_get_blob(&conn, &body.sd_hash)
        .await
        .map_err(AppError::from)?;
    verify_blob_hash(&sd_raw, &body.sd_hash).map_err(AppError::from)?;
    let mut sd = parse_sd_blob(&sd_raw).map_err(AppError::from)?;
    if let Some(k) = body.key.clone() {
        if !k.is_empty() {
            sd.key = k;
        }
    }

    let mut out_bytes = Vec::new();
    for entry in &sd.blobs {
        let raw = client_get_blob(&conn, &entry.blob_hash)
            .await
            .map_err(AppError::from)?;
        verify_blob_hash(&raw, &entry.blob_hash).map_err(AppError::from)?;
        let plain = lbry_blob::decrypt_content_blob(&raw, &sd.key, &entry.iv)
            .map_err(AppError::from)?;
        out_bytes.extend_from_slice(&plain);
    }

    let filename = {
        let raw = hex::decode(&sd.filename).unwrap_or_else(|_| b"assembled.bin".to_vec());
        String::from_utf8_lossy(&raw).to_string()
    };
    let safe_name = format!(
        "{}_{}",
        &body.sd_hash[..16.min(body.sd_hash.len())],
        filename.replace(['/', '\\'], "_")
    );
    let path = PathBuf::from("cache").join(&safe_name);
    std::fs::write(&path, &out_bytes).map_err(|e| AppError::from(anyhow!(e)))?;

    conn.close(0u32.into(), b"done");
    endpoint.close().await;

    Ok(Json(PlayResponse {
        ok: true,
        filename,
        bytes: out_bytes.len(),
        content_blobs: sd.blobs.len(),
        media_path: format!("/media/{safe_name}"),
        message: format!(
            "Verified {} LBRY-shaped blobs over Iroh (sd_hash matches).",
            sd.blobs.len()
        ),
    }))
}

async fn serve_media(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Response, AppError> {
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(AppError::from(anyhow!("bad name")));
    }
    let path = PathBuf::from("cache").join(&name);
    let data = std::fs::read(&path).map_err(|e| AppError::from(anyhow!("media: {e}")))?;
    let ctype = if name.ends_with(".wav") {
        "audio/wav"
    } else if name.ends_with(".mp4") {
        "video/mp4"
    } else if name.ends_with(".webm") {
        "video/webm"
    } else if name.ends_with(".mp3") {
        "audio/mpeg"
    } else {
        "application/octet-stream"
    };
    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, ctype)],
        data,
    )
        .into_response())
}

struct AppError(anyhow::Error);

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("api error: {:#}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "ok": false,
                "error": format!("{:#}", self.0),
            })),
        )
            .into_response()
    }
}

