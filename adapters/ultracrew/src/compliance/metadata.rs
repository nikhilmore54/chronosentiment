/// Compliance Metadata — provenance and version information for compliance packs.
///
/// Every compliance pack exposes a [`ComplianceDescriptor`] so that the
/// optimizer can produce audit-ready constraint reports that cite the exact
/// regulatory version applied.
///
/// # Why this matters
///
/// - **Explainability**: every constraint violation in a schedule report can
///   cite the exact regulation and version that triggered it.
/// - **Audit logs**: evidence records can reference the compliance descriptor
///   to prove which rules were active at the time of scheduling.
/// - **Regulatory upgrades**: when a regulator publishes a new amendment, the old
///   descriptor is preserved and the new one supersedes it — no silent changes.
///
/// # Future extensions (do not add yet)
///
/// - `source_documents: Vec<String>` — regulation references, CAR numbers
/// - `jurisdiction: String`           — ISO 3166-1 alpha-2 country code
/// - `applicable_domains: Vec<String>`— "hospital", "factory", "retail"
/// - `supersedes: Option<String>`     — ID of the pack this version replaces
/// - `certification_status: String`   — "draft", "approved", "superseded"
///
/// Do not add dynamic plugin loading or runtime discovery here.
/// Static registration is sufficient for PX-001.

/// Stable provenance record for a compliance pack.
///
/// Attach one of these to every [`CompliancePack`] implementation so that
/// the registry can report which rules are active and why.
///
/// [`CompliancePack`]: crate::compliance::traits::CompliancePack
#[derive(Debug, Clone)]
pub struct ComplianceDescriptor {
    /// Unique, stable identifier for this pack version, e.g. `"eu-wtd-2003"`.
    pub id: String,

    /// Human-readable name, e.g. `"EU Working Time Directive (2003/88/EC)"`.
    pub name: String,

    /// Issuing authority, e.g. `"EU"`, `"OSHA"`, `"Ministry of Labour"`.
    pub authority: String,

    /// Semantic version of the rule set, e.g. `"2024.1"`.
    pub version: String,

    /// Date from which this version is effective (ISO 8601, optional).
    pub effective_from: Option<String>,

    /// Date on which this version expires or is superseded (ISO 8601, optional).
    pub expires_on: Option<String>,
}

impl ComplianceDescriptor {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        authority: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        ComplianceDescriptor {
            id: id.into(),
            name: name.into(),
            authority: authority.into(),
            version: version.into(),
            effective_from: None,
            expires_on: None,
        }
    }

    pub fn with_effective_from(mut self, date: impl Into<String>) -> Self {
        self.effective_from = Some(date.into());
        self
    }

    pub fn with_expires_on(mut self, date: impl Into<String>) -> Self {
        self.expires_on = Some(date.into());
        self
    }
}