with open("core/src/pipeline.rs", "r") as f:
    text = f.read()

text = text.replace("safe_log!(", "println!(")
text = text.replace("use crate::safe_log;", "")

with open("core/src/pipeline.rs", "w") as f:
    f.write(text)

with open("core/src/reco.rs", "r") as f:
    text = f.read()

text = text.replace("consensus_conf *= (1.0 - 0.3 * entropy_penalty);", "consensus_conf *= 1.0 - 0.3 * entropy_penalty;")

with open("core/src/reco.rs", "w") as f:
    f.write(text)
