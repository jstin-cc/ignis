use crate::model::{ModelId, UsageEvent};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

const MTOK: u64 = 1_000_000;

/// Cost per million tokens for one model.
#[derive(Clone, Debug)]
pub struct ModelPricing {
    pub input_per_mtok: Decimal,
    pub output_per_mtok: Decimal,
    pub cache_read_per_mtok: Decimal,
    pub cache_write_5m_per_mtok: Decimal,
    pub cache_write_1h_per_mtok: Decimal,
}

/// Result of a cost computation for one `UsageEvent`.
#[derive(Clone, Debug)]
pub struct ComputedCost {
    pub cost_usd: Decimal,
    /// True when the model had no entry in the pricing table.
    pub unpriced: bool,
}

#[derive(Debug, Error)]
pub enum PricingError {
    #[error("failed to parse embedded pricing JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid decimal in pricing JSON for field '{field}': {value}")]
    Decimal { field: &'static str, value: String },
}

/// Pricing table loaded from the embedded `pricing.json`.
#[derive(Clone)]
pub struct PricingTable {
    models: HashMap<String, ModelPricing>,
}

impl PricingTable {
    /// Load from the JSON embedded at compile time.
    pub fn embedded_default() -> Result<Self, PricingError> {
        let raw: RawPricingFile = serde_json::from_str(include_str!("pricing.json"))?;
        let mut models = HashMap::with_capacity(raw.models.len());
        for (id, r) in raw.models {
            let p = ModelPricing {
                input_per_mtok: parse_decimal("input_per_mtok", &r.input_per_mtok)?,
                output_per_mtok: parse_decimal("output_per_mtok", &r.output_per_mtok)?,
                cache_read_per_mtok: parse_decimal("cache_read_per_mtok", &r.cache_read_per_mtok)?,
                cache_write_5m_per_mtok: parse_decimal(
                    "cache_write_5m_per_mtok",
                    &r.cache_write_5m_per_mtok,
                )?,
                cache_write_1h_per_mtok: parse_decimal(
                    "cache_write_1h_per_mtok",
                    &r.cache_write_1h_per_mtok,
                )?,
            };
            models.insert(id, p);
        }
        Ok(Self { models })
    }

    /// Look up pricing for a model ID.
    ///
    /// Tries an exact match first. On miss, strips a trailing `-YYYYMMDD` date
    /// suffix (e.g. `claude-haiku-4-5-20251001` → `claude-haiku-4-5`) and retries.
    pub fn lookup(&self, model: &ModelId) -> Option<&ModelPricing> {
        self.models
            .get(&model.0)
            .or_else(|| self.models.get(strip_date_suffix(&model.0)))
    }

    /// Compute the USD cost for one usage event.
    ///
    /// Returns `ComputedCost { cost_usd: 0, unpriced: true }` when the model is
    /// not in the table — the caller should add the model to `pricing_warnings`.
    pub fn compute_cost(&self, event: &UsageEvent) -> ComputedCost {
        let Some(p) = self.lookup(&event.model) else {
            return ComputedCost {
                cost_usd: Decimal::ZERO,
                unpriced: true,
            };
        };

        let cost = tokens_cost(event.input_tokens, &p.input_per_mtok)
            + tokens_cost(event.output_tokens, &p.output_per_mtok)
            + tokens_cost(event.cache_read_tokens, &p.cache_read_per_mtok)
            + tokens_cost(
                event.cache_creation_ephemeral_5m,
                &p.cache_write_5m_per_mtok,
            )
            + tokens_cost(
                event.cache_creation_ephemeral_1h,
                &p.cache_write_1h_per_mtok,
            );

        ComputedCost {
            cost_usd: cost,
            unpriced: false,
        }
    }
}

fn tokens_cost(tokens: u64, per_mtok: &Decimal) -> Decimal {
    Decimal::from(tokens) * per_mtok / Decimal::from(MTOK)
}

fn parse_decimal(field: &'static str, value: &str) -> Result<Decimal, PricingError> {
    Decimal::from_str(value).map_err(|_| PricingError::Decimal {
        field,
        value: value.to_owned(),
    })
}

/// Strip a trailing `-YYYYMMDD` suffix from a model ID string.
fn strip_date_suffix(id: &str) -> &str {
    let bytes = id.as_bytes();
    // Format: 9 trailing chars: '-' + 8 ASCII digits.
    if bytes.len() > 9 && bytes[bytes.len() - 9] == b'-' {
        let suffix = &bytes[bytes.len() - 8..];
        if suffix.iter().all(|b| b.is_ascii_digit()) {
            return &id[..id.len() - 9];
        }
    }
    id
}

// ── Private deserialization types ─────────────────────────────────────────────

#[derive(Deserialize)]
struct RawPricingFile {
    models: HashMap<String, RawModelPricing>,
}

#[derive(Deserialize)]
struct RawModelPricing {
    input_per_mtok: String,
    output_per_mtok: String,
    cache_read_per_mtok: String,
    cache_write_5m_per_mtok: String,
    cache_write_1h_per_mtok: String,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::UsageEvent;
    use chrono::Utc;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    fn table() -> PricingTable {
        PricingTable::embedded_default().expect("embedded pricing.json must parse")
    }

    fn event_with(model: &str, input: u64, output: u64, cache_read: u64) -> UsageEvent {
        UsageEvent {
            session_id: "s".into(),
            uuid: "u".into(),
            timestamp: Utc::now(),
            project_path: PathBuf::from("C:\\test"),
            git_branch: None,
            model: ModelId::from(model),
            is_sidechain: false,
            input_tokens: input,
            output_tokens: output,
            cache_read_tokens: cache_read,
            cache_creation_tokens: 0,
            cache_creation_ephemeral_5m: 0,
            cache_creation_ephemeral_1h: 0,
            web_search_requests: 0,
            web_fetch_requests: 0,
        }
    }

    #[test]
    fn embedded_default_loads_without_error() {
        let t = table();
        assert!(t.models.len() >= 4, "expected at least 4 model entries");
    }

    #[test]
    fn embedded_default_has_current_models() {
        let t = table();
        for id in [
            "claude-opus-4-7",
            "claude-opus-4-6",
            "claude-sonnet-4-6",
            "claude-haiku-4-5",
        ] {
            assert!(
                t.models.contains_key(id),
                "pricing.json missing required model '{id}'"
            );
        }
    }

    #[test]
    fn opus_47_pricing_matches_anthropic_2026_04() {
        let t = table();
        let p = t
            .lookup(&ModelId::from("claude-opus-4-7"))
            .expect("opus-4-7 must be priced");
        assert_eq!(p.input_per_mtok, Decimal::from_str("5.00").unwrap());
        assert_eq!(p.output_per_mtok, Decimal::from_str("25.00").unwrap());
        assert_eq!(p.cache_read_per_mtok, Decimal::from_str("0.50").unwrap());
        assert_eq!(
            p.cache_write_5m_per_mtok,
            Decimal::from_str("6.25").unwrap()
        );
        assert_eq!(
            p.cache_write_1h_per_mtok,
            Decimal::from_str("10.00").unwrap()
        );
    }

    #[test]
    fn haiku_45_pricing_matches_anthropic_2026_04() {
        let t = table();
        let p = t
            .lookup(&ModelId::from("claude-haiku-4-5"))
            .expect("haiku-4-5 must be priced");
        assert_eq!(p.input_per_mtok, Decimal::from_str("1.00").unwrap());
        assert_eq!(p.output_per_mtok, Decimal::from_str("5.00").unwrap());
        assert_eq!(p.cache_read_per_mtok, Decimal::from_str("0.10").unwrap());
        assert_eq!(
            p.cache_write_5m_per_mtok,
            Decimal::from_str("1.25").unwrap()
        );
        assert_eq!(
            p.cache_write_1h_per_mtok,
            Decimal::from_str("2.00").unwrap()
        );
    }

    #[test]
    fn exact_match_lookup() {
        let t = table();
        assert!(t.lookup(&ModelId::from("claude-sonnet-4-6")).is_some());
    }

    #[test]
    fn date_suffix_fallback() {
        let t = table();
        // claude-haiku-4-5-20251001 is not in the table; claude-haiku-4-5 is.
        let result = t.lookup(&ModelId::from("claude-haiku-4-5-20251001"));
        assert!(result.is_some(), "date-suffix fallback must resolve");
    }

    #[test]
    fn unknown_model_returns_none() {
        let t = table();
        assert!(t.lookup(&ModelId::from("claude-opus-99")).is_none());
    }

    #[test]
    fn unpriced_event_returns_zero_cost_and_flag() {
        let t = table();
        let ev = event_with("claude-opus-99", 1000, 500, 0);
        let cost = t.compute_cost(&ev);
        assert_eq!(cost.cost_usd, Decimal::ZERO);
        assert!(cost.unpriced);
    }

    #[test]
    fn cost_calculation_sonnet() {
        let t = table();
        // 1M input tokens at $3.00/MTok = $3.00
        let ev = event_with("claude-sonnet-4-6", 1_000_000, 0, 0);
        let cost = t.compute_cost(&ev);
        assert!(!cost.unpriced);
        assert_eq!(cost.cost_usd, Decimal::from_str("3.00").unwrap());
    }

    #[test]
    fn strip_date_suffix_works() {
        assert_eq!(
            strip_date_suffix("claude-haiku-4-5-20251001"),
            "claude-haiku-4-5"
        );
        assert_eq!(strip_date_suffix("claude-sonnet-4-6"), "claude-sonnet-4-6");
        assert_eq!(strip_date_suffix("short"), "short");
        assert_eq!(strip_date_suffix("no-date-here1234"), "no-date-here1234");
    }
}
