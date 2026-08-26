// ChronoSentiment — Research/Decision Timeline
//
// The Timeline is a complete, chronological record of how the investor's
// thinking about a company evolved. Every state transition is recorded.
//
// Platform primitive: Timeline
// ChronoSentiment realisation: ResearchTimeline / DecisionTimeline
//
// The Timeline is built from TimelineEvents emitted by the Workspace.
// It is append-only — events are never removed or modified.

use serde::{Deserialize, Serialize};

// ── Timeline Event Kind ───────────────────────────────────────────────────────

/// The kind of event recorded in the timeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimelineEventKind {
    // Workspace lifecycle
    WorkspaceOpened,
    WorkspaceClosed,
    // Evidence
    EvidenceAdded,
    // Thesis
    ThesisCreated,
    ThesisRevised,
    ThesisInvalidated,
    // Review
    ReviewScheduled,
    ReviewCompleted,
    // Decision / Position
    DecisionMade,
    PositionOpened,
    PositionMonitored,
    PositionClosed,
    // Outcome
    OutcomeRecorded,
    // Learning
    LearningCaptured,
    InsightAdded,
}

impl TimelineEventKind {
    /// Human-readable label for display.
    pub fn label(&self) -> &'static str {
        match self {
            TimelineEventKind::WorkspaceOpened => "Workspace Opened",
            TimelineEventKind::WorkspaceClosed => "Workspace Closed",
            TimelineEventKind::EvidenceAdded => "Evidence Added",
            TimelineEventKind::ThesisCreated => "Thesis Created",
            TimelineEventKind::ThesisRevised => "Thesis Revised",
            TimelineEventKind::ThesisInvalidated => "Thesis Invalidated",
            TimelineEventKind::ReviewScheduled => "Review Scheduled",
            TimelineEventKind::ReviewCompleted => "Review Completed",
            TimelineEventKind::DecisionMade => "Decision Made",
            TimelineEventKind::PositionOpened => "Position Opened",
            TimelineEventKind::PositionMonitored => "Position Monitored",
            TimelineEventKind::PositionClosed => "Position Closed",
            TimelineEventKind::OutcomeRecorded => "Outcome Recorded",
            TimelineEventKind::LearningCaptured => "Learning Captured",
            TimelineEventKind::InsightAdded => "Insight Added",
        }
    }
}

// ── Timeline Event ────────────────────────────────────────────────────────────

/// A single immutable event in the Research/Decision Timeline.
///
/// Events are append-only — once recorded, they cannot be modified or removed.
/// The timeline provides a complete audit trail of how the investment thinking
/// evolved from initial research through outcome and learning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub event_id: String,
    pub workspace_id: String,
    pub kind: TimelineEventKind,
    pub description: String,
    pub timestamp: u64,
}

// ── Timeline View ─────────────────────────────────────────────────────────────

/// A structured, queryable view of the Research/Decision Timeline.
///
/// The TimelineView is a read-only projection of the Workspace's event log.
/// It provides filtering, grouping, and summary capabilities.
pub struct TimelineView<'a> {
    pub workspace_id: &'a str,
    pub subject: &'a str,
    events: &'a [TimelineEvent],
}

impl<'a> TimelineView<'a> {
    pub fn new(workspace_id: &'a str, subject: &'a str, events: &'a [TimelineEvent]) -> Self {
        Self {
            workspace_id,
            subject,
            events,
        }
    }

    /// Return all events in chronological order.
    pub fn all(&self) -> &[TimelineEvent] {
        self.events
    }

    /// Return events of a specific kind.
    pub fn by_kind(&self, kind: &TimelineEventKind) -> Vec<&TimelineEvent> {
        self.events.iter().filter(|e| &e.kind == kind).collect()
    }

    /// Return events within a timestamp range [from, to].
    pub fn in_range(&self, from: u64, to: u64) -> Vec<&TimelineEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .collect()
    }

    /// Return events after a given timestamp.
    pub fn since(&self, timestamp: u64) -> Vec<&TimelineEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp > timestamp)
            .collect()
    }

    /// Return the most recent event of a specific kind.
    pub fn latest_of_kind(&self, kind: &TimelineEventKind) -> Option<&TimelineEvent> {
        self.events.iter().rev().find(|e| &e.kind == kind)
    }

    /// Return a summary of event counts by kind.
    pub fn summary(&self) -> Vec<(String, usize)> {
        let kinds = [
            TimelineEventKind::WorkspaceOpened,
            TimelineEventKind::EvidenceAdded,
            TimelineEventKind::ThesisCreated,
            TimelineEventKind::ThesisRevised,
            TimelineEventKind::ReviewCompleted,
            TimelineEventKind::DecisionMade,
            TimelineEventKind::OutcomeRecorded,
            TimelineEventKind::LearningCaptured,
            TimelineEventKind::InsightAdded,
        ];
        kinds
            .iter()
            .map(|k| (k.label().to_string(), self.by_kind(k).len()))
            .filter(|(_, count)| *count > 0)
            .collect()
    }

    /// Generate a human-readable timeline narrative.
    pub fn narrative(&self) -> Vec<String> {
        self.events
            .iter()
            .map(|e| format!("[t={}] {}: {}", e.timestamp, e.kind.label(), e.description))
            .collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(id: &str, kind: TimelineEventKind, ts: u64) -> TimelineEvent {
        TimelineEvent {
            event_id: id.to_string(),
            workspace_id: "ws-001".to_string(),
            kind,
            description: format!("Event {}", id),
            timestamp: ts,
        }
    }

    #[test]
    fn timeline_view_filters_by_kind() {
        let events = vec![
            make_event("e1", TimelineEventKind::WorkspaceOpened, 1000),
            make_event("e2", TimelineEventKind::EvidenceAdded, 1001),
            make_event("e3", TimelineEventKind::EvidenceAdded, 1002),
            make_event("e4", TimelineEventKind::ThesisCreated, 1003),
        ];
        let view = TimelineView::new("ws-001", "Reliance", &events);
        assert_eq!(view.by_kind(&TimelineEventKind::EvidenceAdded).len(), 2);
        assert_eq!(view.by_kind(&TimelineEventKind::ThesisCreated).len(), 1);
    }

    #[test]
    fn timeline_view_in_range() {
        let events = vec![
            make_event("e1", TimelineEventKind::WorkspaceOpened, 1000),
            make_event("e2", TimelineEventKind::EvidenceAdded, 2000),
            make_event("e3", TimelineEventKind::ThesisCreated, 3000),
        ];
        let view = TimelineView::new("ws-001", "Reliance", &events);
        let range = view.in_range(1500, 2500);
        assert_eq!(range.len(), 1);
        assert_eq!(range[0].event_id, "e2");
    }

    #[test]
    fn timeline_view_latest_of_kind() {
        let events = vec![
            make_event("e1", TimelineEventKind::EvidenceAdded, 1000),
            make_event("e2", TimelineEventKind::EvidenceAdded, 2000),
        ];
        let view = TimelineView::new("ws-001", "Reliance", &events);
        let latest = view
            .latest_of_kind(&TimelineEventKind::EvidenceAdded)
            .unwrap();
        assert_eq!(latest.event_id, "e2");
    }

    #[test]
    fn timeline_narrative_is_chronological() {
        let events = vec![
            make_event("e1", TimelineEventKind::WorkspaceOpened, 1000),
            make_event("e2", TimelineEventKind::ThesisCreated, 1001),
        ];
        let view = TimelineView::new("ws-001", "Reliance", &events);
        let narrative = view.narrative();
        assert_eq!(narrative.len(), 2);
        assert!(narrative[0].contains("Workspace Opened"));
        assert!(narrative[1].contains("Thesis Created"));
    }
}
