use anyhow::Context;
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};
use ignis_core::{build_snapshot, scan_all, Config, ModelUsage, PricingTable, Summary};
use rust_decimal::Decimal;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "ignis", about = "Claude Code usage tracker", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Copy, Clone, ValueEnum)]
enum ExportFormat {
    Json,
    Csv,
}

#[derive(Copy, Clone, ValueEnum)]
enum ExportPeriod {
    Today,
    Week,
    Month,
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
    /// Export usage data as JSON or CSV.
    Export {
        /// Output format.
        #[arg(short, long, value_enum, default_value = "json")]
        format: ExportFormat,
        /// Time period to export.
        #[arg(short, long, value_enum, default_value = "month")]
        period: ExportPeriod,
        /// Write output to FILE instead of stdout.
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
        /// Overwrite output file if it already exists.
        #[arg(long)]
        force: bool,
    },
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
        Command::Export {
            format,
            period,
            output,
            force,
        } => {
            let (label, summary) = match period {
                ExportPeriod::Today => ("today", &snap.today),
                ExportPeriod::Week => ("week", &snap.this_week),
                ExportPeriod::Month => ("month", &snap.this_month),
            };
            match format {
                ExportFormat::Json => export_json(label, summary, output.as_deref(), force)?,
                ExportFormat::Csv => export_csv(label, summary, output.as_deref(), force)?,
            }
        }
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

fn print_session(snap: &ignis_core::Snapshot) {
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
    scan: &ignis_core::ScanResult,
    snap: &ignis_core::Snapshot,
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
    match s.char_indices().nth(max) {
        Some((i, _)) => &s[..i],
        None => s,
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

/// Writes `content` to `path` atomically (via `.ignis_tmp` + rename), or to stdout if
/// `path` is `None`. Errors if `path` already exists and `force` is false.
fn write_output(path: Option<&Path>, force: bool, content: &[u8]) -> anyhow::Result<()> {
    let Some(p) = path else {
        io::stdout().write_all(content)?;
        return Ok(());
    };

    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            anyhow::bail!("directory '{}' does not exist", parent.display());
        }
    }

    if !force && p.exists() {
        anyhow::bail!(
            "file '{}' already exists — use --force to overwrite",
            p.display()
        );
    }

    let tmp = p.with_extension("ignis_tmp");
    std::fs::write(&tmp, content)
        .with_context(|| format!("failed to write '{}'", tmp.display()))?;
    std::fs::rename(&tmp, p).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        anyhow::anyhow!(
            "failed to rename '{}' -> '{}': {e}",
            tmp.display(),
            p.display()
        )
    })?;
    Ok(())
}

fn export_json(
    period: &str,
    summary: &Summary,
    output: Option<&Path>,
    force: bool,
) -> anyhow::Result<()> {
    let by_model: Vec<_> = summary
        .by_model
        .iter()
        .map(|(id, u)| {
            serde_json::json!({
                "model":         id.0,
                "input_tokens":  u.input_tokens,
                "output_tokens": u.output_tokens,
                "cost_usd":      u.cost_usd.to_string(),
            })
        })
        .collect();

    let by_project: Vec<_> = summary
        .by_project
        .iter()
        .map(|(path, p)| {
            serde_json::json!({
                "project":      path.display().to_string(),
                "total_tokens": p.total_tokens,
                "cost_usd":     p.total_cost_usd.to_string(),
            })
        })
        .collect();

    let out = serde_json::json!({
        "period":          period,
        "total_cost_usd":  summary.total_cost_usd.to_string(),
        "total_tokens":    summary.total_tokens,
        "event_count":     summary.event_count,
        "by_model":        by_model,
        "by_project":      by_project,
    });
    let content = format!("{}\n", serde_json::to_string_pretty(&out)?);
    write_output(output, force, content.as_bytes())
}

fn export_csv(
    period: &str,
    summary: &Summary,
    output: Option<&Path>,
    force: bool,
) -> anyhow::Result<()> {
    let mut buf = String::from("period,model,input_tokens,output_tokens,cost_usd\n");
    let mut rows: Vec<_> = summary.by_model.iter().collect();
    rows.sort_by_key(|(id, _)| id.0.as_str());
    for (id, u) in rows {
        buf.push_str(&format!(
            "{},{},{},{},{}\n",
            period, id.0, u.input_tokens, u.output_tokens, u.cost_usd
        ));
    }
    write_output(output, force, buf.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ignis_core::Summary;

    fn empty_summary() -> Summary {
        Summary::default()
    }

    #[test]
    fn export_json_creates_new_file() {
        let path = std::env::temp_dir().join("ignis_test_export_new.json");
        let _ = std::fs::remove_file(&path);

        export_json("today", &empty_summary(), Some(&path), false).unwrap();

        assert!(path.exists());
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"period\""));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn export_json_refuses_overwrite_without_force() {
        let path = std::env::temp_dir().join("ignis_test_export_guard.json");
        std::fs::write(&path, b"original").unwrap();

        let err = export_json("today", &empty_summary(), Some(&path), false).unwrap_err();
        assert!(err.to_string().contains("already exists"));
        assert_eq!(std::fs::read(&path).unwrap(), b"original");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn export_json_force_overwrites_existing_file() {
        let path = std::env::temp_dir().join("ignis_test_export_force.json");
        std::fs::write(&path, b"old").unwrap();

        export_json("today", &empty_summary(), Some(&path), true).unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"period\""));
        assert!(!text.contains("old"));
        let _ = std::fs::remove_file(&path);
    }
}
