import json
import matplotlib.pyplot as plt

def main():
    with open("clusters.json", "r") as f:
        data = json.load(f)

    prices = [d["price"] for d in data]
    ts = list(range(len(prices)))

    plt.figure(figsize=(14,6))
    plt.plot(ts, prices, label="INFY.NS Time Series", color='blue', alpha=0.6)

    colors = {
        0: 'purple',   # Conviction
        1: 'green',    # Momentum
        2: 'orange',   # Reversion
        3: 'red'       # Volatility
    }

    # plot clusters
    cluster_count = 0
    for i, d in enumerate(data):
        for c in d["clusters"]:
            cluster_count += 1
            arch_color = colors.get(c["archetype"], 'black')
            
            # draw the shaded span
            plt.axvspan(c["start_idx"], c["end_idx"], color=arch_color, alpha=0.2)
            
            # center marker
            plt.scatter(c["center"], prices[int(c["center"].__round__())], color=arch_color, marker='*', s=150, zorder=5)
            
            # individual signals
            for sig in c["signals"]:
                idx = sig["idx"]
                plt.scatter(idx, prices[idx], color=arch_color, marker='o', s=30, zorder=4)
                
            # annotate
            plt.annotate(f"{c['label']}\nW:{c['weight']:.2f}",
                         (c["center"], prices[int(c["center"].__round__())]),
                         textcoords="offset points",
                         xytext=(0,10), ha='center',
                         fontsize=8, bbox=dict(boxstyle="round,pad=0.3", fc="cyan", ec="b", lw=1, alpha=0.5))

    print(f"📊 Visualization Loaded: {cluster_count} Clusters")
    plt.legend()
    plt.title("Portfolio Engine Cluster Visualization")
    
    # Save chart as png for fast review
    plt.savefig("clusters_output.png")
    print("✅ Saved to clusters_output.png")

if __name__ == "__main__":
    main()
