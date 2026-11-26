import os
import ast
import re
from typing import Dict, List, Set, Any
from .inventory import Artifact, Inventory

class ArtifactEvidence:
    def __init__(self):
        self.imported_by: Set[str] = set()
        self.imports: Set[str] = set()
        self.called_by: Set[str] = set()
        self.doc_references: Set[str] = set()
        self.ci_references: Set[str] = set()
        self.functions: Set[str] = set()
        self.classes: Set[str] = set()
        self.comments: List[str] = []
        self.last_modified: float = 0.0

class EvidenceCollector:
    def __init__(self, inventory: Inventory):
        self.inventory = inventory
        self.evidence: Dict[str, ArtifactEvidence] = {art_id: ArtifactEvidence() for art_id in inventory.artifacts}
        
    def collect(self):
        self._collect_ast_and_imports()
        self._collect_doc_references()
        self._collect_ci_references()
        self._collect_metadata()
        
    def _collect_ast_and_imports(self):
        for art_id, art in self.inventory.artifacts.items():
            for filepath in art.get_all_files():
                full_path = os.path.join(self.inventory.root_dir, filepath)
                if not filepath.endswith('.py'):
                    continue
                try:
                    with open(full_path, 'r', encoding='utf-8') as f:
                        content = f.read()
                    
                    tree = ast.parse(content)
                    
                    # Extract classes and functions
                    for node in ast.walk(tree):
                        if isinstance(node, ast.FunctionDef):
                            self.evidence[art_id].functions.add(node.name)
                        elif isinstance(node, ast.ClassDef):
                            self.evidence[art_id].classes.add(node.name)
                        elif isinstance(node, ast.Import):
                            for name in node.names:
                                self.evidence[art_id].imports.add(name.name)
                        elif isinstance(node, ast.ImportFrom):
                            if node.module:
                                self.evidence[art_id].imports.add(node.module)
                                
                    # Extract comments/docstrings using basic regex for now
                    self.evidence[art_id].comments = re.findall(r'#.*', content)
                except Exception:
                    pass

    def _collect_doc_references(self):
        # Scan markdown files for links or mentions of other files
        for art_id, art in self.inventory.artifacts.items():
            for filepath in art.get_all_files():
                if not filepath.endswith('.md'):
                    continue
                full_path = os.path.join(self.inventory.root_dir, filepath)
                try:
                    with open(full_path, 'r', encoding='utf-8') as f:
                        content = f.read()
                    for other_art_id, other_art in self.inventory.artifacts.items():
                        if art_id == other_art_id:
                            continue
                        # If a primary file is mentioned in this document
                        basename = os.path.basename(other_art.primary_file)
                        if basename in content:
                            self.evidence[other_art_id].doc_references.add(art_id)
                except Exception:
                    pass

    def _collect_ci_references(self):
        # Scan .github, Makefiles, etc.
        infra_files = []
        for root, _, files in os.walk(self.inventory.root_dir):
            if '.github' in root or 'Makefile' in files:
                for file in files:
                    infra_files.append(os.path.join(root, file))
                    
        for infra_file in infra_files:
            try:
                with open(infra_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                for art_id, art in self.inventory.artifacts.items():
                    basename = os.path.basename(art.primary_file)
                    if basename in content:
                        self.evidence[art_id].ci_references.add(infra_file)
            except Exception:
                pass

    def _collect_metadata(self):
        for art_id, art in self.inventory.artifacts.items():
            full_path = os.path.join(self.inventory.root_dir, art.primary_file)
            try:
                self.evidence[art_id].last_modified = os.path.getmtime(full_path)
            except OSError:
                pass
