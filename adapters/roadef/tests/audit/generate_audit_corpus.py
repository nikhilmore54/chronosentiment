import json
import os

base_dir = "adapters/roadef/tests/audit"
cases = ["feasible", "budget_violation", "maxsegments_violation", "disconnected", "intervention_edge", "ecmp_tie"]

for case in cases:
    os.makedirs(f"{base_dir}/{case}", exist_ok=True)

# Base Diamond Network
net = {
  "directed": True,
  "multigraph": False,
  "nodes": [{"id": i} for i in range(4)],
  "links": [
    {"id": 0, "from": 0, "to": 1, "metric": 10, "capacity": 100},
    {"id": 1, "from": 1, "to": 0, "metric": 10, "capacity": 100},
    {"id": 2, "from": 0, "to": 2, "metric": 10, "capacity": 100},
    {"id": 3, "from": 2, "to": 0, "metric": 10, "capacity": 100},
    {"id": 4, "from": 1, "to": 3, "metric": 10, "capacity": 100},
    {"id": 5, "from": 3, "to": 1, "metric": 10, "capacity": 100},
    {"id": 6, "from": 2, "to": 3, "metric": 10, "capacity": 100},
    {"id": 7, "from": 3, "to": 2, "metric": 10, "capacity": 100}
  ]
}

tm_2_slots = {
  "num_time_slots": 2,
  "demands": [
    {"s": 0, "t": 3, "v": [100.0, 100.0]}
  ]
}

tm_1_slot = {
  "num_time_slots": 1,
  "demands": [
    {"s": 0, "t": 3, "v": [100.0]}
  ]
}

# 1. Feasible
with open(f"{base_dir}/feasible/net.json", "w") as f: json.dump(net, f)
with open(f"{base_dir}/feasible/tm.json", "w") as f: json.dump(tm_1_slot, f)
with open(f"{base_dir}/feasible/scenario.json", "w") as f: 
    json.dump({"max_segments": 10, "budget": [], "interventions": []}, f)
with open(f"{base_dir}/feasible/srpaths.json", "w") as f:
    json.dump({"srpaths": [{"d": 0, "t": 0, "w": [1]}]}, f) # Specific path 0->1->3

# 2. Budget Violation
with open(f"{base_dir}/budget_violation/net.json", "w") as f: json.dump(net, f)
with open(f"{base_dir}/budget_violation/tm.json", "w") as f: json.dump(tm_2_slots, f)
with open(f"{base_dir}/budget_violation/scenario.json", "w") as f: 
    # Budget is 0 at internal t=1 (JSON t=1)
    json.dump({"max_segments": 10, "budget": [{"t": 1, "value": 0}], "interventions": []}, f)
with open(f"{base_dir}/budget_violation/srpaths.json", "w") as f:
    # Path changes from [] at t=0 to [1] at t=1, costing budget!
    json.dump({"srpaths": [{"d": 0, "t": 0, "w": []}, {"d": 0, "t": 1, "w": [1]}]}, f)

# 3. Max Segments Violation
with open(f"{base_dir}/maxsegments_violation/net.json", "w") as f: json.dump(net, f)
with open(f"{base_dir}/maxsegments_violation/tm.json", "w") as f: json.dump(tm_1_slot, f)
with open(f"{base_dir}/maxsegments_violation/scenario.json", "w") as f: 
    # Max segments = 1. If path is [1], it has 2 segments (0->1, 1->3). Violates!
    json.dump({"max_segments": 1, "budget": [], "interventions": []}, f)
with open(f"{base_dir}/maxsegments_violation/srpaths.json", "w") as f:
    json.dump({"srpaths": [{"d": 0, "t": 0, "w": [1]}]}, f)

# 4. Disconnected
disconnected_net = dict(net)
# Remove edges to node 3
disconnected_net["links"] = [l for l in net["links"] if l["to"] != 3 and l["from"] != 3]
with open(f"{base_dir}/disconnected/net.json", "w") as f: json.dump(disconnected_net, f)
with open(f"{base_dir}/disconnected/tm.json", "w") as f: json.dump(tm_1_slot, f)
with open(f"{base_dir}/disconnected/scenario.json", "w") as f: 
    json.dump({"max_segments": 10, "budget": [], "interventions": []}, f)
with open(f"{base_dir}/disconnected/srpaths.json", "w") as f:
    json.dump({"srpaths": [{"d": 0, "t": 0, "w": []}]}, f)

# 5. Intervention Edge
with open(f"{base_dir}/intervention_edge/net.json", "w") as f: json.dump(net, f)
with open(f"{base_dir}/intervention_edge/tm.json", "w") as f: json.dump(tm_2_slots, f)
with open(f"{base_dir}/intervention_edge/scenario.json", "w") as f: 
    # Intervene on edge 1->3 at internal t=1 (JSON t=1). This breaks the explicit path [1].
    # In networktools, the link ID is its 0-based index in the links array. 1->3 is index 4.
    json.dump({"max_segments": 10, "budget": [{"t": 1, "value": 100}], "interventions": [{"t": 1, "links": [4]}]}, f)
with open(f"{base_dir}/intervention_edge/srpaths.json", "w") as f:
    # Explicit path goes exactly through the broken edge.
    json.dump({"srpaths": [{"d": 0, "t": 0, "w": [1]}, {"d": 0, "t": 1, "w": [1]}]}, f)

# 6. ECMP Tie
with open(f"{base_dir}/ecmp_tie/net.json", "w") as f: json.dump(net, f)
with open(f"{base_dir}/ecmp_tie/tm.json", "w") as f: json.dump(tm_1_slot, f)
with open(f"{base_dir}/ecmp_tie/scenario.json", "w") as f: 
    json.dump({"max_segments": 10, "budget": [], "interventions": []}, f)
with open(f"{base_dir}/ecmp_tie/srpaths.json", "w") as f:
    # Empty path -> follows shortest path -> splits 50/50
    json.dump({"srpaths": [{"d": 0, "t": 0, "w": []}]}, f)

print("Audit corpus generated successfully.")
