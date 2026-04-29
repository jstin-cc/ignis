use anyhow::Context;
use chrono::Utc;
use ignis_core::{
    api::ApiState, build_snapshot, scan_all, scan_incremental, Config, FilePosition, PricingTable,
    UsageEvent,
};
use notify::{RecursiveMode, Watcher};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::Duration;

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
    let mut events: Vec<UsageEvent> = scan.events;
    let mut positions: Vec<FilePosition> = scan.positions;
    let mut seen_uuids: HashSet<String> = events.iter().map(|e| e.uuid.clone()).collect();

    let snapshot = build_snapshot(&events, &pricing, Utc::now());

    let state = ApiState::new(
        snapshot,
        config.api_token.clone(),
        config.plan.token_limit(),
        config.allowed_origins.clone(),
    );

    // Background re-scan: react to file changes (notify) + 30-second periodic fallback.
    // Uses scan_incremental so unchanged files are not re-read (ADR-011). Returned
    // events are deduplicated by UUID to keep rotated files from double-counting.
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
    let scan_interval = Duration::from_secs(config.scan_interval_secs.max(5).into());
    tokio::spawn(async move {
        let _watcher = watcher_opt; // keep alive so watch stays registered
        let _tx = notify_tx; // keep channel open for periodic-only mode

        let mut tick = tokio::time::interval(scan_interval);
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
            let positions_snapshot = positions.clone();
            let scan = match tokio::task::spawn_blocking(move || {
                scan_incremental(&dir, &positions_snapshot)
            })
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("scanner task panicked: {e}");
                    continue; // keep last good snapshot, retry on next tick
                }
            };
            for err in &scan.errors {
                eprintln!("warning: {err}");
            }

            let mut added = 0usize;
            for ev in scan.events {
                if seen_uuids.insert(ev.uuid.clone()) {
                    events.push(ev);
                    added += 1;
                }
            }
            positions = scan.positions;

            if added == 0 && scan.errors.is_empty() {
                // Nothing changed — keep current snapshot, just refresh the plan
                // limit in case the user mutated it.
                if let Ok(cfg) = Config::load() {
                    state_bg.update_plan_token_limit(cfg.plan.token_limit());
                }
                continue;
            }

            let snap = build_snapshot(&events, &pricing_bg, Utc::now());
            state_bg.update_snapshot(snap);
            if let Ok(cfg) = Config::load() {
                state_bg.update_plan_token_limit(cfg.plan.token_limit());
            }
        }
    });

    let app = ignis_core::api::router(state);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    eprintln!("ignis-api listening on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}

async fn shutdown_signal() {
    if let Err(e) = tokio::signal::ctrl_c().await {
        eprintln!("Ctrl+C handler error: {e}");
    }
    eprintln!("\nshutting down");
}
