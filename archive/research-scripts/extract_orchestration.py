import re

def main():
    with open("archive/research_outputs/original_engine.rs", "r") as f:
        code = f.read()

    # Extract AssetEvoState
    asset_state = re.search(r"pub struct AssetEvoState \{.*?\}", code, re.DOTALL)
    asset_impl = re.search(r"impl AssetEvoState \{.*?\}", code, re.DOTALL)

    # Extract GlobalEvoState
    global_state = re.search(r"pub struct GlobalEvoState \{.*?\}", code, re.DOTALL)
    global_impl = re.search(r"impl GlobalEvoState \{.*?\}", code, re.DOTALL)

    out = "use std::collections::HashMap;\nuse serde::{Serialize, Deserialize};\n\n"
    if asset_state: out += f"#[derive(Debug, Clone, Serialize, Deserialize)]\n{asset_state.group(0)}\n\n"
    if asset_impl: out += f"{asset_impl.group(0)}\n\n"
    if global_state: out += f"#[derive(Debug, Clone, Default, Serialize, Deserialize)]\n{global_state.group(0)}\n\n"
    if global_impl: out += f"{global_impl.group(0)}\n\n"

    with open("financial/strategies/src/orchestration/runtime_state.rs", "w") as f:
        f.write(out)
        
    with open("financial/strategies/src/orchestration/mod.rs", "w") as f:
        f.write("pub mod runtime_state;\n")

if __name__ == "__main__":
    main()
