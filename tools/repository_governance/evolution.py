import os
from typing import Dict, List, Set
from .inventory import Inventory, Artifact
from .evidence import EvidenceCollector
import difflib

class EvolutionFamily:
    def __init__(self, family_id: str):
        self.family_id = family_id
        self.members: List[str] = [] # List of art_ids
        self.canonical: str = None
        self.reference: str = None

class EvolutionTracker:
    def __init__(self, inventory: Inventory, evidence_collector: EvidenceCollector):
        self.inventory = inventory
        self.evidence_collector = evidence_collector
        self.families: Dict[str, EvolutionFamily] = {}
        self.artifact_to_family: Dict[str, str] = {}
        
    def detect_families(self):
        """Groups artifacts into evolutionary families based on multiple signals."""
        family_counter = 1
        
        # Simple heuristic: artifacts in the same directory sharing similar prefixes
        # e.g. cleanup.py, cleanup_v2.py, cleanup_final.py
        
        # Gather all standalone python scripts (mostly where this happens)
        scripts = [art for art in self.inventory.artifacts.values() if art.primary_file.endswith('.py') and len(art.related_files) == 0]
        
        unassigned = set(art.artifact_id for art in scripts)
        
        while unassigned:
            art_id = unassigned.pop()
            art = self.inventory.artifacts[art_id]
            basename = os.path.basename(art.primary_file)
            prefix = basename.split('_')[0].split('.')[0]
            
            family_members = [art_id]
            to_remove = set()
            
            for other_id in unassigned:
                other_art = self.inventory.artifacts[other_id]
                other_basename = os.path.basename(other_art.primary_file)
                
                # Multiple signals check:
                # 1. Filename similarity
                seq = difflib.SequenceMatcher(None, basename, other_basename)
                is_similar_name = seq.ratio() > 0.6 or other_basename.startswith(prefix)
                
                # 2. Shared imports
                ev1 = self.evidence_collector.evidence[art_id]
                ev2 = self.evidence_collector.evidence[other_id]
                shared_imports = ev1.imports.intersection(ev2.imports)
                
                # 3. Shared functions
                shared_funcs = ev1.functions.intersection(ev2.functions)
                
                if is_similar_name and (len(shared_imports) > 0 or len(shared_funcs) > 0 or seq.ratio() > 0.8):
                    family_members.append(other_id)
                    to_remove.add(other_id)
                    
            unassigned -= to_remove
            
            if len(family_members) > 1:
                fam_id = f"FAM-{family_counter:04d}"
                family_counter += 1
                fam = EvolutionFamily(fam_id)
                fam.members = family_members
                
                # Sort by last modified
                family_members.sort(key=lambda aid: self.evidence_collector.evidence[aid].last_modified, reverse=True)
                
                # Canonical = Most recently modified
                fam.canonical = family_members[0]
                # Reference = Most referenced in docs (if any)
                fam.reference = max(family_members, key=lambda aid: len(self.evidence_collector.evidence[aid].doc_references))
                
                self.families[fam_id] = fam
                for m in family_members:
                    self.artifact_to_family[m] = fam_id
                    
    def trace_research_lineage(self) -> Dict[str, List[str]]:
        """Maps Research documents (CS-R-xxx) to implementing code artifacts."""
        lineage = {}
        for art_id, art in self.inventory.artifacts.items():
            if art.primary_file.endswith('.md') and "research" in art.primary_file:
                # This is a research doc
                # Find all artifacts that reference it, or that it references
                ev = self.evidence_collector.evidence[art_id]
                lineage[art_id] = list(ev.doc_references)
        return lineage
