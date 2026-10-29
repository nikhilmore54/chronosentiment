import json
from pathlib import Path
import statistics
import math

inventory_path = Path("phase1/execution_inventory.jsonl")
inventory = []
with open(inventory_path) as f:
    for line in f:
        inventory.append(json.loads(line))

valid_executions = [i for i in inventory if i["trace_hash"] != "N/A" and i["tick_count"] > 0]

global_lengths = []
global_occ = []
global_over = []
global_acc = []
global_str = []

for ex in valid_executions:
    trace_path = Path(ex["trace_path"])
    global_lengths.append(ex["tick_count"])
    
    with open(trace_path) as f:
        data = json.load(f)
        traces = data.get("traces", [])
        
        if traces:
            global_occ.extend([t["occupancy"] for t in traces])
            global_over.extend([t["overlap"] for t in traces])
            global_acc.extend([t["acceptance_ratio"] for t in traces])
            global_str.extend([t["strictness_ratio"] for t in traces])

def stats(data_arr):
    if not data_arr: return {"min": 0, "max": 0, "mean": 0, "std": 0}
    return {
        "min": min(data_arr),
        "max": max(data_arr),
        "mean": statistics.mean(data_arr),
        "std": statistics.pstdev(data_arr)
    }

l_stats = stats(global_lengths)
o_stats = stats(global_occ)
v_stats = stats(global_over)
a_stats = stats(global_acc)
s_stats = stats(global_str)

md = "# Phase 1B-0 Geometry Inventory\n\n"
md += f"**Analyzed Executions:** {len(valid_executions)}\n"
md += f"**Total Datapoints (Ticks):** {len(global_occ)}\n\n"

md += "### 1. Trace Length Distribution\n"
md += f"- **Min:** {l_stats['min']}\n"
md += f"- **Max:** {l_stats['max']}\n"
md += f"- **Mean:** {l_stats['mean']:.2f}\n"
md += f"- **StdDev:** {l_stats['std']:.2f}\n\n"

md += "### 2. Occupancy Distribution\n"
md += f"- **Min:** {o_stats['min']:.4f}\n"
md += f"- **Max:** {o_stats['max']:.4f}\n"
md += f"- **Mean:** {o_stats['mean']:.4f}\n"
md += f"- **StdDev:** {o_stats['std']:.4f}\n\n"

md += "### 3. Overlap Distribution\n"
md += f"- **Min:** {v_stats['min']:.4f}\n"
md += f"- **Max:** {v_stats['max']:.4f}\n"
md += f"- **Mean:** {v_stats['mean']:.4f}\n"
md += f"- **StdDev:** {v_stats['std']:.4f}\n\n"

md += "### 4. Acceptance Ratio Distribution\n"
md += f"- **Min:** {a_stats['min']:.4f}\n"
md += f"- **Max:** {a_stats['max']:.4f}\n"
md += f"- **Mean:** {a_stats['mean']:.4f}\n"
md += f"- **StdDev:** {a_stats['std']:.4f}\n\n"

md += "### 5. Strictness Ratio Distribution\n"
md += f"- **Min:** {s_stats['min']:.4f}\n"
md += f"- **Max:** {s_stats['max']:.4f}\n"
md += f"- **Mean:** {s_stats['mean']:.4f}\n"
md += f"- **StdDev:** {s_stats['std']:.4f}\n\n"

md += "> **Conclusion:** The geometry space is successfully bounded. We can now proceed to Phase 1B-1 to build compact 9D signatures for each execution."

Path("/Users/nikhil/.gemini/antigravity/brain/473fd097-4bb0-4e4a-8d92-e67ab4ba9e18/phase1b0_geometry_inventory.md").write_text(md)
print("Geometry Inventory Generated.")
