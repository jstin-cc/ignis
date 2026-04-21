// API response types for the WinUsage HTTP API (docs/api.md).
// Cost amounts are strings (serialized rust_decimal::Decimal) — never parse as float.

export interface ModelUsage {
  model: string;
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cache_creation_tokens: number;
  cost_usd: string;
  event_count: number;
}

export interface ProjectUsage {
  project_path: string;
  total_tokens: number;
  total_cost_usd: string;
  session_count: number;
}

export interface ActiveSession {
  session_id: string;
  project_path: string;
  git_branch: string | null;
  first_seen: string;
  last_seen: string;
  event_count: number;
  total_cost_usd: string;
}

export interface ActiveBlock {
  start: string;
  end: string;
  cost_usd: string;
  token_count: number;
  event_count: number;
  /** 0–100: fraction of the 5-hour window elapsed (time-based) */
  percent_elapsed: number;
  /** Plan token limit for this block */
  block_token_limit: number;
  /** 0–100: fraction of the plan token limit consumed (token-based) */
  block_token_pct: number;
}

export type PlanKind = "pro" | "max5" | "max20" | "custom";

export interface PlanConfig {
  kind: PlanKind;
  custom_token_limit: number | null;
}

export interface SummaryResponse {
  range: string;
  taken_at: string;
  total_cost_usd: string;
  total_tokens: number;
  event_count: number;
  by_model: ModelUsage[];
  by_project: ProjectUsage[];
  active_session: ActiveSession | null;
  active_block: ActiveBlock | null;
  pricing_warnings: string[];
  sidechain_cost_usd: string;
  sidechain_event_count: number;
}

export interface SessionModelUsage {
  model: string;
  cost_usd: string;
  tokens: number;
}

export interface Session {
  session_id: string;
  project_path: string;
  git_branch: string | null;
  first_seen: string;
  last_seen: string;
  is_active: boolean;
  event_count: number;
  total_cost_usd: string;
  by_model: SessionModelUsage[];
}

export interface SessionsResponse {
  taken_at: string;
  sessions: Session[];
}

export interface HeatmapDay {
  date: string;
  cost_usd: string;
}

export interface UsageData {
  today: SummaryResponse | null;
  month: SummaryResponse | null;
  activeSession: Session | null;
  activeBlock: ActiveBlock | null;
  heatmap: HeatmapDay[];
  loading: boolean;
  error: string | null;
}
