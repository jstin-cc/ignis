//! winusage-core — scanner, parser, pricing and aggregation primitives.
//!
//! This crate is the single source of truth for reading Claude Code JSONL logs
//! and computing usage/cost summaries. API, CLI and Tray are downstream
//! consumers and must not parse JSONL themselves.

pub mod aggregate;
pub mod model;
pub mod parser;
pub mod pricing;
pub mod scanner;

pub use aggregate::build_snapshot;
pub use model::{ModelId, ModelUsage, ProjectUsage, SessionState, Snapshot, Summary, UsageEvent};
pub use pricing::{ComputedCost, ModelPricing, PricingTable};
