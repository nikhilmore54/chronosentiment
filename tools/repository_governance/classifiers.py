from typing import Dict, List, Any
from .inventory import Inventory, Artifact
from .evidence import EvidenceCollector, ArtifactEvidence

class Recommendation:
    def __init__(self, target_class: str, action: str, evidence_list: List[str], confidence: int):
        self.target_class = target_class
        self.action = action
        self.evidence_list = evidence_list
        self.confidence = confidence

class Classifier:
    def __init__(self, inventory: Inventory, evidence_collector: EvidenceCollector):
        self.inventory = inventory
        self.evidence_collector = evidence_collector
        self.recommendations: Dict[str, Recommendation] = {}
        
    def classify_all(self):
        for art_id, art in self.inventory.artifacts.items():
            ev = self.evidence_collector.evidence[art_id]
            self.recommendations[art_id] = self._classify(art, ev)
            
    def _classify(self, art: Artifact, ev: ArtifactEvidence) -> Recommendation:
        primary = art.primary_file
        evidence_list = []
        confidence = 100
        
        # 1. Product Check
        is_product = any(primary.startswith(root) for root in self.inventory.config.get('product_roots', []))
        if is_product:
            evidence_list.append("✓ Located in product root")
            return Recommendation("Product", "Keep", evidence_list, 100)
            
        # 2. Infrastructure Check
        is_infra = any(primary.startswith(root) for root in self.inventory.config.get('infrastructure_roots', []))
        if is_infra or primary.endswith('Makefile') or primary.endswith('Dockerfile'):
            evidence_list.append("✓ Infrastructure configuration")
            return Recommendation("Infrastructure", "Keep", evidence_list, 100)
            
        # 3. Research Check
        is_research = any(primary.startswith(root) for root in self.inventory.config.get('research_roots', []))
        is_referenced_by_research = any("docs/research" in doc for doc in ev.doc_references)
        
        if is_research or is_referenced_by_research:
            evidence_list.append("✓ Research context detected")
            if is_referenced_by_research:
                evidence_list.append(f"✓ Referenced by {len(ev.doc_references)} research documents")
            return Recommendation("Research", "Keep", evidence_list, 95)
            
        # 4. Tool Check
        is_tool = any(primary.startswith(root) for root in self.inventory.config.get('tool_roots', []))
        if is_tool:
            evidence_list.append("✓ Located in tools root")
            return Recommendation("Tool", "Keep", evidence_list, 100)
            
        # 5. Transient Check (Orphaned / Disposable)
        # Tightened deletion policy: No imports, no callers, not in docs, not in CI, not research evidence, not intentionally archived.
        evidence_list.append(f"✓ Imports: {len(ev.imports)}")
        evidence_list.append(f"✓ Called by: {len(ev.called_by)}")
        evidence_list.append(f"✓ Doc references: {len(ev.doc_references)}")
        evidence_list.append(f"✓ CI references: {len(ev.ci_references)}")
        
        is_disposable = (
            len(ev.imports) == 0 and
            len(ev.called_by) == 0 and
            len(ev.doc_references) == 0 and
            len(ev.ci_references) == 0
        )
        
        if is_disposable:
            # Maybe it should be promoted to Tool if it has lots of functions? (heuristic)
            if len(ev.functions) > 5 and len(ev.comments) > 10:
                evidence_list.append("✓ High density of logic and documentation detected")
                return Recommendation("Transient", "Promote to Tool", evidence_list, 80)
            else:
                return Recommendation("Transient", "Delete", evidence_list, 97)
                
        # Fallback
        evidence_list.append("⚠ Mixed signals detected")
        return Recommendation("Transient", "Investigate", evidence_list, 50)
