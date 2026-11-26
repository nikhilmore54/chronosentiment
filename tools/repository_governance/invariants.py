from typing import List, Dict
from .inventory import Inventory
from .evidence import EvidenceCollector

class InvariantViolation:
    def __init__(self, rule_description: str, artifact_id: str, details: str):
        self.rule_description = rule_description
        self.artifact_id = artifact_id
        self.details = details

class InvariantChecker:
    def __init__(self, inventory: Inventory, evidence_collector: EvidenceCollector):
        self.inventory = inventory
        self.evidence_collector = evidence_collector
        self.violations: List[InvariantViolation] = []
        
    def check_all(self):
        rules = self.inventory.config.get('invariants', [])
        for rule in rules:
            rule_type = rule.get('rule')
            if rule_type == 'no_import':
                self._check_no_import(rule)

    def _check_no_import(self, rule: dict):
        desc = rule.get('description')
        source_root = rule.get('source')
        target_root = rule.get('target')
        
        # simplified check
        for art_id, art in self.inventory.artifacts.items():
            if source_root and art.primary_file.startswith(source_root):
                ev = self.evidence_collector.evidence[art_id]
                for imp in ev.imports:
                    # In Python, check if import matches target root
                    if target_root and imp.startswith(target_root.replace('/', '.')):
                        self.violations.append(InvariantViolation(desc, art_id, f"Imports forbidden module: {imp}"))
                        
            # Categories check
            source_cats = rule.get('source_categories', [])
            target_cats = rule.get('target_categories', [])
            
            # This would integrate with the classifier if it ran after classification,
            # For now, it's a structural check based on paths.
