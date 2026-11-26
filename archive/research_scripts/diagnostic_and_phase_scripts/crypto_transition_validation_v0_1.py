import pandas as pd
import numpy as np
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_FILE = WORKSPACE / "datasets" / "crypto_24h" / "transition_discovery_v0_1_BTCUSDT.csv"
REPORT_FILE = WORKSPACE / "datasets" / "crypto_24h" / "transition_validation_v0_1.md"

def df_to_markdown(df):
    if len(df) == 0:
        return ""
    headers = list(df.columns)
    markdown = "| " + " | ".join(headers) + " |\n"
    markdown += "|---" * len(headers) + "|\n"
    for _, row in df.iterrows():
        row_strs = []
        for x in row:
            if isinstance(x, float):
                row_strs.append(f"{x:.4f}")
            else:
                row_strs.append(str(x))
        markdown += "| " + " | ".join(row_strs) + " |\n"
    return markdown

def run_validation():
    df = pd.read_csv(DATA_FILE)
    n_total = len(df)
    n_train = int(n_total * 0.6)
    n_val = int(n_total * 0.8)
    
    train = df.iloc[:n_train].copy()
    val = df.iloc[n_train:n_val].copy()
    
    # Derive threshold strictly from Train
    vol_median = train['volatility_1m_std_24h'].median()
    
    def get_vol_transition(row):
        past_high = row['past_volatility_24h'] > vol_median
        curr_high = row['volatility_1m_std_24h'] > vol_median
        if not past_high and curr_high: return "LOW -> HIGH"
        if past_high and not curr_high: return "HIGH -> LOW"
        if past_high and curr_high: return "MAINTAIN_HIGH"
        return "MAINTAIN_LOW"
        
    def get_dir_transition(row):
        past_dir = row['past_trend_dir']
        curr_dir = row['trend_dir']
        if past_dir == 1 and curr_dir == -1: return "UP -> DOWN"
        if past_dir == -1 and curr_dir == 1: return "DOWN -> UP"
        if past_dir == 1 and curr_dir == 1: return "MAINTAIN_UP"
        if past_dir == -1 and curr_dir == -1: return "MAINTAIN_DOWN"
        return "OTHER"
        
    val['vol_trans'] = val.apply(get_vol_transition, axis=1)
    val['dir_trans'] = val.apply(get_dir_transition, axis=1)
    val['block'] = val['timestamp'] // 43200000
    
    hypotheses = [
        {"id": "T1", "trans_type": "vol_trans", "trans": "HIGH -> LOW", "target": "vol_expansion_300m"},
        {"id": "T2", "trans_type": "vol_trans", "trans": "HIGH -> LOW", "target": "excursion_exceeded_300m"},
        {"id": "T3", "trans_type": "dir_trans", "trans": "MAINTAIN_UP", "target": "excursion_exceeded_300m"}
    ]
    
    with open(REPORT_FILE, "w") as f:
        f.write("# CRYPTO STATE TRANSITION v0.1\n")
        f.write("## STAGE C — CHRONOLOGICAL VALIDATION\n\n")
        
        for hyp in hypotheses:
            target_col = hyp["target"]
            baseline_n = len(val)
            baseline_events = int(val[target_col].sum())
            baseline_prob = baseline_events / baseline_n if baseline_n > 0 else 0
            
            subset = val[val[hyp['trans_type']] == hyp['trans']]
            cond_n = len(subset)
            cond_events = int(subset[target_col].sum())
            cond_prob = cond_events / cond_n if cond_n > 0 else 0
            
            diff = cond_prob - baseline_prob
            
            # Block stability (expectation from Train is a NEGATIVE deviation, i.e., cond < baseline)
            blocks = []
            stable_count = 0
            valid_blocks = 0
            
            for b, group in subset.groupby('block'):
                if len(group) > 0:
                    b_events = int(group[target_col].sum())
                    b_n = len(group)
                    b_prob = b_events / b_n
                    b_diff = b_prob - baseline_prob
                    
                    if b_diff < 0:
                        stable_count += 1
                    valid_blocks += 1
                    
                    blocks.append({
                        "Block_ID": b,
                        "N": b_n,
                        "Events": b_events,
                        "Prob": f"{b_prob*100:.1f}%",
                        "Diff_from_Baseline": f"{b_diff*100:+.1f}pp"
                    })
                    
            stability_pct = (stable_count / valid_blocks) * 100 if valid_blocks > 0 else 0
            
            f.write(f"### {hyp['id']} \n")
            f.write(f"- Transition: `{hyp['trans']}`\n")
            f.write(f"- Outcome: `{target_col}`\n\n")
            
            f.write("#### Aggregate Validation Telemetry\n")
            f.write("```text\n")
            f.write(f"Baseline N             = {baseline_n}\n")
            f.write(f"Baseline events        = {baseline_events}\n")
            f.write(f"Baseline probability   = {baseline_prob*100:.1f}%\n\n")
            
            f.write(f"Conditional N          = {cond_n}\n")
            f.write(f"Conditional events     = {cond_events}\n")
            f.write(f"Conditional probability= {cond_prob*100:.1f}%\n\n")
            
            f.write(f"Difference             = {diff*100:+.1f}pp\n")
            f.write(f"Block stability        = {stability_pct:.1f}% ({stable_count}/{valid_blocks} blocks < baseline)\n")
            f.write("```\n\n")
            
            f.write("#### Individual Block Breakdown\n")
            blocks_df = pd.DataFrame(blocks)
            f.write(df_to_markdown(blocks_df))
            f.write("\n---\n\n")
            
    print(f"Validation complete. Report generated at {REPORT_FILE}")

if __name__ == "__main__":
    run_validation()
