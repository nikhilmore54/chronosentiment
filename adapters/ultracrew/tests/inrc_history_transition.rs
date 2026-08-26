use std::path::PathBuf;
use std::sync::Arc;
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::history::extract_next_history;
use ultracrew::inrc::optimization::{InrcContext, InrcGenome, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};

#[test]
fn test_history_transition() {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");

    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();

    let ecology = WorkforceEcology::new();
    let context = InrcContext::new(scenario, week_data, history, ecology);
    let ctx_arc = Arc::new(context);

    // Let's create a specific genome for Nurse HN_0
    // Mon: Early (shift 0), Tue: Early, Wed: Off, Thu: Off, Fri: Late (shift 2), Sat: Night (shift 3), Sun: Off

    let total_bits = ctx_arc.num_nurses * ctx_arc.num_days * ctx_arc.shift_types.len();
    let mut bits = vec![false; total_bits];

    // Function to set assignment
    let mut set_assign = |n: usize, d: usize, s: usize| {
        let idx =
            n * (ctx_arc.num_days * ctx_arc.shift_types.len()) + d * ctx_arc.shift_types.len() + s;
        bits[idx] = true;
    };

    set_assign(0, 0, 0); // Mon Early
    set_assign(0, 1, 0); // Tue Early
    set_assign(0, 4, 2); // Fri Late
    set_assign(0, 5, 3); // Sat Night

    // Let's also check a nurse who works ALL weekend
    // Nurse HN_1 works Sat and Sun
    set_assign(1, 5, 0); // Sat
    set_assign(1, 6, 0); // Sun

    // Let's check a nurse who works NO weekend but had weekend in history

    let genome = InrcGenome { bits };
    let next_hist = extract_next_history(&ctx_arc, &genome);

    let n0 = next_hist
        .nurse_history
        .iter()
        .find(|nh| nh.nurse == "HN_0")
        .unwrap();
    let n1 = next_hist
        .nurse_history
        .iter()
        .find(|nh| nh.nurse == "HN_1")
        .unwrap();

    // Mon: E, Tue: E, Wed: Off, Thu: Off, Fri: L, Sat: N, Sun: Off
    // At end of Sunday (Day 6):
    // Off, so consecutive days off = 1
    // consecutive working days = 0
    // last shift type: Night (since Sat was N, Sun was Off, it retains the last worked shift, which was Night)
    // consecutive shift types = 0 (since it was broken by Day Off on Sun)

    assert_eq!(n0.number_of_consecutive_days_off, 1, "n0 consec days off");
    assert_eq!(n0.number_of_consecutive_working_days, 0, "n0 consec work");
    assert_eq!(n0.last_assigned_shift_type, "Night", "n0 last shift");
    assert_eq!(
        n0.number_of_consecutive_assignments, 0,
        "n0 consec assignments"
    );
    assert_eq!(
        n0.number_of_working_weekends,
        ctx_arc.history.nurse_history[0].number_of_working_weekends + 1,
        "n0 worked weekend because Sat was worked"
    );
    assert_eq!(
        n0.number_of_assignments,
        ctx_arc.history.nurse_history[0].number_of_assignments + 4,
        "n0 assignments"
    );

    // HN_1 works Sat and Sun. End of Sunday:
    // consecutive days off = 0
    // consecutive working days = 2 (or more if they worked Fri, but they didn't)
    // last shift = Early
    // consecutive shift types = 2
    assert_eq!(n1.number_of_consecutive_days_off, 0, "n1 consec days off");
    assert_eq!(n1.number_of_consecutive_working_days, 2, "n1 consec work");
    assert_eq!(n1.last_assigned_shift_type, "Early", "n1 last shift");
    assert_eq!(
        n1.number_of_consecutive_assignments, 2,
        "n1 consec assignments"
    );
    assert_eq!(
        n1.number_of_working_weekends,
        ctx_arc.history.nurse_history[1].number_of_working_weekends + 1,
        "n1 worked weekend"
    );
    assert_eq!(
        n1.number_of_assignments,
        ctx_arc.history.nurse_history[1].number_of_assignments + 2,
        "n1 assignments"
    );
}
