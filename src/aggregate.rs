use crate::model::{
    BurnRateBucket, HeatmapDay, HeatmapHourBucket, ModelId, ModelUsage, SessionBlock, SessionState,
    Snapshot, Summary, UsageEvent,
};
use crate::pricing::PricingTable;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Sessions inactive for longer than this are not considered "active".
const ACTIVE_THRESHOLD_SECS: i64 = 300;

/// Duration of one Claude Code billing window.
const BILLING_BLOCK_HOURS: i64 = 5;

/// Number of days covered by the activity heatmap (12 weeks).
const HEATMAP_DAYS: i64 = 84;

/// Build a complete [`Snapshot`] from a flat list of usage events.
///
/// `now` is passed explicitly so callers (and tests) can control the clock.
pub fn build_snapshot(
    events: &[UsageEvent],
    pricing: &PricingTable,
    now: DateTime<Utc>,
) -> Snapshot {
    let windows = Windows::for_now(now);
    let mut sessions: HashMap<String, SessionState> = HashMap::new();
    let mut today = Summary::default();
    let mut this_week = Summary::default();
    let mut this_month = Summary::default();
    let mut last_30_days = Summary::default();
    let mut unpriced: HashSet<ModelId> = HashSet::new();

    // Track unique session IDs per project per window to compute session_count.
    let mut today_proj_sessions: HashMap<std::path::PathBuf, HashSet<String>> = HashMap::new();
    let mut week_proj_sessions: HashMap<std::path::PathBuf, HashSet<String>> = HashMap::new();
    let mut month_proj_sessions: HashMap<std::path::PathBuf, HashSet<String>> = HashMap::new();
    let mut last_30_days_proj_sessions: HashMap<std::path::PathBuf, HashSet<String>> =
        HashMap::new();

    for ev in events {
        let computed = pricing.compute_cost(ev);
        if computed.unpriced {
            unpriced.insert(ev.model.clone());
        }
        let cost = computed.cost_usd;

        let session = sessions
            .entry(ev.session_id.clone())
            .or_insert_with(|| SessionState {
                session_id: ev.session_id.clone(),
                project_path: ev.project_path.clone(),
                git_branch: ev.git_branch.clone(),
                first_seen: ev.timestamp,
                last_seen: ev.timestamp,
                event_count: 0,
                total_cost_usd: Decimal::ZERO,
                by_model: BTreeMap::new(),
            });

        if ev.timestamp < session.first_seen {
            session.first_seen = ev.timestamp;
        }
        if ev.timestamp > session.last_seen {
            session.last_seen = ev.timestamp;
        }
        session.event_count += 1;
        session.total_cost_usd += cost;
        accumulate_model(&mut session.by_model, ev, cost);

        if windows.in_today(ev.timestamp) {
            accumulate_summary(&mut today, ev, cost);
            today_proj_sessions
                .entry(ev.project_path.clone())
                .or_default()
                .insert(ev.session_id.clone());
        }
        if windows.in_week(ev.timestamp) {
            accumulate_summary(&mut this_week, ev, cost);
            week_proj_sessions
                .entry(ev.project_path.clone())
                .or_default()
                .insert(ev.session_id.clone());
        }
        if windows.in_month(ev.timestamp) {
            accumulate_summary(&mut this_month, ev, cost);
            month_proj_sessions
                .entry(ev.project_path.clone())
                .or_default()
                .insert(ev.session_id.clone());
        }
        if windows.in_last_30_days(ev.timestamp) {
            accumulate_summary(&mut last_30_days, ev, cost);
            last_30_days_proj_sessions
                .entry(ev.project_path.clone())
                .or_default()
                .insert(ev.session_id.clone());
        }
    }

    // Back-fill session_count from the tracked sets.
    for (path, ids) in &today_proj_sessions {
        if let Some(proj) = today.by_project.get_mut(path) {
            proj.session_count = ids.len() as u64;
        }
    }
    for (path, ids) in &week_proj_sessions {
        if let Some(proj) = this_week.by_project.get_mut(path) {
            proj.session_count = ids.len() as u64;
        }
    }
    for (path, ids) in &month_proj_sessions {
        if let Some(proj) = this_month.by_project.get_mut(path) {
            proj.session_count = ids.len() as u64;
        }
    }
    for (path, ids) in &last_30_days_proj_sessions {
        if let Some(proj) = last_30_days.by_project.get_mut(path) {
            proj.session_count = ids.len() as u64;
        }
    }

    let mut sessions_vec: Vec<SessionState> = sessions.into_values().collect();
    sessions_vec.sort_unstable_by_key(|s| std::cmp::Reverse(s.last_seen));
    sessions_vec.truncate(500);

    let active_session = sessions_vec
        .iter()
        .find(|s| now.signed_duration_since(s.last_seen).num_seconds() <= ACTIVE_THRESHOLD_SECS)
        .cloned();

    let mut pricing_warnings: Vec<ModelId> = unpriced.into_iter().collect();
    pricing_warnings.sort();

    let blocks = billing_blocks(events, pricing);
    let active_block = active_block_at(&blocks, now);
    let today_local = now.with_timezone(&Local).date_naive();
    let heatmap = daily_costs(events, pricing, today_local);
    let burn_rate = build_burn_rate(events, pricing, now);
    let hourly_heatmap_week = build_hourly_heatmap(events, pricing, now);

    Snapshot {
        taken_at: now,
        today,
        this_week,
        this_month,
        last_30_days,
        active_session,
        sessions: sessions_vec,
        active_block,
        pricing_warnings,
        heatmap,
        burn_rate,
        hourly_heatmap_week,
    }
}

/// Aggregate per-day costs for the 84-day heatmap window ending on `today_local`.
///
/// Returns exactly [`HEATMAP_DAYS`] entries in chronological order, including
/// zero-cost days so the caller can fill the grid without date arithmetic.
pub fn daily_costs(
    events: &[UsageEvent],
    pricing: &PricingTable,
    today_local: NaiveDate,
) -> Vec<HeatmapDay> {
    let start = today_local - Duration::days(HEATMAP_DAYS - 1);
    let mut by_date: BTreeMap<NaiveDate, Decimal> = BTreeMap::new();

    for ev in events {
        let date = ev.timestamp.with_timezone(&Local).date_naive();
        if date < start || date > today_local {
            continue;
        }
        *by_date.entry(date).or_default() += pricing.compute_cost(ev).cost_usd;
    }

    (0..HEATMAP_DAYS)
        .map(|i| {
            let date = start + Duration::days(i);
            HeatmapDay {
                date,
                cost_usd: by_date.get(&date).copied().unwrap_or_default(),
            }
        })
        .collect()
}

/// Builds 30 one-minute buckets covering `(now - 30 min) ..= now` for the burn-rate sparkline.
///
/// Sidechain events are excluded (sub-agent calls are noise for live burn-rate).
/// Each bucket covers exactly one calendar minute. Empty minutes get zeros.
/// Returns buckets sorted ascending by minute_start, oldest first.
pub fn build_burn_rate(
    events: &[UsageEvent],
    pricing: &PricingTable,
    now: DateTime<Utc>,
) -> Vec<BurnRateBucket> {
    const BUCKET_COUNT: i64 = 30;
    let window_start = now - Duration::minutes(BUCKET_COUNT);

    let mut buckets: Vec<BurnRateBucket> = (0..BUCKET_COUNT)
        .map(|i| BurnRateBucket {
            minute_start: window_start + Duration::minutes(i),
            tokens: 0,
            cost_usd: Decimal::ZERO,
        })
        .collect();

    for ev in events {
        if ev.is_sidechain {
            continue;
        }
        if ev.timestamp < window_start || ev.timestamp > now {
            continue;
        }
        let idx = (ev.timestamp - window_start).num_minutes();
        if !(0..BUCKET_COUNT).contains(&idx) {
            continue;
        }
        let b = &mut buckets[idx as usize];
        b.tokens +=
            ev.input_tokens + ev.output_tokens + ev.cache_read_tokens + ev.cache_creation_tokens;
        b.cost_usd += pricing.compute_cost(ev).cost_usd;
    }

    buckets
}

/// Group events into 5-hour billing windows.
///
/// A new block begins when an event arrives more than [`BILLING_BLOCK_HOURS`] after
/// the previous block's start, or with the very first event. Events are processed
/// in chronological order; unordered input is sorted first.
pub fn billing_blocks(events: &[UsageEvent], pricing: &PricingTable) -> Vec<SessionBlock> {
    if events.is_empty() {
        return Vec::new();
    }

    let mut sorted: Vec<&UsageEvent> = events.iter().collect();
    sorted.sort_unstable_by_key(|e| e.timestamp);

    let mut blocks: Vec<SessionBlock> = Vec::new();

    for ev in sorted {
        let cost = pricing.compute_cost(ev).cost_usd;
        let tokens = ev.input_tokens + ev.output_tokens;

        match blocks.last_mut() {
            Some(b) if ev.timestamp < b.end => {
                b.cost_usd += cost;
                b.token_count += tokens;
                b.event_count += 1;
            }
            _ => {
                let start = ev.timestamp;
                blocks.push(SessionBlock {
                    start,
                    end: start + Duration::hours(BILLING_BLOCK_HOURS),
                    cost_usd: cost,
                    token_count: tokens,
                    event_count: 1,
                });
            }
        }
    }

    blocks
}

/// Builds 7×24 = 168 hourly buckets covering the current ISO week (Mon 00:00 local → Sun 23:00 local).
///
/// Sidechain events are excluded (sub-agent calls inflate raw counts).
/// Empty hours get zero values. Returns buckets sorted ascending (Mon h0 first).
pub fn build_hourly_heatmap(
    events: &[UsageEvent],
    pricing: &PricingTable,
    now: DateTime<Utc>,
) -> Vec<HeatmapHourBucket> {
    const BUCKET_COUNT: i64 = 7 * 24;

    let local_now = now.with_timezone(&Local);
    let days_since_monday = local_now.weekday().num_days_from_monday() as i64;

    // Monday 00:00 local → UTC.
    let week_start_local = Local
        .with_ymd_and_hms(local_now.year(), local_now.month(), local_now.day(), 0, 0, 0)
        .single()
        .expect("midnight is always valid")
        - Duration::days(days_since_monday);
    let week_start = week_start_local.to_utc();
    let week_end = week_start + Duration::days(7);

    let mut buckets: Vec<HeatmapHourBucket> = (0..BUCKET_COUNT)
        .map(|i| HeatmapHourBucket {
            hour_start: week_start + Duration::hours(i),
            tokens: 0,
            cost_usd: Decimal::ZERO,
        })
        .collect();

    for ev in events {
        if ev.is_sidechain {
            continue;
        }
        if ev.timestamp < week_start || ev.timestamp >= week_end {
            continue;
        }
        let idx = (ev.timestamp - week_start).num_hours();
        if !(0..BUCKET_COUNT).contains(&idx) {
            continue;
        }
        let b = &mut buckets[idx as usize];
        b.tokens +=
            ev.input_tokens + ev.output_tokens + ev.cache_read_tokens + ev.cache_creation_tokens;
        b.cost_usd += pricing.compute_cost(ev).cost_usd;
    }

    buckets
}

/// Return the block whose window contains `now`, if any.
pub fn active_block_at(blocks: &[SessionBlock], now: DateTime<Utc>) -> Option<SessionBlock> {
    blocks
        .iter()
        .find(|b| now >= b.start && now < b.end)
        .cloned()
}

fn accumulate_model(map: &mut BTreeMap<ModelId, ModelUsage>, ev: &UsageEvent, cost: Decimal) {
    let entry = map.entry(ev.model.clone()).or_default();
    entry.input_tokens += ev.input_tokens;
    entry.output_tokens += ev.output_tokens;
    entry.cache_read_tokens += ev.cache_read_tokens;
    // cache_creation_tokens ist das Top-Level-Summenfeld (= ephemeral_5m + ephemeral_1h).
    // Nur dieses zählen — ephemerals sind keine additive dritte Quelle.
    entry.cache_creation_tokens += ev.cache_creation_tokens;
    entry.cost_usd += cost;
    entry.event_count += 1;
}

fn accumulate_summary(summary: &mut Summary, ev: &UsageEvent, cost: Decimal) {
    let tokens = ev.input_tokens + ev.output_tokens + ev.cache_read_tokens;
    summary.total_cost_usd += cost;
    summary.total_tokens += tokens;
    summary.event_count += 1;
    if ev.is_sidechain {
        summary.sidechain_cost_usd += cost;
        summary.sidechain_event_count += 1;
    }
    accumulate_model(&mut summary.by_model, ev, cost);

    let proj = summary
        .by_project
        .entry(ev.project_path.clone())
        .or_default();
    proj.total_cost_usd += cost;
    proj.total_tokens += tokens;
}

/// Time-window boundaries derived from a single `now` instant.
struct Windows {
    today_start: DateTime<Utc>,
    week_start: DateTime<Utc>,
    month_start: DateTime<Utc>,
    last_30_days_start: DateTime<Utc>,
}

impl Windows {
    fn for_now(now: DateTime<Utc>) -> Self {
        let local = now.with_timezone(&Local);

        let today_start = Local
            .with_ymd_and_hms(local.year(), local.month(), local.day(), 0, 0, 0)
            .single()
            .expect("midnight always exists")
            .to_utc();

        // ISO week: Monday = day 0. chrono weekday().num_days_from_monday() gives 0=Mon..6=Sun.
        let days_since_monday = local.weekday().num_days_from_monday();
        let week_start = today_start - chrono::Duration::days(days_since_monday as i64);

        let month_start = Local
            .with_ymd_and_hms(local.year(), local.month(), 1, 0, 0, 0)
            .single()
            .expect("first of month always exists")
            .to_utc();

        // Rolling 30 days: today plus the 29 preceding days (midnight-aligned).
        let last_30_days_start = today_start - chrono::Duration::days(29);

        Self {
            today_start,
            week_start,
            month_start,
            last_30_days_start,
        }
    }

    fn in_today(&self, ts: DateTime<Utc>) -> bool {
        ts >= self.today_start
    }

    fn in_week(&self, ts: DateTime<Utc>) -> bool {
        ts >= self.week_start
    }

    fn in_month(&self, ts: DateTime<Utc>) -> bool {
        ts >= self.month_start
    }

    fn in_last_30_days(&self, ts: DateTime<Utc>) -> bool {
        ts >= self.last_30_days_start
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pricing::PricingTable;
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;
    use std::str::FromStr;

    fn pricing() -> PricingTable {
        PricingTable::embedded_default().expect("embedded pricing.json must parse")
    }

    fn make_event(
        session_id: &str,
        model: &str,
        ts: DateTime<Utc>,
        input: u64,
        output: u64,
    ) -> UsageEvent {
        UsageEvent {
            session_id: session_id.into(),
            uuid: "u".into(),
            timestamp: ts,
            project_path: PathBuf::from("C:\\project"),
            git_branch: None,
            model: ModelId::from(model),
            is_sidechain: false,
            input_tokens: input,
            output_tokens: output,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            cache_creation_ephemeral_5m: 0,
            cache_creation_ephemeral_1h: 0,
            web_search_requests: 0,
            web_fetch_requests: 0,
        }
    }

    /// Freeze "now" at 2026-04-17 14:00:00 UTC (a Friday).
    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 17, 14, 0, 0).unwrap()
    }

    /// 2026-04-17 09:00 UTC — same day as `now()`.
    fn ts_today() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 17, 9, 0, 0).unwrap()
    }

    /// 2026-04-14 10:00 UTC — Monday of the same ISO week, still in month.
    fn ts_this_week_not_today() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 14, 10, 0, 0).unwrap()
    }

    /// 2026-04-02 10:00 UTC — same month, previous week.
    fn ts_this_month_not_week() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 2, 10, 0, 0).unwrap()
    }

    /// 2026-03-28 12:00 UTC — safely in the previous month regardless of local timezone.
    fn ts_last_month() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 28, 12, 0, 0).unwrap()
    }

    #[test]
    fn empty_events_returns_zero_snapshot() {
        let snap = build_snapshot(&[], &pricing(), now());
        assert_eq!(snap.today.event_count, 0);
        assert_eq!(snap.this_week.event_count, 0);
        assert_eq!(snap.this_month.event_count, 0);
        assert!(snap.active_session.is_none());
        assert!(snap.sessions.is_empty());
    }

    #[test]
    fn today_event_appears_in_all_windows() {
        let ev = make_event("s1", "claude-sonnet-4-6", ts_today(), 1_000_000, 0);
        let snap = build_snapshot(&[ev], &pricing(), now());

        // 1M input @ $3.00/MTok
        let expected = rust_decimal::Decimal::from_str("3.00").unwrap();
        assert_eq!(snap.today.event_count, 1);
        assert_eq!(snap.today.total_cost_usd, expected);
        assert_eq!(snap.this_week.event_count, 1);
        assert_eq!(snap.this_month.event_count, 1);
    }

    #[test]
    fn week_event_skips_today_window() {
        let ev = make_event("s1", "claude-sonnet-4-6", ts_this_week_not_today(), 0, 0);
        let snap = build_snapshot(&[ev], &pricing(), now());
        assert_eq!(snap.today.event_count, 0);
        assert_eq!(snap.this_week.event_count, 1);
        assert_eq!(snap.this_month.event_count, 1);
    }

    #[test]
    fn month_event_skips_today_and_week_windows() {
        let ev = make_event("s1", "claude-sonnet-4-6", ts_this_month_not_week(), 0, 0);
        let snap = build_snapshot(&[ev], &pricing(), now());
        assert_eq!(snap.today.event_count, 0);
        assert_eq!(snap.this_week.event_count, 0);
        assert_eq!(snap.this_month.event_count, 1);
    }

    #[test]
    fn last_month_event_skips_all_windows() {
        let ev = make_event("s1", "claude-sonnet-4-6", ts_last_month(), 0, 0);
        let snap = build_snapshot(&[ev], &pricing(), now());
        assert_eq!(snap.today.event_count, 0);
        assert_eq!(snap.this_week.event_count, 0);
        assert_eq!(snap.this_month.event_count, 0);
    }

    #[test]
    fn active_session_detected_within_threshold() {
        // last event was 60s before now → active
        let ts = now() - chrono::Duration::seconds(60);
        let ev = make_event("sess-active", "claude-sonnet-4-6", ts, 0, 0);
        let snap = build_snapshot(&[ev], &pricing(), now());
        assert!(snap.active_session.is_some());
        assert_eq!(snap.active_session.unwrap().session_id, "sess-active");
    }

    #[test]
    fn inactive_session_not_active() {
        // last event was 10 minutes before now → inactive
        let ts = now() - chrono::Duration::minutes(10);
        let ev = make_event("sess-old", "claude-sonnet-4-6", ts, 0, 0);
        let snap = build_snapshot(&[ev], &pricing(), now());
        assert!(snap.active_session.is_none());
    }

    #[test]
    fn sessions_ordered_by_last_seen_descending() {
        let ev1 = make_event("older", "claude-sonnet-4-6", ts_this_week_not_today(), 0, 0);
        let ev2 = make_event("newer", "claude-sonnet-4-6", ts_today(), 0, 0);
        let snap = build_snapshot(&[ev1, ev2], &pricing(), now());
        assert_eq!(snap.sessions[0].session_id, "newer");
        assert_eq!(snap.sessions[1].session_id, "older");
    }

    #[test]
    fn unpriced_model_recorded_in_pricing_warnings() {
        let ev = make_event("s1", "claude-unknown-99", ts_today(), 100, 0);
        let snap = build_snapshot(&[ev], &pricing(), now());
        assert_eq!(snap.pricing_warnings.len(), 1);
        assert_eq!(snap.pricing_warnings[0], ModelId::from("claude-unknown-99"));
    }

    #[test]
    fn multiple_sessions_aggregated_independently() {
        let ev1 = make_event("s1", "claude-sonnet-4-6", ts_today(), 1_000_000, 0);
        let ev2 = make_event("s2", "claude-sonnet-4-6", ts_today(), 1_000_000, 0);
        let snap = build_snapshot(&[ev1, ev2], &pricing(), now());
        assert_eq!(snap.sessions.len(), 2);
        assert_eq!(snap.today.event_count, 2);

        let expected = rust_decimal::Decimal::from_str("6.00").unwrap();
        assert_eq!(snap.today.total_cost_usd, expected);
    }

    // ── billing_blocks() ──────────────────────────────────────────────────────

    #[test]
    fn empty_events_yields_no_blocks() {
        assert!(billing_blocks(&[], &pricing()).is_empty());
    }

    #[test]
    fn single_event_creates_one_block() {
        let ev = make_event("s1", "claude-sonnet-4-6", ts_today(), 100, 0);
        let blocks = billing_blocks(std::slice::from_ref(&ev), &pricing());
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].start, ev.timestamp);
        assert_eq!(blocks[0].end, ev.timestamp + chrono::Duration::hours(5));
        assert_eq!(blocks[0].event_count, 1);
    }

    #[test]
    fn events_within_5h_are_one_block() {
        let t0 = ts_today();
        let ev1 = make_event("s1", "claude-sonnet-4-6", t0, 100, 0);
        let ev2 = make_event(
            "s1",
            "claude-sonnet-4-6",
            t0 + chrono::Duration::hours(4),
            100,
            0,
        );
        let blocks = billing_blocks(&[ev1, ev2], &pricing());
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].event_count, 2);
    }

    #[test]
    fn event_exactly_at_block_end_starts_new_block() {
        let t0 = ts_today();
        let ev1 = make_event("s1", "claude-sonnet-4-6", t0, 100, 0);
        // exactly at t0 + 5h → NOT inside [start, end) → new block
        let ev2 = make_event(
            "s1",
            "claude-sonnet-4-6",
            t0 + chrono::Duration::hours(5),
            100,
            0,
        );
        let blocks = billing_blocks(&[ev1, ev2], &pricing());
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].event_count, 1);
        assert_eq!(blocks[1].event_count, 1);
    }

    #[test]
    fn events_more_than_5h_apart_create_separate_blocks() {
        let t0 = ts_today();
        let ev1 = make_event("s1", "claude-sonnet-4-6", t0, 100, 0);
        let ev2 = make_event(
            "s1",
            "claude-sonnet-4-6",
            t0 + chrono::Duration::hours(6),
            100,
            0,
        );
        let blocks = billing_blocks(&[ev1, ev2], &pricing());
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn unsorted_events_are_sorted_before_bucketing() {
        let t0 = ts_today();
        // deliberately reversed
        let ev_late = make_event(
            "s1",
            "claude-sonnet-4-6",
            t0 + chrono::Duration::hours(2),
            100,
            0,
        );
        let ev_early = make_event("s1", "claude-sonnet-4-6", t0, 100, 0);
        let blocks = billing_blocks(&[ev_late, ev_early], &pricing());
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].start, t0);
    }

    #[test]
    fn active_block_found_when_now_within_window() {
        let t0 = ts_today();
        let ev = make_event("s1", "claude-sonnet-4-6", t0, 100, 0);
        let blocks = billing_blocks(&[ev], &pricing());
        // now() is also ts_today() + a few hours — block spans t0..t0+5h
        let active = active_block_at(&blocks, t0 + chrono::Duration::hours(2));
        assert!(active.is_some());
    }

    #[test]
    fn active_block_none_when_now_outside_window() {
        let t0 = ts_today();
        let ev = make_event("s1", "claude-sonnet-4-6", t0, 100, 0);
        let blocks = billing_blocks(&[ev], &pricing());
        let active = active_block_at(&blocks, t0 + chrono::Duration::hours(6));
        assert!(active.is_none());
    }

    // ── build_hourly_heatmap() ────────────────────────────────────────────────

    #[test]
    fn hourly_heatmap_has_168_buckets() {
        let buckets = build_hourly_heatmap(&[], &pricing(), now());
        assert_eq!(buckets.len(), 168);
    }

    #[test]
    fn hourly_heatmap_buckets_span_full_week() {
        let buckets = build_hourly_heatmap(&[], &pricing(), now());
        let first = buckets[0].hour_start;
        let last = buckets[167].hour_start;
        assert_eq!(
            (last - first).num_hours(),
            167,
            "first and last bucket must be 167 hours apart"
        );
    }

    #[test]
    fn hourly_heatmap_counts_tokens_for_week_event() {
        let ev = make_event("s1", "claude-sonnet-4-6", ts_this_week_not_today(), 100, 50);
        let buckets = build_hourly_heatmap(&[ev], &pricing(), now());
        let total_tokens: u64 = buckets.iter().map(|b| b.tokens).sum();
        // 100 input + 50 output = 150 tokens in the bucket
        assert_eq!(total_tokens, 150);
    }

    #[test]
    fn hourly_heatmap_excludes_sidechain_events() {
        let mut ev = make_event("s1", "claude-sonnet-4-6", ts_today(), 100, 50);
        ev.is_sidechain = true;
        let buckets = build_hourly_heatmap(&[ev], &pricing(), now());
        let total_tokens: u64 = buckets.iter().map(|b| b.tokens).sum();
        assert_eq!(total_tokens, 0);
    }

    #[test]
    fn hourly_heatmap_excludes_events_outside_week() {
        // ts_last_month is well outside the current ISO week
        let ev = make_event("s1", "claude-sonnet-4-6", ts_last_month(), 100, 50);
        let buckets = build_hourly_heatmap(&[ev], &pricing(), now());
        let total_tokens: u64 = buckets.iter().map(|b| b.tokens).sum();
        assert_eq!(total_tokens, 0);
    }
}
