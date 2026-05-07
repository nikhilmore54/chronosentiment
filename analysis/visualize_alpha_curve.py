import pandas as pd
import matplotlib.pyplot as plt
import re

def parse_alpha_curve(log_file):
    data = []
    with open(log_file, 'r') as f:
        for line in f:
            if "[ALPHA_CURVE]" in line:
                m = re.search(r"r1=(.*?) r3=(.*?) r5=(.*?) r10=(.*?) r20=(.*?) r50=(.*?) r100=(.*)", line)
                if m:
                    data.append([float(x) * 10000 for x in m.groups()])
    
    df = pd.DataFrame(data, columns=[1, 3, 5, 10, 20, 50, 100])
    return df.mean()

def plot_horizon(means):
    plt.figure(figsize=(10, 6))
    means.plot(kind='line', marker='o')
    plt.title("Structural Alpha Curve: E[r_t] (bps)")
    plt.xlabel("Horizon (Ticks)")
    plt.ylabel("Return (bps)")
    plt.grid(True, linestyle='--', alpha=0.7)
    plt.axhline(0, color='black', linewidth=1)
    plt.savefig("alpha_horizon_curve.png")
    print("Curve generated: alpha_horizon_curve.png")

if __name__ == "__main__":
    # Mock usage
    means = pd.Series([0.2, 0.8, 1.2, 2.5, 3.8, 5.2, 7.1], index=[1, 3, 5, 10, 20, 50, 100])
    plot_horizon(means)
