import time
import json
import os
from pathlib import Path

def main():
    path = Path("analysis/real_live/governor_state.json")
    path.parent.mkdir(parents=True, exist_ok=True)
    while True:
        state = {"gov_mult": 1.0, "ts": int(time.time())}
        with open(path, "w") as f:
            json.dump(state, f)
        time.sleep(0.5)

if __name__ == "__main__":
    main()
