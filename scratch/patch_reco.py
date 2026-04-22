import os

filepath = "/Users/nikhil/ChronoSentiment_MEGA_FINAL/core/src/reco.rs"
with open(filepath, "r") as f:
    text = f.read()

if "use crate::Side;" not in text:
    text = "use crate::Side;\n" + text

with open(filepath, "w") as f:
    f.write(text)
