import os
import hashlib
from typing import Dict, List, Optional
import yaml

class Artifact:
    def __init__(self, artifact_id: str, primary_file: str, related_files: List[str] = None):
        self.artifact_id = artifact_id
        self.primary_file = primary_file
        self.related_files = related_files or []
        
    def get_all_files(self) -> List[str]:
        return [self.primary_file] + self.related_files

class Inventory:
    def __init__(self, root_dir: str, config_path: str):
        self.root_dir = root_dir
        with open(config_path, 'r') as f:
            self.config = yaml.safe_load(f)
        self.artifacts: Dict[str, Artifact] = {}
        self.file_to_artifact: Dict[str, str] = {}
        
    def _is_excluded(self, path: str) -> bool:
        parts = os.path.relpath(path, self.root_dir).split(os.sep)
        return any(part in self.config.get('excluded_directories', []) for part in parts)
        
    def scan(self):
        """Scans the repository and builds logical artifacts."""
        artifact_counter = 1
        
        # Group by directory for scripts, or just treat each entry script as an artifact
        # For simplicity, each standalone script is an artifact, and directories like 'tools/repository_governance' are one artifact.
        
        # We will do a two pass: first identify module directories, then standalone files
        module_dirs = set()
        
        for root, dirs, files in os.walk(self.root_dir):
            if self._is_excluded(root):
                dirs[:] = [] # Stop traversing
                continue
                
            if '__init__.py' in files or 'main.py' in files:
                module_dirs.add(root)
                
            # Rust crates are modules
            if 'Cargo.toml' in files:
                module_dirs.add(root)
                
            # JS apps are modules
            if 'package.json' in files:
                module_dirs.add(root)
                
        # Create artifacts for modules
        for mod_dir in module_dirs:
            art_id = f"ART-{artifact_counter:04d}"
            artifact_counter += 1
            
            rel_dir = os.path.relpath(mod_dir, self.root_dir)
            
            # Find all files in this module
            mod_files = []
            for r, _, fs in os.walk(mod_dir):
                if self._is_excluded(r): continue
                for f in fs:
                    mod_files.append(os.path.relpath(os.path.join(r, f), self.root_dir))
                    
            if mod_files:
                primary = next((f for f in mod_files if f.endswith('main.py') or f.endswith('Cargo.toml') or f.endswith('package.json')), mod_files[0])
                art = Artifact(art_id, primary, [f for f in mod_files if f != primary])
                self.artifacts[art_id] = art
                for f in mod_files:
                    self.file_to_artifact[f] = art_id

        # Second pass: standalone files
        for root, dirs, files in os.walk(self.root_dir):
            if self._is_excluded(root):
                dirs[:] = []
                continue
                
            # Skip if root is inside a module dir
            if any(root.startswith(m) for m in module_dirs):
                continue
                
            for file in files:
                ext = os.path.splitext(file)[1]
                if ext in ['.py', '.sh', '.rs', '.js', '.ts', '.md']:
                    rel_path = os.path.relpath(os.path.join(root, file), self.root_dir)
                    if rel_path not in self.file_to_artifact:
                        art_id = f"ART-{artifact_counter:04d}"
                        artifact_counter += 1
                        art = Artifact(art_id, rel_path)
                        self.artifacts[art_id] = art
                        self.file_to_artifact[rel_path] = art_id

    def get_artifact_by_file(self, filepath: str) -> Optional[Artifact]:
        art_id = self.file_to_artifact.get(filepath)
        if art_id:
            return self.artifacts[art_id]
        return None
