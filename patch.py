import sys

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/candidate_validation_time_machine_v0_1.py", "r") as f:
    content = f.read()

content = content.replace('def simulate_portfolio(sim_results: list[dict], cost_bps: int):', '''def simulate_portfolio(sim_results: list[dict], cost_bps: int):
    concurrent_counts = []
    total_gross = 0.0''')

content = content.replace('''    for ts, typ, r in events:
        if typ == "entry":
            if len(active) >= MAX_CONCURRENT:''', '''    for ts, typ, r in events:
        concurrent_counts.append(len(active))
        if typ == "entry":
            if len(active) >= MAX_CONCURRENT:''')

content = content.replace('''            equity += net
            equity_curve.append(equity)
            gross_returns.append(gross)''', '''            equity += net
            total_gross += gross
            equity_curve.append(equity)
            gross_returns.append(gross)''')

ret_block = '''    total_net = sum(net_returns) if net_returns else 0.0
    win_rate = (win_cnt / (win_cnt + loss_cnt)) * 100 if (win_cnt + loss_cnt) else 0
    
    peak = equity_curve[0]
    max_dd = 0.0
    for val in equity_curve:
        if val > peak: peak = val
        dd = (peak - val) / peak if peak != 0 else 0
        if dd > max_dd: max_dd = dd
        
    return {
        "final_equity": equity,
        "net_return_pct": total_net / NOTIONAL * 100,
        "win_rate_pct": win_rate,
        "max_drawdown_pct": max_dd * 100,
    }'''

new_ret_block = '''    total_net = sum(net_returns) if net_returns else 0.0
    win_rate = (win_cnt / (win_cnt + loss_cnt)) * 100 if (win_cnt + loss_cnt) else 0
    
    peak = equity_curve[0]
    max_dd = 0.0
    for val in equity_curve:
        if val > peak: peak = val
        dd = (peak - val) / peak if peak != 0 else 0
        if dd > max_dd: max_dd = dd
        
    avg_concurrent = sum(concurrent_counts)/len(concurrent_counts) if concurrent_counts else 0.0
    turnover = sum(abs(a) for a in gross_returns) / NOTIONAL if gross_returns else 0.0
    avg_net = sum(net_returns)/len(net_returns) if net_returns else 0.0
        
    return {
        "final_equity": equity,
        "gross_return_pct": total_gross / NOTIONAL * 100,
        "net_return_pct": total_net / NOTIONAL * 100,
        "win_rate_pct": win_rate,
        "expectancy_per_trade_pct": (avg_net / NOTIONAL * 100),
        "max_drawdown_pct": max_dd * 100,
        "average_concurrent_positions": avg_concurrent,
        "turnover_multiple": turnover,
    }'''

content = content.replace(ret_block, new_ret_block)

main_end = '''    for cost_name, bps in COST_SCENARIOS.items():
        base_res = simulate_portfolio(baseline_sims, bps)
        cand_res = simulate_portfolio(candidate_sims, bps)
        for m in metrics:
            b_val = f"{base_res[m]:.2f}"
            c_val = f"{cand_res[m]:.2f}"
            print(f"| {cost_name} | {m} | {b_val} | {c_val} |")'''

new_main_end = '''    metrics = ["gross_return_pct", "net_return_pct", "win_rate_pct", "expectancy_per_trade_pct", "max_drawdown_pct", "average_concurrent_positions", "turnover_multiple"]
    for cost_name, bps in COST_SCENARIOS.items():
        base_res = simulate_portfolio(baseline_sims, bps)
        cand_res = simulate_portfolio(candidate_sims, bps)
        for m in metrics:
            b_val = f"{base_res.get(m, 0.0):.2f}"
            c_val = f"{cand_res.get(m, 0.0):.2f}"
            print(f"| {cost_name} | {m} | {b_val} | {c_val} |")
            
    print("\\n### Exit Reasons Count")
    print("| Exit Reason | Baseline | Candidate |")
    print("|---|---|---|")
    all_exits = set([r["sim_exit_reason"] for r in baseline_sims] + [r["sim_exit_reason"] for r in candidate_sims])
    for ex in sorted(all_exits):
        b_c = sum(1 for r in baseline_sims if r["sim_exit_reason"] == ex)
        c_c = sum(1 for r in candidate_sims if r["sim_exit_reason"] == ex)
        print(f"| {ex} | {b_c} | {c_c} |")
        
    print("\\n### Transition Table")
    print("Baseline -> Candidate : Count")
    transitions = {}
    for b, c in zip(baseline_sims, candidate_sims):
        t = (b["sim_exit_reason"], c["sim_exit_reason"])
        transitions[t] = transitions.get(t, 0) + 1
    for (b_ex, c_ex), count in sorted(transitions.items(), key=lambda x: x[1], reverse=True):
        print(f"{b_ex.ljust(20)} -> {c_ex.ljust(20)} : {count}")'''

content = content.replace(main_end, new_main_end)

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/candidate_validation_time_machine_v0_1.py", "w") as f:
    f.write(content)
