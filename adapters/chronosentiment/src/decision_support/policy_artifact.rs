//! CS-P-006-A Policy Artifact consumption contract.
//!
//! Coralys produces a sealed artifact. ChronoSentiment evaluates it at T.
//! This module does not search, evolve, score, or invent split dates.
//! Outcomes are not inputs to `decide`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::reasoning::assessment::AssessmentProfile;

use super::dataset_partition::ChronologicalPartition;
use super::policy::{ensure_factor, factors_from_profile, DecisionPolicy, PolicyDecision};
use super::DecisionAction;

pub const POLICY_ARTIFACT_SCHEMA_VERSION: &str = "csp006a.policy_artifact.1";
pub const CONTRACT_FIXTURE_ENGINE: &str = "contract.fixture";
pub const CONTRACT_FIXTURE_METHODOLOGY: &str = "csp006a.contract.pending-protocol";
/// Schema `.1` certified concepts. Additional families (risk, cost, capital)
/// require a new schema_version; the evaluator already walks `input_schema`.
pub const CERTIFIED_INPUT_CONCEPTS: [&str; 3] = ["Trend", "Momentum", "Volatility"];

const FORBIDDEN_ENGINES: [&str; 2] = ["chronosentiment.handwritten", "threshold.grid"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitWindow {
    pub inclusive_start: DateTime<Utc>,
    pub exclusive_end: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TrainingProvenance {
    pub protocol_document_id: String,
    pub train: Option<SplitWindow>,
    pub validation: Option<SplitWindow>,
    pub test: Option<SplitWindow>,
}

impl TrainingProvenance {
    /// Map domain partitions onto the frozen CS-P-006-A provenance field names.
    pub fn from_chronological_partition(partition: &ChronologicalPartition) -> Self {
        Self {
            protocol_document_id: "CS-P-006-B.1".to_string(),
            train: Some(SplitWindow {
                inclusive_start: partition.development.inclusive_start,
                exclusive_end: partition.selection.inclusive_start,
            }),
            validation: Some(SplitWindow {
                inclusive_start: partition.selection.inclusive_start,
                exclusive_end: partition.evaluation.inclusive_start,
            }),
            test: Some(SplitWindow {
                inclusive_start: partition.evaluation.inclusive_start,
                exclusive_end: partition.evaluation.exclusive_end,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactorDefinition {
    pub concept: String,
    pub states: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactorPredicate {
    pub concept: String,
    pub present: Option<bool>,
    pub direction: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionRule {
    pub when: Vec<FactorPredicate>,
    pub action: DecisionAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyArtifact {
    pub schema_version: String,
    pub policy_id: String,
    pub policy_version: String,
    pub discovery_engine: String,
    pub discovery_run_id: String,
    pub input_schema: Vec<String>,
    pub factor_definitions: Vec<FactorDefinition>,
    pub action_space: Vec<DecisionAction>,
    pub rules: Vec<DecisionRule>,
    pub unmatched_action: DecisionAction,
    pub training_provenance: TrainingProvenance,
    pub allowed_information_timestamp: DateTime<Utc>,
    pub artifact_hash: String,
    pub methodology_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyArtifactError {
    EmptyField(&'static str),
    WrongSchema,
    ForbiddenEngine,
    IncompleteActionSpace,
    UnknownConcept(String),
    VolatilityDirectionForbidden,
    PredicateNotInSchema,
    WindowIncomplete,
    WindowInvalid,
    DiscoveredWithoutWindows,
    FixtureWithWindows,
    HashMismatch,
}

impl std::fmt::Display for PolicyArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField(name) => write!(f, "{name} must be non-empty"),
            Self::WrongSchema => {
                write!(f, "schema_version must be {POLICY_ARTIFACT_SCHEMA_VERSION}")
            }
            Self::ForbiddenEngine => write!(
                f,
                "discovery_engine is forbidden (handwritten / threshold grid)"
            ),
            Self::IncompleteActionSpace => {
                write!(f, "action_space must be exactly LONG, SHORT, NO_TRADE")
            }
            Self::UnknownConcept(c) => {
                write!(f, "concept {c} is not in the certified input schema")
            }
            Self::VolatilityDirectionForbidden => {
                write!(f, "Volatility is presence-only; no direction predicate")
            }
            Self::PredicateNotInSchema => {
                write!(
                    f,
                    "rule predicate is not in input_schema / factor_definitions"
                )
            }
            Self::WindowIncomplete => {
                write!(
                    f,
                    "TRAIN / VALIDATION / TEST windows must all be present or all absent"
                )
            }
            Self::WindowInvalid => {
                write!(
                    f,
                    "windows must be non-empty and strictly TRAIN then VALIDATION then TEST"
                )
            }
            Self::DiscoveredWithoutWindows => {
                write!(
                    f,
                    "coralys.* artifacts require complete windows from CS-P-006-B"
                )
            }
            Self::FixtureWithWindows => {
                write!(f, "contract.fixture must not carry split windows")
            }
            Self::HashMismatch => write!(f, "artifact_hash does not match sealed content"),
        }
    }
}

impl std::error::Error for PolicyArtifactError {}

#[derive(Serialize)]
struct ArtifactIdentity<'a> {
    schema_version: &'a str,
    policy_id: &'a str,
    policy_version: &'a str,
    discovery_engine: &'a str,
    discovery_run_id: &'a str,
    input_schema: &'a [String],
    factor_definitions: &'a [FactorDefinition],
    action_space: &'a [DecisionAction],
    rules: &'a [DecisionRule],
    unmatched_action: DecisionAction,
    training_provenance: &'a TrainingProvenance,
    allowed_information_timestamp: DateTime<Utc>,
    methodology_hash: &'a str,
}

pub fn certified_factor_definitions() -> Vec<FactorDefinition> {
    vec![
        FactorDefinition {
            concept: "Trend".to_string(),
            states: vec![
                "Bullish".to_string(),
                "Bearish".to_string(),
                "Neutral".to_string(),
                "absent".to_string(),
            ],
        },
        FactorDefinition {
            concept: "Momentum".to_string(),
            states: vec![
                "Positive".to_string(),
                "Negative".to_string(),
                "Neutral".to_string(),
                "absent".to_string(),
            ],
        },
        FactorDefinition {
            concept: "Volatility".to_string(),
            states: vec!["present".to_string(), "absent".to_string()],
        },
    ]
}

pub fn certified_input_schema() -> Vec<String> {
    CERTIFIED_INPUT_CONCEPTS
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

fn required_actions() -> [DecisionAction; 3] {
    [
        DecisionAction::Long,
        DecisionAction::Short,
        DecisionAction::NoTrade,
    ]
}

fn compute_hash(artifact: &PolicyArtifact) -> String {
    let payload = ArtifactIdentity {
        schema_version: &artifact.schema_version,
        policy_id: &artifact.policy_id,
        policy_version: &artifact.policy_version,
        discovery_engine: &artifact.discovery_engine,
        discovery_run_id: &artifact.discovery_run_id,
        input_schema: &artifact.input_schema,
        factor_definitions: &artifact.factor_definitions,
        action_space: &artifact.action_space,
        rules: &artifact.rules,
        unmatched_action: artifact.unmatched_action,
        training_provenance: &artifact.training_provenance,
        allowed_information_timestamp: artifact.allowed_information_timestamp,
        methodology_hash: &artifact.methodology_hash,
    };
    let bytes = serde_json::to_vec(&payload).expect("policy artifact identity serializes");
    format!("{:x}", Sha256::digest(&bytes))
}

fn window_ok(w: &SplitWindow) -> bool {
    w.inclusive_start < w.exclusive_end
}

fn validate(artifact: &PolicyArtifact) -> Result<(), PolicyArtifactError> {
    if artifact.schema_version != POLICY_ARTIFACT_SCHEMA_VERSION {
        return Err(PolicyArtifactError::WrongSchema);
    }
    for (field, value) in [
        ("policy_id", artifact.policy_id.as_str()),
        ("policy_version", artifact.policy_version.as_str()),
        ("discovery_engine", artifact.discovery_engine.as_str()),
        ("discovery_run_id", artifact.discovery_run_id.as_str()),
        ("methodology_hash", artifact.methodology_hash.as_str()),
        (
            "protocol_document_id",
            artifact.training_provenance.protocol_document_id.as_str(),
        ),
    ] {
        if value.trim().is_empty() {
            return Err(PolicyArtifactError::EmptyField(field));
        }
    }
    if FORBIDDEN_ENGINES.contains(&artifact.discovery_engine.as_str()) {
        return Err(PolicyArtifactError::ForbiddenEngine);
    }

    let mut space = artifact.action_space.clone();
    space.sort_by_key(|a| format!("{a:?}"));
    space.dedup();
    if space.len() != 3 || required_actions().iter().any(|a| !space.contains(a)) {
        return Err(PolicyArtifactError::IncompleteActionSpace);
    }
    if !artifact.action_space.contains(&artifact.unmatched_action) {
        return Err(PolicyArtifactError::IncompleteActionSpace);
    }

    if artifact.input_schema.is_empty() {
        return Err(PolicyArtifactError::EmptyField("input_schema"));
    }
    for concept in &artifact.input_schema {
        if !CERTIFIED_INPUT_CONCEPTS.contains(&concept.as_str()) {
            return Err(PolicyArtifactError::UnknownConcept(concept.clone()));
        }
    }

    for concept in &artifact.input_schema {
        if !artifact
            .factor_definitions
            .iter()
            .any(|d| &d.concept == concept && !d.states.is_empty())
        {
            return Err(PolicyArtifactError::PredicateNotInSchema);
        }
    }

    for rule in &artifact.rules {
        if !artifact.action_space.contains(&rule.action) {
            return Err(PolicyArtifactError::IncompleteActionSpace);
        }
        for pred in &rule.when {
            if !artifact.input_schema.iter().any(|c| c == &pred.concept) {
                return Err(PolicyArtifactError::PredicateNotInSchema);
            }
            if pred.concept == "Volatility" && pred.direction.is_some() {
                return Err(PolicyArtifactError::VolatilityDirectionForbidden);
            }
            if pred.present == Some(false) && pred.direction.is_some() {
                return Err(PolicyArtifactError::PredicateNotInSchema);
            }
            if let Some(dir) = &pred.direction {
                let def = artifact
                    .factor_definitions
                    .iter()
                    .find(|d| d.concept == pred.concept)
                    .ok_or(PolicyArtifactError::PredicateNotInSchema)?;
                if !def.states.iter().any(|s| s == dir) {
                    return Err(PolicyArtifactError::PredicateNotInSchema);
                }
            }
        }
    }

    let prov = &artifact.training_provenance;
    let filled = [
        prov.train.is_some(),
        prov.validation.is_some(),
        prov.test.is_some(),
    ];
    match filled {
        [false, false, false] => {}
        [true, true, true] => {
            let train = prov.train.as_ref().unwrap();
            let val = prov.validation.as_ref().unwrap();
            let test = prov.test.as_ref().unwrap();
            if !window_ok(train) || !window_ok(val) || !window_ok(test) {
                return Err(PolicyArtifactError::WindowInvalid);
            }
            if !(train.exclusive_end <= val.inclusive_start
                && val.exclusive_end <= test.inclusive_start)
            {
                return Err(PolicyArtifactError::WindowInvalid);
            }
        }
        _ => return Err(PolicyArtifactError::WindowIncomplete),
    }

    let is_fixture = artifact.discovery_engine == CONTRACT_FIXTURE_ENGINE;
    let is_coralys = artifact.discovery_engine.starts_with("coralys.");
    if !is_fixture && !is_coralys {
        return Err(PolicyArtifactError::ForbiddenEngine);
    }
    if is_fixture && filled[0] {
        return Err(PolicyArtifactError::FixtureWithWindows);
    }
    if is_coralys && !filled[0] {
        return Err(PolicyArtifactError::DiscoveredWithoutWindows);
    }

    Ok(())
}

impl PolicyArtifact {
    /// Validate and fill `artifact_hash`. Does not invent rules or split dates.
    pub fn seal(mut self) -> Result<Self, PolicyArtifactError> {
        self.artifact_hash.clear();
        validate(&self)?;
        self.artifact_hash = compute_hash(&self);
        Ok(self)
    }
}

/// ChronoSentiment evaluator for a sealed `PolicyArtifact`.
#[derive(Debug)]
pub struct ArtifactDecisionPolicy {
    artifact: PolicyArtifact,
    policy_name: String,
}

impl ArtifactDecisionPolicy {
    pub fn try_from_artifact(artifact: PolicyArtifact) -> Result<Self, PolicyArtifactError> {
        let provided = artifact.artifact_hash.clone();
        let mut draft = artifact;
        draft.artifact_hash.clear();
        let sealed = draft.seal()?;
        if !provided.is_empty() && provided != sealed.artifact_hash {
            return Err(PolicyArtifactError::HashMismatch);
        }
        let policy_name = format!("{}@{}", sealed.policy_id, sealed.policy_version);
        Ok(Self {
            artifact: sealed,
            policy_name,
        })
    }

    pub fn artifact(&self) -> &PolicyArtifact {
        &self.artifact
    }
}

fn predicate_matches(pred: &FactorPredicate, factors: &[super::EvidenceFactor]) -> bool {
    let factor = factors.iter().find(|f| f.concept == pred.concept);
    match (pred.present, factor) {
        (Some(false), None) => pred.direction.is_none(),
        (Some(false), Some(f)) => !f.present && pred.direction.is_none(),
        (Some(true), Some(f)) if f.present => direction_ok(pred, f),
        (Some(true), _) => false,
        (None, Some(f)) => {
            if let Some(dir) = &pred.direction {
                f.present && f.direction.as_deref() == Some(dir.as_str())
            } else {
                true
            }
        }
        (None, None) => pred.direction.is_none(),
    }
}

fn direction_ok(pred: &FactorPredicate, factor: &super::EvidenceFactor) -> bool {
    match &pred.direction {
        None => true,
        Some(dir) => factor.direction.as_deref() == Some(dir.as_str()),
    }
}

fn rule_matches(rule: &DecisionRule, factors: &[super::EvidenceFactor]) -> bool {
    rule.when.iter().all(|p| predicate_matches(p, factors))
}

/// First-match action from the certified TMV strings stored on an observatory record.
pub fn first_match_action_from_tmv(
    artifact: &PolicyArtifact,
    trend: &str,
    momentum: &str,
    volatility: &str,
) -> DecisionAction {
    let factors = [
        super::EvidenceFactor {
            concept: "Trend".into(),
            present: trend != "absent",
            direction: (trend != "absent").then(|| trend.to_string()),
            strength: None,
            assessment_confidence: None,
        },
        super::EvidenceFactor {
            concept: "Momentum".into(),
            present: momentum != "absent",
            direction: (momentum != "absent").then(|| momentum.to_string()),
            strength: None,
            assessment_confidence: None,
        },
        super::EvidenceFactor {
            concept: "Volatility".into(),
            present: volatility == "present",
            direction: None,
            strength: None,
            assessment_confidence: None,
        },
    ];
    for rule in &artifact.rules {
        if rule_matches(rule, &factors) {
            return rule.action;
        }
    }
    artifact.unmatched_action
}

/// First-match action over certified TMV factors. Used by the sealed evaluator and by search.
pub fn first_match_action(
    rules: &[DecisionRule],
    unmatched_action: DecisionAction,
    profile: &AssessmentProfile,
) -> DecisionAction {
    let mut factors = factors_from_profile(profile);
    for concept in CERTIFIED_INPUT_CONCEPTS {
        ensure_factor(&mut factors, concept);
    }
    for rule in rules {
        if rule_matches(rule, &factors) {
            return rule.action;
        }
    }
    unmatched_action
}

impl DecisionPolicy for ArtifactDecisionPolicy {
    fn name(&self) -> &str {
        &self.policy_name
    }

    fn decide(&self, assessment: &AssessmentProfile, _as_of: DateTime<Utc>) -> PolicyDecision {
        let mut factors = factors_from_profile(assessment);
        for concept in &self.artifact.input_schema {
            ensure_factor(&mut factors, concept);
        }
        factors.sort_by(|a, b| a.concept.cmp(&b.concept));

        let matched = self
            .artifact
            .rules
            .iter()
            .find(|rule| rule_matches(rule, &factors));

        let (action, consumed_concepts, action_reason) = match matched {
            Some(rule) => {
                let mut consumed: Vec<String> =
                    rule.when.iter().map(|p| p.concept.clone()).collect();
                consumed.sort();
                consumed.dedup();
                let reason = format!(
                    "artifact rule match → {:?} ({} predicates)",
                    rule.action,
                    rule.when.len()
                );
                (rule.action, consumed, reason)
            }
            None => (
                self.artifact.unmatched_action,
                Vec::new(),
                format!(
                    "no artifact rule matched → {:?}",
                    self.artifact.unmatched_action
                ),
            ),
        };

        let mapping_rule = format!(
            "artifact:{}:{}",
            POLICY_ARTIFACT_SCHEMA_VERSION, self.artifact.artifact_hash
        );
        let diagnostics = format!(
            "{action_reason}. Policy={}. Engine={}. Decision confidence UNAVAILABLE. Outcomes not consulted.",
            self.name(),
            self.artifact.discovery_engine
        );

        PolicyDecision {
            action,
            mapping_rule,
            action_reason,
            diagnostics,
            evidence_refs: vec![self.name().to_string(), self.artifact.artifact_hash.clone()],
            factors,
            consumed_concepts,
        }
    }
}
