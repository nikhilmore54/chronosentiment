use chronosentiment_adapter::product::intraday_decision::*;
fn main() {
    let dataset = "datasets/today_live_dataset.json";
    let briefs = load_intraday_briefs(dataset).unwrap();
    let mut acts = 0;
    let mut avoids = 0;
    for b in briefs {
        if b.entry_action == "ACT" { acts += 1; }
        else if b.entry_action == "AVOID" { avoids += 1; }
    }
    println!("ACT: {}, AVOID: {}", acts, avoids);
}
