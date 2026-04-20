use crate::model::{ModelId, ModelUsage, Snapshot, Summary};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Shared API state threaded through every handler.
#[derive(Clone)]
pub struct ApiState {
    /// Current snapshot; replaced atomically on each re-scan.
    pub snapshot: Arc<std::sync::RwLock<Arc<Snapshot>>>,
    /// Bearer token required for protected endpoints; empty string = dev-mode (no auth).
    pub api_token: String,
    /// Crate version string embedded at build time.
    pub version: &'static str,
}

impl ApiState {
    pub fn new(snapshot: Snapshot, api_token: String) -> Self {
        Self {
            snapshot: Arc::new(std::sync::RwLock::new(Arc::new(snapshot))),
            api_token,
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    /// Replace the current snapshot (called after each re-scan).
    pub fn update_snapshot(&self, snap: Snapshot) {
        if let Ok(mut guard) = self.snapshot.write() {
            *guard = Arc::new(snap);
        }
    }

    fn read_snapshot(&self) -> Arc<Snapshot> {
        self.snapshot
            .read()
            .map(|g| Arc::clone(&g))
            .unwrap_or_else(|poisoned| Arc::clone(&poisoned.into_inner()))
    }
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/summary", get(summary_handler))
        .route("/v1/sessions", get(sessions_handler))
        .fallback(not_found_handler)
        .with_state(state)
}

// ── Error response ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: &'static str,
}

fn error_response(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    (
        status,
        Json(ErrorBody {
            error: ErrorDetail { code, message },
        }),
    )
        .into_response()
}

// ── Auth helpers ──────────────────────────────────────────────────────────────

/// Returns `Err(Response)` when the request should be rejected.
fn check_auth(headers: &HeaderMap, token: &str) -> Result<(), Box<Response>> {
    // Dev-mode: empty token means no auth required.
    if token.is_empty() {
        return Ok(());
    }

    let bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match bearer {
        Some(t) if t == token => Ok(()),
        _ => Err(Box::new(error_response(
            StatusCode::UNAUTHORIZED,
            "auth_required",
            "Bearer token missing or invalid.",
        ))),
    }
}

/// Returns `Err(Response)` when the `Origin` header is present but not allowed.
///
/// Per the spec: requests without an `Origin` header (CLI tools, editor plugins)
/// are always allowed. Requests with `Origin` are blocked unless the allowlist
/// contains a matching entry (currently always empty in MVP → all browser origins
/// are rejected).
fn check_origin(headers: &HeaderMap) -> Result<(), Box<Response>> {
    if headers.get("origin").is_some() {
        return Err(Box::new(error_response(
            StatusCode::FORBIDDEN,
            "origin_rejected",
            "Origin header not in allowlist.",
        )));
    }
    Ok(())
}

// ── Handlers ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
    version: &'static str,
    snapshot_age_ms: i64,
    warnings: Vec<String>,
}

async fn health_handler(State(state): State<ApiState>) -> Response {
    let snap = state.read_snapshot();
    let age_ms = Utc::now()
        .signed_duration_since(snap.taken_at)
        .num_milliseconds();
    let warnings = snap
        .pricing_warnings
        .iter()
        .map(|m| format!("unknown_model:{}", m.0))
        .collect();

    Json(HealthResponse {
        ok: true,
        version: state.version,
        snapshot_age_ms: age_ms,
        warnings,
    })
    .into_response()
}

// ── /v1/summary ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SummaryQuery {
    range: Option<String>,
}

#[derive(Serialize)]
struct SummaryResponse {
    range: String,
    taken_at: DateTime<Utc>,
    total_cost_usd: String,
    total_tokens: u64,
    event_count: u64,
    by_model: Vec<ModelUsageDto>,
    by_project: Vec<ProjectUsageDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active_session: Option<ActiveSessionDto>,
    pricing_warnings: Vec<String>,
}

#[derive(Serialize)]
struct ModelUsageDto {
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    cost_usd: String,
    event_count: u64,
}

#[derive(Serialize)]
struct ProjectUsageDto {
    project_path: String,
    total_tokens: u64,
    total_cost_usd: String,
    session_count: u64,
}

#[derive(Serialize)]
struct ActiveSessionDto {
    session_id: String,
    project_path: String,
    git_branch: Option<String>,
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
    event_count: u64,
    total_cost_usd: String,
}

async fn summary_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(params): Query<SummaryQuery>,
) -> Response {
    if let Err(r) = check_auth(&headers, &state.api_token) {
        return *r;
    }
    if let Err(r) = check_origin(&headers) {
        return *r;
    }

    let range = params.range.as_deref().unwrap_or("today");
    let snap = state.read_snapshot();

    let summary: &Summary = match range {
        "today" => &snap.today,
        "week" => &snap.this_week,
        "month" => &snap.this_month,
        "all" => &snap.this_month, // "all" falls back to this_month in MVP
        _ => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "bad_request",
                "Invalid 'range'. Use today | week | month | all.",
            )
        }
    };

    let by_model = model_usage_dtos(&summary.by_model);
    let by_project = summary
        .by_project
        .iter()
        .map(|(path, proj)| ProjectUsageDto {
            project_path: path.to_string_lossy().into_owned(),
            total_tokens: proj.total_tokens,
            total_cost_usd: proj.total_cost_usd.to_string(),
            session_count: proj.session_count,
        })
        .collect();

    let active_session = snap.active_session.as_ref().map(|s| ActiveSessionDto {
        session_id: s.session_id.clone(),
        project_path: s.project_path.to_string_lossy().into_owned(),
        git_branch: s.git_branch.clone(),
        first_seen: s.first_seen,
        last_seen: s.last_seen,
        event_count: s.event_count,
        total_cost_usd: s.total_cost_usd.to_string(),
    });

    let pricing_warnings = snap
        .pricing_warnings
        .iter()
        .map(|m| format!("unknown_model:{}", m.0))
        .collect();

    Json(SummaryResponse {
        range: range.to_owned(),
        taken_at: snap.taken_at,
        total_cost_usd: summary.total_cost_usd.to_string(),
        total_tokens: summary.total_tokens,
        event_count: summary.event_count,
        by_model,
        by_project,
        active_session,
        pricing_warnings,
    })
    .into_response()
}

// ── /v1/sessions ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SessionsQuery {
    active: Option<bool>,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct SessionsResponse {
    taken_at: DateTime<Utc>,
    sessions: Vec<SessionDto>,
}

#[derive(Serialize)]
struct SessionDto {
    session_id: String,
    project_path: String,
    git_branch: Option<String>,
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
    is_active: bool,
    event_count: u64,
    total_cost_usd: String,
    by_model: Vec<SessionModelDto>,
}

#[derive(Serialize)]
struct SessionModelDto {
    model: String,
    cost_usd: String,
    tokens: u64,
}

async fn sessions_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(params): Query<SessionsQuery>,
) -> Response {
    if let Err(r) = check_auth(&headers, &state.api_token) {
        return *r;
    }
    if let Err(r) = check_origin(&headers) {
        return *r;
    }

    let limit = params.limit.unwrap_or(100).clamp(1, 500);
    let snap = state.read_snapshot();

    let active_id = snap.active_session.as_ref().map(|s| s.session_id.as_str());

    let sessions: Vec<SessionDto> = snap
        .sessions
        .iter()
        .filter(|s| match params.active {
            Some(true) => active_id == Some(s.session_id.as_str()),
            Some(false) => active_id != Some(s.session_id.as_str()),
            None => true,
        })
        .take(limit)
        .map(|s| {
            let is_active = active_id == Some(s.session_id.as_str());
            let by_model = s
                .by_model
                .iter()
                .map(|(mid, usage)| SessionModelDto {
                    model: mid.0.clone(),
                    cost_usd: usage.cost_usd.to_string(),
                    tokens: usage.input_tokens + usage.output_tokens + usage.cache_read_tokens,
                })
                .collect();
            SessionDto {
                session_id: s.session_id.clone(),
                project_path: s.project_path.to_string_lossy().into_owned(),
                git_branch: s.git_branch.clone(),
                first_seen: s.first_seen,
                last_seen: s.last_seen,
                is_active,
                event_count: s.event_count,
                total_cost_usd: s.total_cost_usd.to_string(),
                by_model,
            }
        })
        .collect();

    Json(SessionsResponse {
        taken_at: snap.taken_at,
        sessions,
    })
    .into_response()
}

async fn not_found_handler() -> Response {
    error_response(StatusCode::NOT_FOUND, "not_found", "Unknown path.")
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn model_usage_dtos(map: &std::collections::BTreeMap<ModelId, ModelUsage>) -> Vec<ModelUsageDto> {
    map.iter()
        .map(|(mid, u)| ModelUsageDto {
            model: mid.0.clone(),
            input_tokens: u.input_tokens,
            output_tokens: u.output_tokens,
            cache_read_tokens: u.cache_read_tokens,
            cache_creation_tokens: u.cache_creation_tokens,
            cost_usd: u.cost_usd.to_string(),
            event_count: u.event_count,
        })
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request};
    use chrono::TimeZone;
    use rust_decimal::Decimal;
    use tower::ServiceExt;

    fn empty_snapshot() -> Snapshot {
        Snapshot {
            taken_at: Utc.with_ymd_and_hms(2026, 4, 17, 12, 0, 0).unwrap(),
            today: Summary::default(),
            this_week: Summary::default(),
            this_month: Summary::default(),
            active_session: None,
            sessions: vec![],
            active_block: None,
            pricing_warnings: vec![],
        }
    }

    fn make_state(token: &str) -> ApiState {
        ApiState::new(empty_snapshot(), token.to_owned())
    }

    async fn get_json(app: Router, path: &str) -> (StatusCode, serde_json::Value) {
        let req = Request::builder()
            .method(Method::GET)
            .uri(path)
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        (status, json)
    }

    async fn get_with_bearer(
        app: Router,
        path: &str,
        token: &str,
    ) -> (StatusCode, serde_json::Value) {
        let req = Request::builder()
            .method(Method::GET)
            .uri(path)
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        (status, json)
    }

    #[tokio::test]
    async fn health_returns_200_without_auth() {
        let app = router(make_state("secret"));
        let (status, body) = get_json(app, "/health").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["ok"], true);
        assert!(body["version"].is_string());
        assert!(body["snapshot_age_ms"].is_number());
    }

    #[tokio::test]
    async fn summary_requires_auth_when_token_set() {
        let app = router(make_state("secret"));
        let (status, body) = get_json(app, "/v1/summary").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"]["code"], "auth_required");
    }

    #[tokio::test]
    async fn summary_wrong_token_returns_401() {
        let app = router(make_state("secret"));
        let (status, body) = get_with_bearer(app, "/v1/summary", "wrong").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"]["code"], "auth_required");
    }

    #[tokio::test]
    async fn summary_correct_token_returns_200() {
        let app = router(make_state("secret"));
        let (status, body) = get_with_bearer(app, "/v1/summary", "secret").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["range"], "today");
    }

    #[tokio::test]
    async fn summary_dev_mode_no_token_required() {
        let app = router(make_state(""));
        let (status, _) = get_json(app, "/v1/summary").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn summary_invalid_range_returns_400() {
        let app = router(make_state(""));
        let (status, body) = get_json(app, "/v1/summary?range=invalid").await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "bad_request");
    }

    #[tokio::test]
    async fn summary_range_week_returns_200() {
        let app = router(make_state(""));
        let (status, body) = get_json(app, "/v1/summary?range=week").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["range"], "week");
    }

    #[tokio::test]
    async fn sessions_requires_auth_when_token_set() {
        let app = router(make_state("tok"));
        let (status, body) = get_json(app, "/v1/sessions").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"]["code"], "auth_required");
    }

    #[tokio::test]
    async fn sessions_returns_empty_list() {
        let app = router(make_state(""));
        let (status, body) = get_json(app, "/v1/sessions").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body["sessions"].is_array());
        assert_eq!(body["sessions"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn unknown_path_returns_404() {
        let app = router(make_state(""));
        let (status, body) = get_json(app, "/unknown").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "not_found");
    }

    #[tokio::test]
    async fn origin_header_rejected_on_v1() {
        let app = router(make_state(""));
        let req = Request::builder()
            .method(Method::GET)
            .uri("/v1/summary")
            .header("origin", "https://evil.example.com")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn health_cors_allows_any_origin() {
        // /health should not be blocked by origin check
        let app = router(make_state(""));
        let req = Request::builder()
            .method(Method::GET)
            .uri("/health")
            .header("origin", "https://any.example.com")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn update_snapshot_is_reflected_in_health() {
        let state = make_state("");
        let app = router(state.clone());

        let new_snap = Snapshot {
            taken_at: Utc.with_ymd_and_hms(2026, 4, 18, 6, 0, 0).unwrap(),
            today: Summary::default(),
            this_week: Summary::default(),
            this_month: Summary::default(),
            active_session: None,
            sessions: vec![],
            active_block: None,
            pricing_warnings: vec![ModelId::from("claude-unknown-99")],
        };
        state.update_snapshot(new_snap);

        let (status, body) = get_json(app, "/health").await;
        assert_eq!(status, StatusCode::OK);
        let warnings = body["warnings"].as_array().unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].as_str().unwrap().contains("claude-unknown-99"));
    }

    // Verify Decimal amounts serialize as strings, not floats.
    #[tokio::test]
    async fn summary_cost_is_string_not_float() {
        let state = make_state("");
        // inject a snapshot with a non-zero cost
        let mut summary = Summary::default();
        summary.total_cost_usd = Decimal::new(243, 2); // 2.43
        summary.total_tokens = 1000;
        summary.event_count = 1;
        let snap = Snapshot {
            taken_at: Utc.with_ymd_and_hms(2026, 4, 17, 12, 0, 0).unwrap(),
            today: summary,
            this_week: Summary::default(),
            this_month: Summary::default(),
            active_session: None,
            sessions: vec![],
            active_block: None,
            pricing_warnings: vec![],
        };
        state.update_snapshot(snap);
        let app = router(state);
        let (status, body) = get_json(app, "/v1/summary?range=today").await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body["total_cost_usd"].is_string(),
            "total_cost_usd must be a string, got: {}",
            body["total_cost_usd"]
        );
        assert_eq!(body["total_cost_usd"], "2.43");
    }
}
