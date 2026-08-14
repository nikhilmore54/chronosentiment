//! Read-only context labels for CS-P-004.
//!
//! Loads assessment profiles and instrument symbols already persisted at T.
//! Does not call `decide_at` and does not change LONG/SHORT/NO_TRADE rules.

use std::collections::HashMap;

use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::metrics::concepts::Concept;
use crate::reasoning::assessment::AssessmentProfile;

use super::backtest::DecisionLedger;
use super::laboratory::DecisionContext;
use super::replay::ReplayError;

pub async fn load_decision_context(
    pool: &PgPool,
    ledger: &DecisionLedger,
) -> Result<Vec<DecisionContext>, ReplayError> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION READ ONLY")
        .execute(&mut *tx)
        .await?;

    let inst_rows = sqlx::query("SELECT id, display_symbol FROM instruments")
        .fetch_all(&mut *tx)
        .await?;
    let mut symbols: HashMap<Uuid, String> = HashMap::new();
    for row in inst_rows {
        let id: Uuid = row.try_get("id")?;
        let symbol: String = row.try_get("display_symbol")?;
        symbols.insert(id, symbol);
    }

    let ids: Vec<Uuid> = ledger
        .records
        .iter()
        .filter_map(|r| r.lineage.assessment_id)
        .collect();

    let assess_rows = sqlx::query(
        r#"
        SELECT id, profile_json
        FROM knowledge_assessments
        WHERE id = ANY($1)
        "#,
    )
    .bind(&ids)
    .fetch_all(&mut *tx)
    .await?;

    let mut profiles: HashMap<Uuid, AssessmentProfile> = HashMap::new();
    for row in assess_rows {
        let id: Uuid = row.try_get("id")?;
        let profile_json: serde_json::Value = row.try_get("profile_json")?;
        let profile: AssessmentProfile =
            serde_json::from_value(profile_json).map_err(|e| ReplayError::Profile(e.to_string()))?;
        profiles.insert(id, profile);
    }
    tx.commit().await?;

    let mut out = Vec::with_capacity(ledger.records.len());
    for rec in &ledger.records {
        let label = symbols
            .get(&rec.instrument_id)
            .cloned()
            .unwrap_or_else(|| rec.instrument_id.to_string());
        let profile = rec
            .lineage
            .assessment_id
            .and_then(|id| profiles.get(&id));
        out.push(if rec.evidence.factors.is_empty() {
            let mut ctx = context_from_profile(rec.decision_id, label, profile);
            ctx.confidence_status = Some(format!("{:?}", rec.confidence_status));
            ctx.mapping_rule = Some(rec.evidence.mapping_rule.clone()).filter(|s| !s.is_empty());
            ctx
        } else {
            context_from_record(rec, label)
        });
    }
    Ok(out)
}

pub fn context_from_profile(
    decision_id: Uuid,
    instrument_label: String,
    profile: Option<&AssessmentProfile>,
) -> DecisionContext {
    let mut trend = None;
    let mut trend_strength = None;
    let mut momentum = None;
    let mut momentum_strength = None;
    let mut volatility = None;
    if let Some(profile) = profile {
        for a in &profile.assessments {
            let dir = format!("{:?}", a.direction);
            let strength = a.strength.as_ref().map(|s| format!("{s:?}"));
            match a.concept {
                Concept::Trend => {
                    trend = Some(dir);
                    trend_strength = strength;
                }
                Concept::Momentum => {
                    momentum = Some(dir);
                    momentum_strength = strength;
                }
                Concept::Volatility => {
                    volatility = Some(dir);
                }
                _ => {}
            }
        }
    }
    DecisionContext {
        decision_id,
        instrument_label,
        trend,
        trend_strength,
        momentum,
        momentum_strength,
        volatility,
        confidence_status: None,
        mapping_rule: None,
    }
}

pub fn context_from_record(
    rec: &super::backtest::LedgerRecord,
    instrument_label: String,
) -> DecisionContext {
    let factor = |name: &str| rec.evidence.factors.iter().find(|f| f.concept == name);
    let present_dir = |name: &str| {
        factor(name).and_then(|f| {
            if f.present {
                f.direction.clone()
            } else {
                None
            }
        })
    };
    let present_str = |name: &str| {
        factor(name).and_then(|f| {
            if f.present {
                f.strength.clone()
            } else {
                None
            }
        })
    };
    DecisionContext {
        decision_id: rec.decision_id,
        instrument_label,
        trend: present_dir("Trend"),
        trend_strength: present_str("Trend"),
        momentum: present_dir("Momentum"),
        momentum_strength: present_str("Momentum"),
        volatility: present_dir("Volatility"),
        confidence_status: Some(format!("{:?}", rec.confidence_status)),
        mapping_rule: Some(rec.evidence.mapping_rule.clone()),
    }
}
