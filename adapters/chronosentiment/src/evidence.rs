// ChronoSentiment — Evidence (immutable research sources)
//
// Evidence items are immutable once recorded. No adapter may mutate
// historical evidence. This enforces the platform invariant:
//   "Evidence is immutable once recorded."
//
// Evidence items are the raw material for Investment Theses. They are
// linked to the thesis versions that cite them.

use serde::{Deserialize, Serialize};

// ── Evidence Source Type ──────────────────────────────────────────────────────

/// The type of evidence source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceSourceType {
    /// Company annual report or 10-K filing.
    AnnualReport,
    /// Earnings call transcript or notes.
    EarningsCall,
    /// Documented AI research conversation.
    AiConversation,
    /// News article or press release.
    News,
    /// Investor's own research note or observation.
    PersonalNote,
    /// Financial metrics, ratios, or data.
    FinancialData,
    /// Industry or sector research report.
    SectorResearch,
    /// Regulatory filing (SEC, FCA, etc.).
    RegulatoryFiling,
    /// Expert interview or consultation.
    ExpertInterview,
    /// Committee meeting notes or minutes.
    CommitteeMeeting,
}

// ── Evidence Item ─────────────────────────────────────────────────────────────

/// An immutable evidence item — a single research source.
///
/// Platform invariant: evidence items are append-only. Once added to a
/// Workspace, they cannot be modified or removed. This ensures that the
/// historical record of research is preserved exactly as it was at the
/// time of the decision.
///
/// If an evidence item is found to be incorrect, a new evidence item
/// should be added that supersedes it — the original is never modified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub evidence_id: String,
    pub workspace_id: String,
    pub title: String,
    pub source_type: EvidenceSourceType,
    /// The content or summary of the evidence.
    pub content: String,
    /// Optional URL or file reference.
    pub source_url: Option<String>,
    /// Optional author or source name.
    pub source_name: Option<String>,
    /// Optional date of the source (e.g. publication date of the annual report).
    pub source_date: Option<String>,
    /// IDs of thesis versions that cite this evidence item.
    pub cited_by_thesis_versions: Vec<u32>,
    /// Whether this evidence item has been superseded by a later item.
    pub superseded_by: Option<String>,
    /// Timestamp when this evidence item was recorded (immutable).
    pub recorded_at: u64,
}

impl EvidenceItem {
    /// Create a new evidence item.
    ///
    /// Once created, the item is immutable. The `recorded_at` timestamp
    /// is set at creation time and cannot be changed.
    pub fn new(
        evidence_id: impl Into<String>,
        workspace_id: impl Into<String>,
        title: impl Into<String>,
        source_type: EvidenceSourceType,
        content: impl Into<String>,
        recorded_at: u64,
    ) -> Self {
        Self {
            evidence_id: evidence_id.into(),
            workspace_id: workspace_id.into(),
            title: title.into(),
            source_type,
            content: content.into(),
            source_url: None,
            source_name: None,
            source_date: None,
            cited_by_thesis_versions: Vec::new(),
            superseded_by: None,
            recorded_at,
        }
    }

    /// Builder: set the source URL.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.source_url = Some(url.into());
        self
    }

    /// Builder: set the source name.
    pub fn with_source_name(mut self, name: impl Into<String>) -> Self {
        self.source_name = Some(name.into());
        self
    }

    /// Builder: set the source date.
    pub fn with_source_date(mut self, date: impl Into<String>) -> Self {
        self.source_date = Some(date.into());
        self
    }

    /// Record that a thesis version cites this evidence item.
    ///
    /// This is the only mutation allowed on an evidence item after creation —
    /// it records which thesis versions have cited this evidence, which is
    /// a forward reference that cannot be known at creation time.
    pub fn add_thesis_citation(&mut self, thesis_version: u32) {
        if !self.cited_by_thesis_versions.contains(&thesis_version) {
            self.cited_by_thesis_versions.push(thesis_version);
        }
    }

    /// Mark this evidence item as superseded by a later item.
    ///
    /// The original item is preserved; only the `superseded_by` field is set.
    /// This is the correct way to handle incorrect or outdated evidence —
    /// never delete or modify the original.
    pub fn mark_superseded_by(&mut self, newer_evidence_id: impl Into<String>) {
        self.superseded_by = Some(newer_evidence_id.into());
    }

    /// Whether this evidence item is still current (not superseded).
    pub fn is_current(&self) -> bool {
        self.superseded_by.is_none()
    }
}

// ── Evidence Dossier ──────────────────────────────────────────────────────────

/// A structured view of all evidence in a Workspace, organised by source type.
///
/// The dossier is a read-only view — it does not own the evidence items.
/// It is generated from the Workspace's evidence list on demand.
pub struct EvidenceDossier<'a> {
    pub workspace_id: &'a str,
    pub subject: &'a str,
    pub items: &'a [EvidenceItem],
}

impl<'a> EvidenceDossier<'a> {
    pub fn new(workspace_id: &'a str, subject: &'a str, items: &'a [EvidenceItem]) -> Self {
        Self { workspace_id, subject, items }
    }

    /// Return all evidence items of a specific source type.
    pub fn by_type(&self, source_type: &EvidenceSourceType) -> Vec<&EvidenceItem> {
        self.items.iter().filter(|e| &e.source_type == source_type).collect()
    }

    /// Return all current (non-superseded) evidence items.
    pub fn current(&self) -> Vec<&EvidenceItem> {
        self.items.iter().filter(|e| e.is_current()).collect()
    }

    /// Return all evidence items that cite a specific thesis version.
    pub fn cited_by_thesis(&self, version: u32) -> Vec<&EvidenceItem> {
        self.items
            .iter()
            .filter(|e| e.cited_by_thesis_versions.contains(&version))
            .collect()
    }

    /// Return a summary of evidence counts by source type.
    pub fn summary(&self) -> Vec<(String, usize)> {
        let types = [
            EvidenceSourceType::AnnualReport,
            EvidenceSourceType::EarningsCall,
            EvidenceSourceType::AiConversation,
            EvidenceSourceType::News,
            EvidenceSourceType::PersonalNote,
            EvidenceSourceType::FinancialData,
            EvidenceSourceType::SectorResearch,
            EvidenceSourceType::RegulatoryFiling,
            EvidenceSourceType::ExpertInterview,
            EvidenceSourceType::CommitteeMeeting,
        ];
        types
            .iter()
            .map(|t| {
                let count = self.by_type(t).len();
                (format!("{:?}", t), count)
            })
            .filter(|(_, count)| *count > 0)
            .collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(id: &str, source_type: EvidenceSourceType) -> EvidenceItem {
        EvidenceItem::new(id, "ws-001", "Test Evidence", source_type, "Content.", 1000)
    }

    #[test]
    fn evidence_item_is_current_by_default() {
        let item = make_item("ev-001", EvidenceSourceType::AnnualReport);
        assert!(item.is_current());
    }

    #[test]
    fn superseded_item_is_not_current() {
        let mut item = make_item("ev-001", EvidenceSourceType::AnnualReport);
        item.mark_superseded_by("ev-002");
        assert!(!item.is_current());
    }

    #[test]
    fn thesis_citation_is_recorded() {
        let mut item = make_item("ev-001", EvidenceSourceType::AnnualReport);
        item.add_thesis_citation(1);
        item.add_thesis_citation(2);
        item.add_thesis_citation(1); // duplicate — should not be added twice
        assert_eq!(item.cited_by_thesis_versions, vec![1, 2]);
    }

    #[test]
    fn dossier_filters_by_type() {
        let items = vec![
            make_item("ev-001", EvidenceSourceType::AnnualReport),
            make_item("ev-002", EvidenceSourceType::EarningsCall),
            make_item("ev-003", EvidenceSourceType::AnnualReport),
        ];
        let dossier = EvidenceDossier::new("ws-001", "Reliance", &items);
        assert_eq!(dossier.by_type(&EvidenceSourceType::AnnualReport).len(), 2);
        assert_eq!(dossier.by_type(&EvidenceSourceType::EarningsCall).len(), 1);
    }

    #[test]
    fn dossier_current_excludes_superseded() {
        let mut items = vec![
            make_item("ev-001", EvidenceSourceType::AnnualReport),
            make_item("ev-002", EvidenceSourceType::AnnualReport),
        ];
        items[0].mark_superseded_by("ev-002");
        let dossier = EvidenceDossier::new("ws-001", "Reliance", &items);
        assert_eq!(dossier.current().len(), 1);
        assert_eq!(dossier.current()[0].evidence_id, "ev-002");
    }
}