//! Champion Lifecycle Audit — domain-agnostic core module.
//!
//! Answers the single open question from Sprint 3.5D:
//!
//!   Best Ever ≠ Best Final — WHY?
//!
//! The audit tracks every genome that was ever the best external score seen,
//! recording the full chain of custody:
//!
//!   Discovery → Admission → Eviction → Reason
//!
//! ## What this module knows
//!
//! - Generation numbers
//! - External scores (opaque scalars)
//! - Which observer produced each score (`observer_id`)
//! - Proxy objective vectors (opaque `Vec<f64>`)
//! - Archive mechanics snapshots (`ChampionStatus`)
//!
//! ## What this module does NOT know
//!
//! - HC Coverage, HC Skills, HC Successions (INRC)
//! - Feasibility in any domain-specific sense
//! - Nurse rostering, VRP, trading, routing, or any domain concept
//! - Viability-aware dominance (not yet approved — awaiting Archive Forensics)
//!
//! ## Scientific Debt Ledger
//!
//! SD-003: Champion Retention Error implies Memory Failure
//!   Evidence: Best external champion lost in multiple runs (retention_error = 1)
//!   Unknown:  Admission failure vs eviction failure; dominance vs crowding vs archive limit
//!   Status:   UNRESOLVED — this module is the instrument to resolve it
//!
//! SD-004: Viability-aware archives may help
//!   Evidence: None
//!   Status:   HYPOTHESIS ONLY — no implementation permitted until Archive Forensics completes

/// Why a champion left the archive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExitReason {
    /// Discovered externally but never entered the archive.
    NeverAdmitted,
    /// A Pareto-dominating solution replaced it.
    Dominated,
    /// Archive capacity / crowding distance pressure removed it.
    Crowding,
    /// Hard archive size limit was hit; this member was dropped.
    ArchiveLimit,
    /// Evicted but the specific cause was not recorded by the adapter.
    Unknown,
}

impl std::fmt::Display for ExitReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NeverAdmitted => write!(f, "NeverAdmitted"),
            Self::Dominated => write!(f, "Dominated"),
            Self::Crowding => write!(f, "Crowding"),
            Self::ArchiveLimit => write!(f, "ArchiveLimit"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Full lifecycle record for one champion candidate.
///
/// Tracks the transition of a genome:
/// Discovery (Gen X) → Admitted (Gen Y) → Evicted (Gen Z) [Reason: R]
#[derive(Clone, Debug)]
pub struct ChampionLifecycle {
    /// Unique identifier assigned by [`ChampionTracker`].
    pub uid: u64,

    /// Generation at which this genome was first observed as the best external score.
    pub discovered_at: u64,

    /// Generation at which this genome entered the Pareto archive.
    /// `None` if it was never admitted.
    pub admitted_at: Option<u64>,

    /// Generation at which this genome left the Pareto archive.
    /// `None` if it is still in the archive.
    pub evicted_at: Option<u64>,

    /// Number of generations this genome spent inside the archive.
    /// Computed at eviction time or at run finalization.
    pub archive_lifetime: u64,

    /// Why this genome left the archive.
    /// `None` while it is still the active champion.
    pub exit_reason: Option<ExitReason>,

    /// Which external observer produced `external_score`.
    ///
    /// Examples:
    ///   - `"inrc_official_total"` — INRC total penalty (HC * 1000 + soft)
    ///   - `"inrc_soft_total"`     — INRC soft constraints only
    ///   - `"vrp_distance"`        — VRP total route distance
    ///   - `"trading_pnl"`         — Trading strategy P&L
    ///
    /// Stored as a plain `String` so the core crate remains domain-agnostic.
    /// The adapter supplies the identifier at observation time.
    ///
    /// Without this field, a score like `41395` in a log six months from now
    /// would be ambiguous between OfficialTotal, SoftTotal, or a custom observer.
    pub observer_id: String,

    /// External benchmark score at discovery (lower = better for penalty scores).
    pub external_score: f64,

    /// Proxy objective vector at discovery (O1..On).
    /// Length is determined by the domain adapter.
    pub objective_vector: Vec<f64>,

    /// Snapshot of archive mechanics at the moment of discovery.
    /// Populated by the adapter when available.
    pub status_at_discovery: Option<ChampionStatus>,
}

impl ChampionLifecycle {
    fn new(
        uid: u64,
        discovered_at: u64,
        observer_id: String,
        external_score: f64,
        objective_vector: Vec<f64>,
    ) -> Self {
        Self {
            uid,
            discovered_at,
            admitted_at: None,
            evicted_at: None,
            archive_lifetime: 0,
            exit_reason: None,
            observer_id,
            external_score,
            objective_vector,
            status_at_discovery: None,
        }
    }

    /// Returns true while this champion has not yet been diagnosed as lost.
    pub fn is_active(&self) -> bool {
        self.exit_reason.is_none()
    }

    fn record_admission(&mut self, generation: u64) {
        if self.admitted_at.is_none() {
            self.admitted_at = Some(generation);
        }
    }

    fn record_eviction(&mut self, generation: u64, reason: ExitReason) {
        if self.evicted_at.is_none() {
            self.evicted_at = Some(generation);
            self.exit_reason = Some(reason);
            self.archive_lifetime = match self.admitted_at {
                Some(entry) => generation.saturating_sub(entry),
                None => 0,
            };
        }
    }

    fn mark_never_admitted(&mut self, superseded_at: u64) {
        if self.admitted_at.is_none() && self.evicted_at.is_none() {
            self.evicted_at = Some(superseded_at);
            self.exit_reason = Some(ExitReason::NeverAdmitted);
            self.archive_lifetime = 0;
        }
    }
}

impl std::fmt::Display for ChampionLifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Champion(uid={uid}, disc={disc}, observer={obs}, ext={score:.1}, admitted={adm:?}, evicted={ev:?}, lifetime={lt}, reason={reason})",
            uid = self.uid,
            disc = self.discovered_at,
            obs = self.observer_id,
            score = self.external_score,
            adm = self.admitted_at,
            ev = self.evicted_at,
            lt = self.archive_lifetime,
            reason = self
                .exit_reason
                .as_ref()
                .map(|r| r.to_string())
                .unwrap_or_else(|| "Active".to_string()),
        )
    }
}

/// Snapshot of archive mechanics at the moment a champion is discovered.
///
/// This single snapshot often reveals the problem immediately:
///
/// ```text
/// Best External Found → Pareto Rank = 7, Crowding = 0.01 → Removed 3 gens later
/// ```
/// points to archive mechanics, whereas:
/// ```text
/// Best External Found → Viability = 320, Pareto Rank = 1
/// ```
/// points to feasibility.
#[derive(Clone, Debug)]
pub struct ChampionStatus {
    pub uid: u64,

    /// Whether the genome satisfies all hard constraints at discovery.
    /// Populated by the adapter using domain-specific feasibility logic.
    pub feasible: bool,

    /// A scalar viability score supplied by the adapter (0.0 = fully feasible).
    /// The adapter defines the scale; the core stores it opaquely.
    pub viability_score: f64,

    /// Whether this genome was present in the Pareto archive at discovery.
    pub archive_member: bool,

    /// Pareto rank at discovery (0 = non-dominated front).
    pub pareto_rank: usize,

    /// Crowding distance at discovery.
    pub crowding_distance: f64,
}

/// Tracks all champion lifecycle records across a full run.
///
/// The adapter calls [`observe`] every generation with the current
/// best-external-score genome. The tracker handles UID assignment,
/// supersession detection, and retention error counting.
pub struct ChampionTracker {
    /// All lifecycle records, in order of discovery.
    pub records: Vec<ChampionLifecycle>,

    next_uid: u64,

    /// Index into `records` of the currently active champion.
    active_idx: Option<usize>,

    /// Best external score seen across the entire run.
    pub best_external_ever: f64,

    /// Best external score present in the final archive at run end.
    pub best_external_final: f64,

    /// Number of champions whose best-ever score was better than the final
    /// archive best but are no longer in the archive.
    pub retention_error_count: u64,
}

impl ChampionTracker {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            next_uid: 0,
            active_idx: None,
            best_external_ever: f64::MAX,
            best_external_final: f64::MAX,
            retention_error_count: 0,
        }
    }

    /// Observe a candidate genome this generation.
    ///
    /// Parameters:
    /// - `generation`      : current generation number
    /// - `observer_id`     : which external observer produced this score
    ///                       (e.g. `"inrc_official_total"`, `"vrp_distance"`)
    /// - `external_score`  : external benchmark score (lower = better)
    /// - `objective_vector`: proxy Pareto objectives
    /// - `in_archive`      : whether this genome is currently in the archive
    /// - `status`          : optional archive-mechanics snapshot
    ///
    /// Returns:
    /// - `(uid, true)`  when a **new** champion record was created (new best-ever score).
    /// - `(uid, false)` when the existing active champion record was updated (not a new best).
    ///
    /// The boolean flag lets adapters distinguish "new champion genome" from
    /// "existing champion updated" — critical for genome-hash → tracker-uid mapping.
    pub fn observe(
        &mut self,
        generation: u64,
        observer_id: impl Into<String>,
        external_score: f64,
        objective_vector: Vec<f64>,
        in_archive: bool,
        status: Option<ChampionStatus>,
    ) -> (u64, bool) {
        if external_score < self.best_external_ever {
            // New best — supersede the previous active champion
            self.best_external_ever = external_score;

            if let Some(idx) = self.active_idx {
                let prev = &mut self.records[idx];
                if prev.is_active() {
                    prev.mark_never_admitted(generation);
                }
            }

            let uid = self.next_uid;
            self.next_uid += 1;

            let mut record = ChampionLifecycle::new(
                uid,
                generation,
                observer_id.into(),
                external_score,
                objective_vector,
            );
            record.status_at_discovery = status;

            if in_archive {
                record.record_admission(generation);
            }

            self.records.push(record);
            self.active_idx = Some(self.records.len() - 1);
            (uid, true)
        } else {
            // Not a new best — update admission status of current champion if needed
            if let Some(idx) = self.active_idx {
                let rec = &mut self.records[idx];
                if in_archive && rec.admitted_at.is_none() {
                    rec.record_admission(generation);
                }
            }
            let uid = self.active_idx.map(|i| self.records[i].uid).unwrap_or(0);
            (uid, false)
        }
    }

    /// Called by the adapter when a champion genome is evicted from the archive.
    pub fn notify_eviction(&mut self, uid: u64, generation: u64, reason: ExitReason) {
        if let Some(rec) = self.records.iter_mut().find(|r| r.uid == uid) {
            rec.record_eviction(generation, reason);
        }
    }

    /// Called at run end with the best external score present in the final archive.
    pub fn finalize(&mut self, best_external_in_final_archive: f64) {
        self.best_external_final = best_external_in_final_archive;
        for rec in &self.records {
            if rec.external_score < best_external_in_final_archive && !rec.is_active() {
                self.retention_error_count += 1;
            }
        }
    }

    /// Print a human-readable summary to stdout.
    pub fn print_summary(&self) {
        println!("=== Champion Lifecycle Audit ===");
        println!("  Best External Ever  : {:.1}", self.best_external_ever);
        println!("  Best External Final : {:.1}", self.best_external_final);
        println!("  Retention Errors    : {}", self.retention_error_count);
        println!("  Total Champions     : {}", self.records.len());
        println!();

        let (mut never, mut dominated, mut crowding, mut limit, mut unknown, mut active) =
            (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);

        for rec in &self.records {
            match &rec.exit_reason {
                None => active += 1,
                Some(ExitReason::NeverAdmitted) => never += 1,
                Some(ExitReason::Dominated) => dominated += 1,
                Some(ExitReason::Crowding) => crowding += 1,
                Some(ExitReason::ArchiveLimit) => limit += 1,
                Some(ExitReason::Unknown) => unknown += 1,
            }
        }

        println!("  Exit Reason Breakdown:");
        println!("    Active (current)   : {}", active);
        println!("    NeverAdmitted      : {}", never);
        println!("    Dominated          : {}", dominated);
        println!("    Crowding           : {}", crowding);
        println!("    ArchiveLimit       : {}", limit);
        println!("    Unknown            : {}", unknown);
        println!();

        println!("  Individual Records:");
        for rec in &self.records {
            println!("    {}", rec);
            if let Some(ref s) = rec.status_at_discovery {
                println!(
                    "      Status: feasible={}, viability={:.1}, in_archive={}, rank={}, crowding={:.4}",
                    s.feasible,
                    s.viability_score,
                    s.archive_member,
                    s.pareto_rank,
                    s.crowding_distance
                );
            }
        }
    }

    /// Serialize to JSON lines for logging/export.
    pub fn to_json_lines(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            r#"{{"type":"champion_summary","best_ever":{:.1},"best_final":{:.1},"retention_errors":{}}}"#,
            self.best_external_ever, self.best_external_final, self.retention_error_count
        ));
        for rec in &self.records {
            let admitted = rec
                .admitted_at
                .map(|g| g.to_string())
                .unwrap_or_else(|| "null".to_string());
            let evicted = rec
                .evicted_at
                .map(|g| g.to_string())
                .unwrap_or_else(|| "null".to_string());
            let reason = rec
                .exit_reason
                .as_ref()
                .map(|r| r.to_string())
                .unwrap_or_else(|| "Active".to_string());
            let proxy: Vec<String> = rec
                .objective_vector
                .iter()
                .map(|v| format!("{:.4}", v))
                .collect();
            let status_json = rec.status_at_discovery.as_ref().map(|s| {
                format!(
                    r#","feasible":{},"viability":{:.1},"in_archive":{},"rank":{},"crowding":{:.4}"#,
                    s.feasible, s.viability_score, s.archive_member, s.pareto_rank, s.crowding_distance
                )
            }).unwrap_or_default();
            lines.push(format!(
                r#"{{"type":"champion","uid":{},"disc":{},"observer":"{}","ext":{:.1},"admitted":{},"evicted":{},"lifetime":{},"reason":"{}","proxy":[{}]{}}}"#,
                rec.uid, rec.discovered_at, rec.observer_id, rec.external_score,
                admitted, evicted, rec.archive_lifetime, reason,
                proxy.join(","), status_json,
            ));
        }
        lines.join("\n")
    }
}

impl Default for ChampionTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_never_admitted() {
        let mut tracker = ChampionTracker::new();

        // Gen 100: best score 41395, NOT in archive
        let (uid0, is_new0) = tracker.observe(
            100,
            "inrc_official_total",
            41395.0,
            vec![0.8, 0.2, 0.5, 0.3, 0.6],
            false,
            None,
        );
        assert_eq!(uid0, 0);
        assert!(is_new0);
        assert_eq!(tracker.records.len(), 1);
        assert_eq!(tracker.records[0].admitted_at, None);
        assert_eq!(tracker.records[0].observer_id, "inrc_official_total");
        assert!(tracker.records[0].is_active());

        // Gen 200: new best 40000, in archive — supersedes previous
        let (uid1, is_new1) = tracker.observe(
            200,
            "inrc_official_total",
            40000.0,
            vec![0.9, 0.1, 0.4, 0.2, 0.7],
            true,
            None,
        );
        assert_eq!(uid1, 1);
        assert!(is_new1);
        assert_eq!(tracker.records.len(), 2);
        assert_eq!(
            tracker.records[0].exit_reason,
            Some(ExitReason::NeverAdmitted)
        );
        assert_eq!(tracker.records[0].evicted_at, Some(200));
        assert_eq!(tracker.records[0].archive_lifetime, 0);
    }

    #[test]
    fn test_observe_returns_false_for_non_new_best() {
        let mut tracker = ChampionTracker::new();
        let (uid0, is_new0) = tracker.observe(
            50,
            "inrc_official_total",
            5000.0,
            vec![0.7, 0.3],
            false,
            None,
        );
        assert_eq!(uid0, 0);
        assert!(is_new0);

        // Same score — not a new best
        let (uid1, is_new1) = tracker.observe(
            51,
            "inrc_official_total",
            5000.0,
            vec![0.7, 0.3],
            false,
            None,
        );
        assert_eq!(uid1, 0); // returns active champion uid
        assert!(!is_new1);

        // Worse score — not a new best
        let (uid2, is_new2) = tracker.observe(
            52,
            "inrc_official_total",
            6000.0,
            vec![0.8, 0.2],
            false,
            None,
        );
        assert_eq!(uid2, 0);
        assert!(!is_new2);
    }

    #[test]
    fn test_observer_id_preserved() {
        let mut tracker = ChampionTracker::new();
        tracker.observe(50, "inrc_soft_total", 5000.0, vec![0.7, 0.3], false, None);
        assert_eq!(tracker.records[0].observer_id, "inrc_soft_total");

        // New best with different observer
        tracker.observe(
            100,
            "inrc_official_total",
            4000.0,
            vec![0.6, 0.4],
            true,
            None,
        );
        assert_eq!(tracker.records[1].observer_id, "inrc_official_total");
    }

    #[test]
    fn test_retention_error_counting() {
        let mut tracker = ChampionTracker::new();

        // Champion A: score 37130, never admitted
        tracker.observe(
            50,
            "inrc_official_total",
            37130.0,
            vec![0.7, 0.3],
            false,
            None,
        );
        // Champion B: new best 36000, in archive
        tracker.observe(
            5000,
            "inrc_official_total",
            36000.0,
            vec![0.5, 0.5],
            true,
            None,
        );

        // A was superseded at gen 5000 with NeverAdmitted
        tracker.finalize(36000.0);

        // A had score 37130 > 36000 (final best), so NOT a retention error
        assert_eq!(tracker.retention_error_count, 0);
        assert_eq!(tracker.best_external_ever, 36000.0);
    }

    #[test]
    fn test_retention_error_real_case() {
        // Mirrors the observed run: Best Ever = 37130, Best Final = 43925
        let mut tracker = ChampionTracker::new();

        // Champion A: score 37130, admitted at gen 50
        tracker.observe(
            50,
            "inrc_official_total",
            37130.0,
            vec![0.7, 0.3],
            true,
            None,
        );
        // Evicted at gen 300 (reason unknown)
        tracker.notify_eviction(0, 300, ExitReason::Unknown);

        // No new champion found — finalize with worse score
        tracker.finalize(43925.0);

        // A had score 37130 < 43925 and is not active → retention error
        assert_eq!(tracker.retention_error_count, 1);
    }

    #[test]
    fn test_archive_lifetime_computed() {
        let mut tracker = ChampionTracker::new();
        tracker.observe(
            100,
            "inrc_official_total",
            28000.0,
            vec![0.9, 0.1],
            true,
            None,
        );
        tracker.notify_eviction(0, 600, ExitReason::Dominated);
        assert_eq!(tracker.records[0].archive_lifetime, 500);
        assert_eq!(tracker.records[0].exit_reason, Some(ExitReason::Dominated));
    }

    #[test]
    fn test_champion_status_snapshot() {
        let mut tracker = ChampionTracker::new();
        let status = ChampionStatus {
            uid: 0,
            feasible: false,
            viability_score: 320.0,
            archive_member: true,
            pareto_rank: 1,
            crowding_distance: 0.85,
        };
        tracker.observe(
            200,
            "inrc_official_total",
            29000.0,
            vec![0.6, 0.4],
            true,
            Some(status),
        );
        let rec = &tracker.records[0];
        let s = rec.status_at_discovery.as_ref().unwrap();
        assert!(!s.feasible);
        assert_eq!(s.pareto_rank, 1);
        assert_eq!(s.viability_score, 320.0);
    }

    #[test]
    fn test_json_lines_contains_observer_id() {
        let mut tracker = ChampionTracker::new();
        tracker.observe(
            100,
            "inrc_official_total",
            41395.0,
            vec![0.8, 0.2],
            false,
            None,
        );
        let json = tracker.to_json_lines();
        assert!(json.contains(r#""observer":"inrc_official_total""#));
        assert!(json.contains(r#""ext":41395.0"#));
    }
}
