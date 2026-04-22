use anyhow::Context;
use chrono::Utc;
use ignis_core::{build_snapshot, scan_all, Config, PricingTable};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let config = Config::load().context("failed to load config")?;
    let pricing = PricingTable::embedded_default().context("failed to load pricing table")?;

    eprintln!("scanning: {}", config.claude_projects_dir.display());

    let scan = scan_all(&config.claude_projects_dir);

    for err in &scan.errors {
        eprintln!("warning: {err}");
    }

    let snap = build_snapshot(&scan.events, &pricing, Utc::now());

    let output = json!({
        "scanned_files":      scan.positions.len(),
        "total_events":       scan.events.len(),
        "scan_errors":        scan.errors.len(),
        "today_cost_usd":     snap.today.total_cost_usd.to_string(),
        "this_week_cost_usd": snap.this_week.total_cost_usd.to_string(),
        "this_month_cost_usd":snap.this_month.total_cost_usd.to_string(),
        "active_session":     snap.active_session.as_ref().map(|s| &s.session_id),
        "session_count":      snap.sessions.len(),
        "pricing_warnings":   snap.pricing_warnings.iter().map(|m| &m.0).collect::<Vec<_>>(),
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
