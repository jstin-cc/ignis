use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::Context;
use chrono::{Local, Utc};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use notify::{RecursiveMode, Watcher};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use rust_decimal::Decimal;
use winusage_core::{build_snapshot, scan_all, Config, PricingTable, Snapshot, Summary};

// ── Palette ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct Palette {
    bg: Color,
    panel: Color,
    border: Color,
    text: Color,
    dim: Color,
    muted: Color,
    accent: Color,
    warning: Color,
}

fn detect_palette() -> Palette {
    if std::env::var("NO_COLOR").is_ok() {
        Palette {
            bg: Color::Reset,
            panel: Color::Reset,
            border: Color::DarkGray,
            text: Color::White,
            dim: Color::Gray,
            muted: Color::DarkGray,
            accent: Color::Yellow,
            warning: Color::Yellow,
        }
    } else {
        Palette {
            bg: Color::Rgb(0x1F, 0x1E, 0x1B),
            panel: Color::Rgb(0x29, 0x27, 0x24),
            border: Color::Rgb(0x3D, 0x3A, 0x34),
            text: Color::Rgb(0xF4, 0xF3, 0xEE),
            dim: Color::Rgb(0xB1, 0xAD, 0xA1),
            muted: Color::Rgb(0x7A, 0x76, 0x6D),
            accent: Color::Rgb(0xC1, 0x5F, 0x3C),
            warning: Color::Rgb(0xD4, 0xA5, 0x74),
        }
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum View {
    Daily,
    Monthly,
}

struct App {
    snap: Snapshot,
    config: Config,
    pricing: PricingTable,
    view: View,
    error_count: usize,
    pal: Palette,
}

impl App {
    fn load(config: Config, pricing: PricingTable, pal: Palette) -> Self {
        let scan = scan_all(&config.claude_projects_dir);
        let snap = build_snapshot(&scan.events, &pricing, Utc::now());
        App {
            error_count: scan.errors.len(),
            snap,
            config,
            pricing,
            view: View::Daily,
            pal,
        }
    }

    fn refresh(&mut self) {
        let scan = scan_all(&self.config.claude_projects_dir);
        self.error_count = scan.errors.len();
        self.snap = build_snapshot(&scan.events, &self.pricing, Utc::now());
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    let config = Config::load().context("failed to load config")?;
    let pricing = PricingTable::embedded_default().context("failed to load pricing")?;
    let pal = detect_palette();

    let (tx, rx) = mpsc::channel::<()>();

    let watch_dir = config.claude_projects_dir.clone();
    let tx_notify = tx.clone();
    let mut watcher = notify::recommended_watcher(move |_| {
        let _ = tx_notify.send(());
    })?;
    if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::Recursive) {
        eprintln!("warning: file watcher unavailable: {e}");
    }
    let _watcher = watcher;

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(5));
        if tx.send(()).is_err() {
            break;
        }
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::load(config, pricing, pal);
    let result = run(&mut terminal, &mut app, rx);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    rx: mpsc::Receiver<()>,
) -> anyhow::Result<()> {
    loop {
        if rx.try_recv().is_ok() {
            while rx.try_recv().is_ok() {}
            app.refresh();
        }
        terminal.draw(|f| draw(f, app))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    KeyCode::Char('r') => app.refresh(),
                    KeyCode::Char('d') => app.view = View::Daily,
                    KeyCode::Char('m') => app.view = View::Monthly,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// ── Drawing ───────────────────────────────────────────────────────────────────

fn draw(f: &mut Frame, app: &App) {
    let p = app.pal;
    let area = f.area();
    f.render_widget(Block::default().style(Style::default().bg(p.bg)), area);

    let (summary, view_label) = match app.view {
        View::Daily => (&app.snap.today, "TODAY"),
        View::Monthly => (&app.snap.this_month, "THIS MONTH"),
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(7),
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    draw_header(f, chunks[0], app, view_label);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);
    draw_summary_panel(f, top[0], app, summary, view_label);
    draw_session_panel(f, top[1], app);

    draw_model_table(f, chunks[2], app, summary);
    draw_burn_rate(f, chunks[3], app);
    draw_footer(f, chunks[4], app);
}

fn draw_header(f: &mut Frame, area: Rect, app: &App, view_label: &str) {
    let p = app.pal;
    let now = Local::now().format("%H:%M:%S").to_string();
    let title = format!(" WinUsage watch  [{view_label}]");
    let right = format!("{now} ");
    let pad = area
        .width
        .saturating_sub((title.len() + right.len()) as u16) as usize;
    let line = Line::from(vec![
        Span::styled(
            title,
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ".repeat(pad), Style::default().fg(p.muted)),
        Span::styled(right, Style::default().fg(p.dim)),
    ]);
    f.render_widget(Paragraph::new(line).style(Style::default().bg(p.bg)), area);
}

fn draw_summary_panel(f: &mut Frame, area: Rect, app: &App, summary: &Summary, label: &str) {
    let p = app.pal;
    let block = styled_block(label, p);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let tokens: u64 = summary
        .by_model
        .values()
        .map(|u| u.input_tokens + u.output_tokens)
        .sum();
    let lines = vec![
        Line::from(Span::styled(
            format!("  {}", fmt_cost(summary.total_cost_usd)),
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!(
                "  {}  ·  {} events",
                fmt_tokens(tokens),
                summary.event_count
            ),
            Style::default().fg(p.dim),
        )),
    ];
    f.render_widget(
        Paragraph::new(lines).style(Style::default().bg(p.panel)),
        inner,
    );
}

fn draw_session_panel(f: &mut Frame, area: Rect, app: &App) {
    let p = app.pal;
    let block = styled_block("ACTIVE SESSION", p);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = match &app.snap.active_session {
        None => vec![Line::from(Span::styled(
            "  no active session",
            Style::default().fg(p.muted),
        ))],
        Some(s) => {
            let name = s
                .project_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let branch_span = match s.git_branch.as_deref() {
                Some(b) => Span::styled(format!(" [{b}]"), Style::default().fg(p.muted)),
                None => Span::raw(""),
            };
            let elapsed = (Utc::now() - s.first_seen).num_seconds().max(0);
            let tokens: u64 = s
                .by_model
                .values()
                .map(|u| u.input_tokens + u.output_tokens)
                .sum();
            vec![
                Line::from(vec![
                    Span::styled(format!("  {name}"), Style::default().fg(p.text)),
                    branch_span,
                    Span::styled(
                        format!("  {}", fmt_duration(elapsed)),
                        Style::default().fg(p.accent),
                    ),
                ]),
                Line::from(Span::styled(
                    format!(
                        "  {}  ·  {}",
                        fmt_tokens(tokens),
                        fmt_cost(s.total_cost_usd)
                    ),
                    Style::default().fg(p.dim),
                )),
            ]
        }
    };
    f.render_widget(
        Paragraph::new(lines).style(Style::default().bg(p.panel)),
        inner,
    );
}

fn draw_model_table(f: &mut Frame, area: Rect, app: &App, summary: &Summary) {
    let p = app.pal;
    let block = styled_block("BY MODEL", p);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines: Vec<Line> = if summary.by_model.is_empty() {
        vec![Line::from(Span::styled(
            "  (no events)",
            Style::default().fg(p.muted),
        ))]
    } else {
        summary
            .by_model
            .iter()
            .map(|(model, usage)| {
                let total = usage.input_tokens + usage.output_tokens;
                Line::from(vec![
                    Span::styled(
                        format!("  {:<36}", truncate(&model.0, 35)),
                        Style::default().fg(p.text),
                    ),
                    Span::styled(
                        format!("{:>8}", fmt_cost(usage.cost_usd)),
                        Style::default().fg(p.accent),
                    ),
                    Span::styled(
                        format!("  {:>9} tokens", fmt_tokens(total)),
                        Style::default().fg(p.dim),
                    ),
                ])
            })
            .collect()
    };
    f.render_widget(
        Paragraph::new(lines).style(Style::default().bg(p.panel)),
        inner,
    );
}

fn draw_burn_rate(f: &mut Frame, area: Rect, app: &App) {
    let p = app.pal;
    let block = styled_block("BURN RATE", p);
    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "  live burn rate — coming in v0.2",
            Style::default().fg(p.muted),
        )))
        .style(Style::default().bg(p.panel)),
        inner,
    );
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let p = app.pal;
    let mut spans = vec![Span::raw(" ")];
    for (key, desc) in &[
        ("[q]", " quit  "),
        ("[r]", " refresh  "),
        ("[d]", " daily  "),
        ("[m]", " monthly"),
    ] {
        spans.push(Span::styled(*key, Style::default().fg(p.accent)));
        spans.push(Span::styled(*desc, Style::default().fg(p.muted)));
    }
    if app.error_count > 0 {
        spans.push(Span::styled(
            format!("  {} scan warning(s)", app.error_count),
            Style::default().fg(p.warning),
        ));
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(p.bg)),
        area,
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn styled_block(title: &str, p: Palette) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .title_style(Style::default().fg(p.dim).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(p.border))
        .style(Style::default().bg(p.panel))
}

fn fmt_cost(d: Decimal) -> String {
    format!("${d:.2}")
}

fn fmt_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn fmt_duration(secs: i64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
