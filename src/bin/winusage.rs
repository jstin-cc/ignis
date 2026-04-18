use anyhow::Context;
use chrono::Utc;
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;
use winusage_core::{build_snapshot, scan_all, Config, ModelUsage, PricingTable, Summary};

#[derive(Parser)]
#[command(name = "winusage", about = "Claude Code usage tracker", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show today's usage summary by model.
    Daily,
    /// Show this month's usage summary by model.
    Monthly,
    /// Show the currently active session, if any.
    Session,
    /// Full scan — dump a JSON summary to stdout (dev tool).
    Scan,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load().context("failed to load config")?;
    let pricing = PricingTable::embedded_default().context("failed to load pricing table")?;

    let scan = scan_all(&config.claude_projects_dir);
    for err in &scan.errors {
        eprintln!("warning: {err}");
    }

    let snap = build_snapshot(&scan.events, &pricing, Utc::now());

    match cli.command {
        Command::Daily => print_summary("Today", &snap.today),
        Command::Monthly => print_summary("This Month", &snap.this_month),
        Command::Session => print_session(&snap),
        Command::Scan => print_scan_json(&scan, &snap)?,
    }

    Ok(())
}

// ── Formatters ────────────────────────────────────────────────────────────────

fn print_summary(label: &str, summary: &Summary) {
    const COL_MODEL: usize = 28;
    const COL_INPUT: usize = 12;
    const COL_OUTPUT: usize = 12;
    const COL_COST: usize = 10;
    const TOTAL_WIDTH: usize = COL_MODEL + COL_INPUT + COL_OUTPUT + COL_COST;

    println!("{label}");
    println!(
        "{:<COL_MODEL$}{:>COL_INPUT$}{:>COL_OUTPUT$}{:>COL_COST$}",
        "Model", "Input", "Output", "Cost"
    );
    println!("{}", "─".repeat(TOTAL_WIDTH));

    if summary.by_model.is_empty() {
        println!("  (no events)");
    } else {
        for (model_id, usage) in &summary.by_model {
            println!(
                "{:<COL_MODEL$}{:>COL_INPUT$}{:>COL_OUTPUT$}{:>COL_COST$}",
                truncate(&model_id.0, COL_MODEL - 1),
                format_tokens(usage.input_tokens),
                format_tokens(usage.output_tokens),
                format_cost(usage.cost_usd),
            );
        }
    }

    println!("{}", "─".repeat(TOTAL_WIDTH));
    println!(
        "{:<COL_MODEL$}{:>COL_INPUT$}{:>COL_OUTPUT$}{:>COL_COST$}",
        "Total",
        format_tokens(summary_input_total(summary)),
        format_tokens(summary_output_total(summary)),
        format_cost(summary.total_cost_usd),
    );
}

fn print_session(snap: &winusage_core::Snapshot) {
    match &snap.active_session {
        None => println!("no active session"),
        Some(s) => {
            let project = s.project_path.display();
            let branch = s
                .git_branch
                .as_deref()
                .map(|b| format!(" [{b}]"))
                .unwrap_or_default();
            println!("Active session: {}", s.session_id);
            println!("  Project : {project}{branch}");
            println!("  Events  : {}", s.event_count);
            println!("  Cost    : {}", format_cost(s.total_cost_usd));
            println!(
                "  Last    : {}",
                s.last_seen.format("%Y-%m-%d %H:%M:%S UTC")
            );
        }
    }
}

fn print_scan_json(
    scan: &winusage_core::ScanResult,
    snap: &winusage_core::Snapshot,
) -> anyhow::Result<()> {
    let output = serde_json::json!({
        "scanned_files":       scan.positions.len(),
        "total_events":        scan.events.len(),
        "scan_errors":         scan.errors.len(),
        "today_cost_usd":      snap.today.total_cost_usd.to_string(),
        "this_week_cost_usd":  snap.this_week.total_cost_usd.to_string(),
        "this_month_cost_usd": snap.this_month.total_cost_usd.to_string(),
        "active_session":      snap.active_session.as_ref().map(|s| &s.session_id),
        "session_count":       snap.sessions.len(),
        "pricing_warnings":    snap.pricing_warnings.iter().map(|m| &m.0).collect::<Vec<_>>(),
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn format_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_cost(d: Decimal) -> String {
    format!("${d:.2}")
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

fn summary_input_total(summary: &Summary) -> u64 {
    summary
        .by_model
        .values()
        .map(|u: &ModelUsage| u.input_tokens)
        .sum()
}

fn summary_output_total(summary: &Summary) -> u64 {
    summary
        .by_model
        .values()
        .map(|u: &ModelUsage| u.output_tokens)
        .sum()
}
