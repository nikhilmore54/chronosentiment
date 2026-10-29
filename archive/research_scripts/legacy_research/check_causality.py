
import sys

def check_causality(file_path):
    outcomes = {"outcome=0": 0, "outcome=1": 0}
    edges = []
    
    with open(file_path, 'r') as f:
        for line in f:
            if "[EDGE_CAUSAL]" in line:
                if "outcome=0" in line:
                    outcomes["outcome=0"] += 1
                elif "outcome=1" in line:
                    outcomes["outcome=1"] += 1
                    try:
                        edge_str = line.split("edge=")[1].split(" ")[0]
                        edges.append(float(edge_str))
                    except:
                        pass
            
    print(f"Outcomes: {outcomes}")
    if edges:
        edges.sort()
        print(f"Edge range: {edges[0]} to {edges[-1]}")
        print(f"Sample edges: {edges[:5]} ... {edges[-5:]}")
    else:
        print("No outcome=1 found")

if __name__ == "__main__":
    check_causality("/tmp/run_scientific_v13.txt")
