//! winusage-core — scanner, parser, pricing and aggregation primitives.
//!
//! This crate is the single source of truth for reading Claude Code JSONL logs
//! and computing usage/cost summaries. API, CLI and Tray are downstream
//! consumers and must not parse JSONL themselves.

pub mod aggregate;
pub mod api;
pub mod config;
pub mod model;
pub mod parser;
pub mod pricing;
pub mod provider;
pub mod scanner;

pub use aggregate::{active_block_at, billing_blocks, build_snapshot, daily_costs};
pub use config::{Config, ConfigError, PlanConfig, PlanKind};
pub use model::{
    HeatmapDay, ModelId, ModelUsage, ProjectUsage, SessionBlock, SessionState, Snapshot, Summary,
    UsageEvent,
};
pub use pricing::{ComputedCost, ModelPricing, PricingTable};
pub use provider::{ClaudeCodeProvider, Provider};
pub use scanner::{scan_all, scan_delta, scan_incremental, FilePosition, ScanError, ScanResult};
