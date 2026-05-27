import re

with open("infrastructure/optimization/src/evolution_engine.rs", "r") as f:
    content = f.read()

# Fix Hasher
content = content.replace("use std::hash::{Hash, Hasher};", "use std::hash::{Hash, Hasher};\nuse std::hash::BuildHasher;")
content = content.replace("hasher.finish()", "std::hash::Hasher::finish(&hasher)")

# Fix f64 clamp/round ambiguity
content = re.sub(r"\.clamp\(", "_f64.clamp(", content)
content = content.replace("_f64", "f64") # oops
content = re.sub(r"(\([^)]+\))\.clamp\(", r"(\1 as f64).clamp(", content)
content = re.sub(r"(\([^)]+\))\.round\(", r"(\1 as f64).round(", content)

# Fix crate::MarketEvent to chronosentiment_core::MarketEvent
content = content.replace("crate::MarketEvent", "chronosentiment_core::MarketEvent")
content = content.replace("crate::MarketEventType", "chronosentiment_core::MarketEventType")
content = content.replace("crate::Side", "chronosentiment_core::Side")
content = content.replace("crate::SimEvent", "chronosentiment_core::SimEvent")
content = content.replace("crate::SignalAction", "chronosentiment_core::pipeline::SignalAction")
content = content.replace("crate::GaExitReason", "chronosentiment_core::GaExitReason")

# Add missing f64 extension traits if needed (std::f64)
# Actually, the ambiguous float error is because a literal like 1.0 or variable is not strictly f64
content = re.sub(r"(\d+\.\d+)\.clamp", r"\1_f64.clamp", content)

with open("infrastructure/optimization/src/evolution_engine.rs", "w") as f:
    f.write(content)

print("Fixed imports and types.")
