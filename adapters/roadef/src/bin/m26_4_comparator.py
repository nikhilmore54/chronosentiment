import pandas as pd
import numpy as np

def main():
    print("=== M26.4 Holdout Comparison: Shadow vs Active Advisory ===")
    
    try:
        df_shadow = pd.read_csv("m26_4a_discovery_curves.csv")
    except FileNotFoundError:
        print("Error: m26_4a_discovery_curves.csv not found.")
        return
        
    try:
        df_active = pd.read_csv("m26_4b_discovery_curves.csv")
    except FileNotFoundError:
        print("Error: m26_4b_discovery_curves.csv not found.")
        return
        
    instances = sorted(list(set(df_shadow['instance_id'].unique()) | set(df_active['instance_id'].unique())))
    
    print(f"\nEvaluating holdout instances: {instances}")
    
    for inst in instances:
        print(f"\n--- Instance setA-{inst:02d} ---")
        
        # Filter for instance
        sh_inst = df_shadow[df_shadow['instance_id'] == inst]
        ac_inst = df_active[df_active['instance_id'] == inst]
        
        # Get best objectives and total nodes expanded
        def get_mode_summary(df_inst):
            modes = df_inst['mode'].unique()
            base_modes = [m for m in modes if not m.endswith('-Final')]
            summary = {}
            for m in base_modes:
                m_df = df_inst[df_inst['mode'] == m]
                best_obj = m_df['best_obj'].min() if not m_df.empty else float('inf')
                
                # Get the final node count from the -Final log
                final_df = df_inst[df_inst['mode'] == f"{m}-Final"]
                if not final_df.empty:
                    total_nodes = final_df.iloc[0]['nodes_expanded']
                else:
                    total_nodes = m_df['nodes_expanded'].max() if not m_df.empty else 0
                    
                summary[m] = {
                    'best_obj': best_obj,
                    'nodes_expanded': total_nodes
                }
            return summary
            
        sh_sum = get_mode_summary(sh_inst)
        ac_sum = get_mode_summary(ac_inst)
        
        # Print comparison table
        print(f"{'Mode':<22} | {'Advisory Type':<15} | {'Best Objective':<15} | {'Total Nodes Visited'}")
        print("-" * 68)
        
        # 1. Natural DFS
        if 'Natural' in sh_sum:
            print(f"{'Natural DFS':<22} | {'None (Baseline)':<15} | {sh_sum['Natural']['best_obj']:<15.4f} | {sh_sum['Natural']['nodes_expanded']}")
            
        # 2. Random Sibling Ordering (Mean)
        r_sh_objs = [sh_sum[m]['best_obj'] for m in sh_sum if m.startswith('Random') and sh_sum[m]['best_obj'] < float('inf')]
        r_sh_nodes = [sh_sum[m]['nodes_expanded'] for m in sh_sum if m.startswith('Random')]
        if r_sh_objs:
            print(f"{'Random Sibling (Mean)':<22} | {'Ordering Only':<15} | {np.mean(r_sh_objs):<15.4f} | {int(np.mean(r_sh_nodes))}")
            
        # 3. Shadow Advisory
        if 'Coralys' in sh_sum:
            print(f"{'Coralys Shadow':<22} | {'Ordering Only':<15} | {sh_sum['Coralys']['best_obj']:<15.4f} | {sh_sum['Coralys']['nodes_expanded']}")
            
        # 4. Active Advisory
        if 'Coralys' in ac_sum:
            print(f"{'Coralys Active':<22} | {'Ordering + Prune':<15} | {ac_sum['Coralys']['best_obj']:<15.4f} | {ac_sum['Coralys']['nodes_expanded']}")
            
        # Compute active pruning speedup and safety
        if 'Coralys' in sh_sum and 'Coralys' in ac_sum:
            sh_obj = sh_sum['Coralys']['best_obj']
            ac_obj = ac_sum['Coralys']['best_obj']
            sh_nodes = sh_sum['Coralys']['nodes_expanded']
            ac_nodes = ac_sum['Coralys']['nodes_expanded']
            
            quality_loss = (ac_obj - sh_obj) / sh_obj * 100.0 if sh_obj > 0 else 0.0
            node_reduction = (sh_nodes - ac_nodes) / sh_nodes * 100.0 if sh_nodes > 0 else 0.0
            
            print("\nActive Pruning Evaluation:")
            print(f"  * Solution Quality Delta: {quality_loss:+.4f}% (Goal: 0.00% loss)")
            print(f"  * Search Node Reduction:   {node_reduction:.2f}%")
            
            if abs(quality_loss) < 1e-4:
                print("  * [PASS] Intervention is 100% safe. Found identical best objective.")
            else:
                print("  * [WARNING] Solution quality difference detected!")
                
            if node_reduction >= 20.0:
                print(f"  * [SUCCESS] Substantial search space reduction of {node_reduction:.2f}% observed.")
            else:
                print(f"  * [OBSERVATION] Search space reduction of {node_reduction:.2f}% is modest on this instance.")

if __name__ == "__main__":
    main()
