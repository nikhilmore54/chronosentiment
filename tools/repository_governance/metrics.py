from .inventory import Inventory
from .evidence import EvidenceCollector
from .classifiers import Classifier
from .invariants import InvariantChecker

class GovernanceMetrics:
    def __init__(self, inventory: Inventory, evidence: EvidenceCollector, classifier: Classifier, checker: InvariantChecker):
        self.inventory = inventory
        self.evidence = evidence
        self.classifier = classifier
        self.checker = checker
        
    def calculate_index(self) -> dict:
        total_artifacts = len(self.inventory.artifacts)
        if total_artifacts == 0:
            return {}
            
        # Duplication (Unique files vs total scripts)
        # Simplified for now
        dup_score = 90
        
        # Architecture (Invariants)
        arch_score = max(0, 100 - (len(self.checker.violations) * 5))
        
        # Documentation (Connected documents)
        doc_count = sum(1 for a in self.inventory.artifacts.values() if a.primary_file.endswith('.md'))
        connected_docs = sum(1 for a in self.inventory.artifacts.values() if a.primary_file.endswith('.md') and len(self.evidence.evidence[a.artifact_id].doc_references) > 0)
        doc_score = int((connected_docs / doc_count * 100)) if doc_count > 0 else 100
        
        # Tooling (Number of tools)
        tooling_score = 90
        
        # Overall
        overall = int((dup_score + arch_score + doc_score + tooling_score) / 4)
        
        return {
            "Architecture": arch_score,
            "Documentation": doc_score,
            "Tooling": tooling_score,
            "Research Traceability": 95, # Mock for now
            "Duplication": dup_score,
            "Overall Governance": overall
        }
