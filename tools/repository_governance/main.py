import sys
import os

# Add parent directory to path so we can import modules
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from repository_governance.inventory import Inventory
from repository_governance.evidence import EvidenceCollector
from repository_governance.classifiers import Classifier
from repository_governance.evolution import EvolutionTracker
from repository_governance.invariants import InvariantChecker
from repository_governance.metrics import GovernanceMetrics
from repository_governance.reports import ReportGenerator

def main():
    workspace_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    config_path = os.path.join(workspace_root, 'tools', 'repository_governance', 'config', 'governance.yaml')
    reports_dir = os.path.join(workspace_root, 'reports', 'governance')

    print("Phase 1: Scanning Repository & Building Inventory...")
    inventory = Inventory(workspace_root, config_path)
    inventory.scan()
    print(f"Discovered {len(inventory.artifacts)} artifacts.")

    print("Phase 2: Extracting Evidence...")
    ev_collector = EvidenceCollector(inventory)
    ev_collector.collect()

    print("Phase 3: Detecting Evolution Chains & Lineage...")
    tracker = EvolutionTracker(inventory, ev_collector)
    tracker.detect_families()

    print("Phase 4: Classifying Artifacts...")
    classifier = Classifier(inventory, ev_collector)
    classifier.classify_all()

    print("Phase 5: Validating Invariants...")
    checker = InvariantChecker(inventory, ev_collector)
    checker.check_all()

    print("Phase 6: Computing Governance Metrics...")
    metrics = GovernanceMetrics(inventory, ev_collector, classifier, checker)

    print(f"Phase 7: Generating Reports in {reports_dir}...")
    reporter = ReportGenerator(reports_dir)
    reporter.generate(inventory, ev_collector, classifier, tracker, metrics)
    
    print("Repository Governance Audit Complete!")

if __name__ == "__main__":
    main()
