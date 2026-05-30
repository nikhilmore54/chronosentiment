use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DomainClass {
    CertifiedFixture,
    HistoricalSlice,
    SyntheticRegime,
    Holdout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubstrateKind {
    Fixture,
    Historical,
    Synthetic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubstrateSource {
    pub kind: SubstrateKind,
    pub reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioDomain {
    pub id: String,
    pub substrate_source: SubstrateSource,
    pub domain_class: DomainClass,
    pub evaluation_eligible: bool,
}

impl ScenarioDomain {
    pub fn certified_fixture(id: impl Into<String>, reference: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            substrate_source: SubstrateSource {
                kind: SubstrateKind::Fixture,
                reference: reference.into(),
                version: Some("v1".to_string()),
            },
            domain_class: DomainClass::CertifiedFixture,
            evaluation_eligible: true,
        }
    }
}
