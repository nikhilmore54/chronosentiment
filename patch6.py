import sys

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/portfolio_exposure_diagnostic.py", "r") as f:
    content = f.read()

content = content.replace('''def get_horizon(decision: dict) -> int:
    h = decision.get("adaptive_horizon_sessions")
    if h is None:
        return 20
    val = math.ceil(float(h))
    return max(1, val)''', '''def get_horizon(decision: dict) -> int:
    return 300''')

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/portfolio_exposure_diagnostic.py", "w") as f:
    f.write(content)
