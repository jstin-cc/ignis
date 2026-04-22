use anyhow::Context;
use chrono::Utc;
use notify::{RecursiveMode, Watcher};
use std::net::SocketAddr;
use std::time::Duration;
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

    let state = ApiState::new(
        snapshot,
        config.api_token.clone(),
        config.plan.token_limit(),
    );

    // Background re-scan: react to file changes (notify) + 30-second periodic fallback.
    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let watcher_opt: Option<notify::RecommendedWatcher> = {
        let tx = notify_tx.clone();
        match notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if res.is_ok() {
                let _ = tx.send(());
            }
        }) {
            Ok(mut w) => {
                if let Err(e) = w.watch(&config.claude_projects_dir, RecursiveMode::Recursive) {
                    eprintln!("warning: file watcher unavailable: {e}");
                }
                Some(w)
            }
            Err(e) => {
                eprintln!("warning: failed to create file watcher: {e}");
                None
            }
        }
    };

    let state_bg = state.clone();
    let pricing_bg = pricing;
    let dir_bg = config.claude_projects_dir.clone();
    tokio::spawn(async move {
        let _watcher = watcher_opt; // keep alive so watch stays registered
        let _tx = notify_tx; // keep channel open for periodic-only mode

        let mut tick = tokio::time::interval(Duration::from_secs(30));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        tick.tick().await; // skip first fire — boot scan already done above

        loop {
            tokio::select! {
                _ = tick.tick() => {}
                _ = notify_rx.recv() => {
                    // drain any extra events that arrived while we were scanning
                    while notify_rx.try_recv().is_ok() {}
                }
            }
            let dir = dir_bg.clone();
            let scan = tokio::task::spawn_blocking(move || scan_all(&dir))
                .await
                .unwrap_or_default();
            for err in &scan.errors {
                eprintln!("warning: {err}");
            }
            let snap = build_snapshot(&scan.events, &pricing_bg, Utc::now());
            state_bg.update_snapshot(snap);
            if let Ok(cfg) = Config::load() {
                state_bg.update_plan_token_limit(cfg.plan.token_limit());
            }
        }
    });

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
