use anyhow::Context;
use chrono::Utc;
use std::net::SocketAddr;
use winusage_core::{api::ApiState, build_snapshot, scan_all, Config, PricingTable};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load().context("failed to load config")?;
    let pricing = PricingTable::embedded_default().context("failed to load pricing table")?;

    let addr: SocketAddr = "127.0.0.1:7337".parse().expect("static addr is valid");

    // Initial scan to populate the snapshot before the server accepts requests.
    let scan = scan_all(&config.claude_projects_dir);
    for err in &scan.errors {
        eprintln!("warning: {err}");
    }
    let snapshot = build_snapshot(&scan.events, &pricing, Utc::now());

    let state = ApiState::new(snapshot, config.api_token.clone());
    let app = winusage_core::api::router(state);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    eprintln!("winusage-api listening on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    eprintln!("\nshutting down");
}
