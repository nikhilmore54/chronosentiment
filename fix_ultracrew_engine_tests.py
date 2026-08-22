import re

file_path = "adapters/ultracrew/tests/ultracrew_engine_tests.rs"

with open(file_path, "r") as f:
    content = f.read()

# Fix ConstraintEngine imports and uses
content = content.replace("use ultracrew::constraint_engine::ConstraintEngine;", "use ultracrew::constraint_engine::{DomainConstraintEvaluator, InrcConstraintEvaluator};")
content = content.replace("ConstraintEngine::new(", "InrcConstraintEvaluator::new(")

# Fix missing crew_role and flight_id in Shift initialization
pattern = r'(Shift\s*\{\s*id:\s*\d+,\s*start_hour:\s*\d+,\s*duration_hours:\s*\d+,\s*required_skill:\s*[^}]+)(\s*\})'
content = re.sub(pattern, r'\1, crew_role: None, flight_id: None \2', content)

with open(file_path, "w") as f:
    f.write(content)

print("Done fixing ultracrew_engine_tests.rs")
