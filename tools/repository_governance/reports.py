import os
import json
import csv
from datetime import datetime
from .inventory import Inventory
from .evidence import EvidenceCollector
from .classifiers import Classifier
from .metrics import GovernanceMetrics
from .evolution import EvolutionTracker

class ReportGenerator:
    def __init__(self, output_dir: str):
        self.output_dir = output_dir
        
    def generate(self, inventory: Inventory, ev: EvidenceCollector, clf: Classifier, tracker: EvolutionTracker, metrics: GovernanceMetrics):
        date_str = datetime.now().strftime("%Y-%m-%d")
        report_dir = os.path.join(self.output_dir, date_str)
        latest_dir = os.path.join(self.output_dir, "latest")
        
        os.makedirs(report_dir, exist_ok=True)
        os.makedirs(latest_dir, exist_ok=True)
        
        self._write_inventory(os.path.join(report_dir, "repository_inventory.csv"), inventory, ev, clf)
        self._write_governance_index(os.path.join(report_dir, "governance_index.md"), metrics)
        self._write_evidence_report(os.path.join(report_dir, "recommendations.md"), inventory, clf)
        self._write_families(os.path.join(report_dir, "evolution_chains.md"), inventory, tracker)
        
        # Symlink or copy to latest
        self._copy_to_latest(report_dir, latest_dir)
        
    def _write_inventory(self, path: str, inventory: Inventory, ev: EvidenceCollector, clf: Classifier):
        with open(path, 'w', newline='', encoding='utf-8') as f:
            writer = csv.writer(f)
            writer.writerow(['Artifact ID', 'Primary File', 'Class', 'Recommendation', 'Confidence'])
            for art_id, art in inventory.artifacts.items():
                rec = clf.recommendations.get(art_id)
                writer.writerow([
                    art_id, 
                    art.primary_file, 
                    rec.target_class if rec else "Unknown", 
                    rec.action if rec else "None",
                    rec.confidence if rec else 0
                ])
                
    def _write_governance_index(self, path: str, metrics: GovernanceMetrics):
        index = metrics.calculate_index()
        with open(path, 'w', encoding='utf-8') as f:
            f.write("# Governance Index\n\n")
            f.write("| Dimension | Score |\n")
            f.write("|---|---:|\n")
            for k, v in index.items():
                f.write(f"| {k} | {v} |\n")
                
    def _write_evidence_report(self, path: str, inventory: Inventory, clf: Classifier):
        with open(path, 'w', encoding='utf-8') as f:
            f.write("# Artifact Recommendations\n\n")
            for art_id, rec in clf.recommendations.items():
                if rec.action != "Keep":
                    art = inventory.artifacts[art_id]
                    f.write(f"## {art.primary_file} ({art_id})\n")
                    f.write(f"**Recommendation:** {rec.target_class} -> {rec.action}\n\n")
                    f.write("**Evidence:**\n")
                    for e in rec.evidence_list:
                        f.write(f"{e}\n")
                    f.write(f"\n**Confidence:** {rec.confidence}%\n\n---\n")

    def _write_families(self, path: str, inventory: Inventory, tracker: EvolutionTracker):
        with open(path, 'w', encoding='utf-8') as f:
            f.write("# Evolution Chains\n\n")
            for fam_id, fam in tracker.families.items():
                f.write(f"## Family {fam_id}\n")
                if fam.canonical:
                    f.write(f"- **Canonical:** {inventory.artifacts[fam.canonical].primary_file}\n")
                for member in fam.members:
                    if member != fam.canonical:
                        f.write(f"- {inventory.artifacts[member].primary_file}\n")
                f.write("\n")

    def _copy_to_latest(self, src_dir: str, dest_dir: str):
        import shutil
        for item in os.listdir(src_dir):
            s = os.path.join(src_dir, item)
            d = os.path.join(dest_dir, item)
            if os.path.isfile(s):
                shutil.copy2(s, d)
