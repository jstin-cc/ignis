use crate::model::{ModelId, ModelUsage, SessionState, Snapshot, Summary, UsageEvent};
use crate::pricing::PricingTable;
use chrono::{DateTime, Datelike, Local, TimeZone, Utc};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Sessions inactive for longer than this are not considered "active".
const ACTIVE_THRESHOLD_SECS: i64 = 300;

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
    let mut unpriced: HashSet<ModelId> = HashSet::new();

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
        }
        if windows.in_week(ev.timestamp) {
            accumulate_summary(&mut this_week, ev, cost);
        }
        if windows.in_month(ev.timestamp) {
            accumulate_summary(&mut this_month, ev, cost);
        }
    }

    let mut sessions_vec: Vec<SessionState> = sessions.into_values().collect();
    sessions_vec.sort_unstable_by_key(|s| std::cmp::Reverse(s.last_seen));

    let active_session = sessions_vec
        .iter()
        .find(|s| now.signed_duration_since(s.last_seen).num_seconds() <= ACTIVE_THRESHOLD_SECS)
        .cloned();

    let mut pricing_warnings: Vec<ModelId> = unpriced.into_iter().collect();
    pricing_warnings.sort();

    Snapshot {
        taken_at: now,
        today,
        this_week,
        this_month,
        active_session,
        sessions: sessions_vec,
        pricing_warnings,
    }
}

fn accumulate_model(map: &mut BTreeMap<ModelId, ModelUsage>, ev: &UsageEvent, cost: Decimal) {
    let entry = map.entry(ev.model.clone()).or_default();
    entry.input_tokens += ev.input_tokens;
    entry.output_tokens += ev.output_tokens;
    entry.cache_read_tokens += ev.cache_read_tokens;
    entry.cache_creation_tokens +=
        ev.cache_creation_tokens + ev.cache_creation_ephemeral_5m + ev.cache_creation_ephemeral_1h;
    entry.cost_usd += cost;
    entry.event_count += 1;
}

fn accumulate_summary(summary: &mut Summary, ev: &UsageEvent, cost: Decimal) {
    let tokens = ev.input_tokens + ev.output_tokens + ev.cache_read_tokens;
    summary.total_cost_usd += cost;
    summary.total_tokens += tokens;
    summary.event_count += 1;
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
        let week_start = Local
            .with_ymd_and_hms(local.year(), local.month(), local.day(), 0, 0, 0)
            .single()
            .expect("midnight always exists")
            .to_utc()
            - chrono::Duration::days(days_since_monday as i64);

        let month_start = Local
            .with_ymd_and_hms(local.year(), local.month(), 1, 0, 0, 0)
            .single()
            .expect("first of month always exists")
            .to_utc();

        Self {
            today_start,
            week_start,
            month_start,
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
}
