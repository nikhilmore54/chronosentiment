//! Markdown report emitter.

use super::types::PipelineReport;

pub fn render_markdown(r: &PipelineReport) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("# RP-608-WL-IMPROVE-001 — Crypto Plan Discovery v2 Report".into());
    lines.push(String::new());
    lines.push(format!("**Protocol:** `{}`", r.protocol));
    lines.push(format!("**Version:** {}", r.version));
    lines.push(format!(
        "**Authority:** {} · baseline_frozen={} · gate2_criteria_changed={}",
        r.authority, r.baseline_frozen, r.gate2_criteria_changed
    ));
    lines.push(String::new());
    lines.push("## Research conclusion".into());
    lines.push(String::new());
    lines.push(r.research_conclusion.clone());
    lines.push(String::new());
    lines.push(format!("**Gate 2 disposition:** {}", r.gate2_disposition));
    lines.push(String::new());

    lines.push("## 1. Data integrity".into());
    lines.push(String::new());
    lines.push(format!("integrity_pass = **{}**", r.integrity_pass));
    for n in &r.integrity_notes {
        lines.push(format!("- `{n}`"));
    }
    lines.push(String::new());

    lines.push("## 2. Feature availability matrix".into());
    lines.push(String::new());
    lines.push(
        "| Feature | Family | Status | Disc present/miss | Hold present/miss | Distinct | Consistent | Notes |"
            .into(),
    );
    lines.push("| --- | --- | --- | ---: | ---: | ---: | --- | --- |".into());
    for f in &r.feature_availability {
        lines.push(format!(
            "| `{}` | {} | **{}** | {}/{} | {}/{} | {} | {} | {} |",
            f.feature_id,
            f.family,
            f.status,
            f.discovery_present_n,
            f.discovery_missing_n,
            f.holdout_present_n,
            f.holdout_missing_n,
            f.n_distinct_discovery,
            f.definition_consistent_across_windows,
            f.notes.replace('|', "/")
        ));
    }
    lines.push(String::new());

    lines.push("## 3. Session composition".into());
    lines.push(String::new());
    for s in &r.session_composition {
        lines.push(format!(
            "### {} — n={} WR={:?}% LONG_share={:.1}% OF_unavail={:.1}%",
            s.window,
            s.n,
            s.wr_pct,
            s.long_share_pct,
            s.of_unavailable_share_pct
        ));
        lines.push(String::new());
        lines.push("| Day | n | WR% |".into());
        lines.push("| --- | ---: | ---: |".into());
        for (d, st) in &s.per_day {
            lines.push(format!(
                "| {} | {} | {:?} |",
                d, st.n, st.win_rate_pct
            ));
        }
        lines.push(String::new());
    }

    lines.push("## 4. v1 lineage registry (immutable)".into());
    lines.push(String::new());
    for entry in &r.lineage_registry {
        lines.push(format!(
            "- **{}** `{}` — holdout **{}** ({}) · rule: {}",
            entry.lineage_id,
            entry.original_feature,
            entry.holdout_result,
            entry.failure_reason,
            entry.future_analysis_rule
        ));
    }
    lines.push(String::new());

    lines.push("## 5. Structured discovery (predeclared families)".into());
    lines.push(String::new());
    lines.push(
        "| Candidate | Family | High/Low WR (n) | Gap pp | Flags |"
            .into(),
    );
    lines.push("| --- | --- | --- | ---: | --- |".into());
    for c in &r.candidates {
        lines.push(format!(
            "| `{}` | {} | {:.2?}/{:.2?} ({}/{}) | {:?} | {} |",
            c.candidate_id,
            c.family,
            c.discovery.high.win_rate_pct,
            c.discovery.low.win_rate_pct,
            c.discovery.high.n,
            c.discovery.low.n,
            c.discovery.wr_gap_pp,
            c.flags.join(",")
        ));
    }
    lines.push(String::new());

    lines.push("## 6. Stability / redundancy".into());
    lines.push(String::new());
    lines.push(
        "| Candidate | Assoc | Stable | Simpson | SingleSess | Promote | Blocks |"
            .into(),
    );
    lines.push("| --- | --- | --- | --- | --- | --- | --- |".into());
    for s in &r.stability {
        lines.push(format!(
            "| `{}` | {} | {} | {} | {} | **{}** | {} |",
            s.candidate_id,
            s.association_ok,
            s.stability_ok,
            s.pooled_vs_session_simpson_risk,
            s.single_session_dominant,
            s.promote_to_compiler,
            s.block_reasons.join(",")
        ));
    }
    lines.push(String::new());

    lines.push("## 7. Hypothesis compiler output".into());
    lines.push(String::new());
    if r.hypotheses.is_empty() {
        lines.push(
            "_No hypotheses promoted through stability → compiler. This is a valid outcome._"
                .into(),
        );
        lines.push(String::new());
    } else {
        for h in &r.hypotheses {
            lines.push(format!("### {}", h.hypothesis_id));
            lines.push(String::new());
            lines.push(format!("- **gate_status:** {}", h.gate_status));
            lines.push(format!("- **feature:** {}", h.feature));
            lines.push(format!("- **comparison:** {}", h.comparison));
            lines.push(format!("- **expected:** {}", h.expected_direction));
            lines.push(format!("- **replication_verdict:** {}", h.replication_verdict));
            lines.push(format!("- **falsify if:** {}", h.falsification_condition));
            lines.push(format!("- **limitations:** {}", h.known_limitations.join("; ")));
            lines.push(String::new());
        }
    }

    lines.push("### Research-only blocked records (transparency; not Gate 2)".into());
    lines.push(String::new());
    if r.research_only_blocked_records.is_empty() {
        lines.push("_None._".into());
        lines.push(String::new());
    } else {
        lines.push(
            "| ID | Feature | Disc gap | Repl verdict | Primary block |"
                .into(),
        );
        lines.push("| --- | --- | ---: | --- | --- |".into());
        for h in &r.research_only_blocked_records {
            let block = h
                .known_limitations
                .first()
                .cloned()
                .unwrap_or_else(|| "—".into());
            lines.push(format!(
                "| `{}` | {} | {:?} | {} | {} |",
                h.hypothesis_id,
                h.feature,
                h.discovery_evidence.wr_gap_pp,
                h.replication_verdict,
                block
            ));
        }
        lines.push(String::new());
    }

    lines.push("## 8. Gate 2 assessment (criteria unchanged)".into());
    lines.push(String::new());
    if r.gate2_assessments.is_empty() {
        lines.push(
            "No Gate 2 assessments — no compiler-promoted hypotheses to evaluate.".into(),
        );
        lines.push(String::new());
        lines.push("Unmet by absence:".into());
        lines.push("- G2-1 measurable hypothesis with discovery+replication support".into());
        lines.push("- G2-4 holdout replication".into());
        lines.push(String::new());
    } else {
        for a in &r.gate2_assessments {
            lines.push(format!(
                "### {} — {}",
                a.hypothesis_id, a.disposition
            ));
            lines.push(String::new());
            lines.push("| Criterion | Satisfied | Detail |".into());
            lines.push("| --- | --- | --- |".into());
            for c in &a.criteria {
                lines.push(format!(
                    "| `{}` | {} | {} |",
                    c.criterion_id, c.satisfied, c.detail.replace('|', "/")
                ));
            }
            lines.push(String::new());
        }
    }

    lines.push("## Governance".into());
    lines.push(String::new());
    lines.push("- Baseline frozen · OF ±0.30 frozen · Gate 3 unauthorized".into());
    lines.push("- Other books deferred · execution authority unchanged".into());
    lines.push("- v1 discovery/replication artifacts remain immutable evidence".into());
    lines.push("- `.cursor/rules/chronosentiment-core.mdc` §3b–3c · `.cursor/rules/rp608-ofx.mdc`".into());
    lines.push(String::new());
    lines.join("\n")
}
