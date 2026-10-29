import subprocess
import re
import os
import sys
from collections import defaultdict
import statistics

SEEDS = list(range(40, 45))
DURATION = 80
CMD_TEMPLATE = "python3 scripts/mock_streamer.py {args} --duration {dur} --seed {seed} | VALIDATION_BLOCK_SIZE=3 EOS_ALPHA=2.0 EOS_LAMBDA=0.6 DISABLE_STRATEGY=1 ./target/debug/examples/live_engine"

def parse_results(output):
    net_expectancy = []
    flt = r"([-+]?\d*\.?\d+(?:[eE][-+]?\d+)?)"
    for line in output.split("\n"):
        if "[NET_EXPECTANCY]" in line:
            m = re.search(fr"d=(\d+) r=([^\s]+) net={flt} t_net={flt}", line)
            if m: net_expectancy.append({"net": float(m.group(3))})
    return net_expectancy

def run_test_suite(name, env_vars, streamer_args=""):
    print(f"\n🧪 Running Test Suite: {name}")
    all_nets = []
    
    for seed in SEEDS:
        print(f"  Seed {seed}...", end="", flush=True)
        cmd = CMD_TEMPLATE.format(dur=DURATION, seed=seed, args=streamer_args)
        env = os.environ.copy()
        env.update(env_vars)
        proc = subprocess.Popen(cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=env)
        stdout, stderr = proc.communicate()
        net = parse_results(stdout)
        
        all_nets.extend([n["net"] for n in net])
        print(" done.")

    mean_net = statistics.mean(all_nets) if all_nets else 0
    std_net = statistics.stdev(all_nets) if len(all_nets) > 1 else 0
    sharpe = mean_net / (std_net + 1e-9)
    
    print(f"\n--- {name} Results Summary ---")
    print(f"  Mean Net PnL: {mean_net*10000:+.2f} bps")
    print(f"  Sharpe Proxy:  {sharpe:.2f}")
    
    return mean_net, sharpe

def main():
    # A. Baseline
    base_net, base_sharpe = run_test_suite("BASELINE", {"TRAP_POLICY": "0"})
    
    # B. Avoid
    avoid_net, avoid_sharpe = run_test_suite("AVOID", {"TRAP_POLICY": "1"})
    
    # C. Flip
    flip_net, flip_sharpe = run_test_suite("FLIP", {"TRAP_POLICY": "2"})
    
    # D. Sniper Hybrid
    sniper_net, sniper_sharpe = run_test_suite("SNIPER HYBRID", {"TRAP_POLICY": "3"})
    
    print(f"\n⚖️ POLICY A/B/C/D COMPARISON")
    print(f"  BASELINE PnL: {base_net*10000:+.2f} bps")
    print(f"  AVOID PnL:    {avoid_net*10000:+.2f} bps")
    print(f"  FLIP PnL:     {flip_net*10000:+.2f} bps")
    print(f"  SNIPER PnL:   {sniper_net*10000:+.2f} bps")
    
    best = max([("BASELINE", base_net), ("AVOID", avoid_net), ("FLIP", flip_net), ("SNIPER", sniper_net)], key=lambda x: x[1])
    print(f"\n🏆 WINNING POLICY: {best[0]}")

if __name__ == "__main__":
    main()
