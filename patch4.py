import json

wipro = json.loads(open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/time_machine/observations/20260911/TIME005-OBS-20260911T100000Z-gen20260917T040836163706Z-WIPRO_NS.json").read())
hdfc = json.loads(open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/time_machine/observations/20260911/TIME005-OBS-20260911T100000Z-gen20260917T040836163706Z-HDFCBANK_NS.json").read())

print("WIPRO")
print(f"Target: {wipro['adaptive_target']}")
print(f"Risk: {wipro['adaptive_risk']}")
print(f"1st bar TS: {wipro['first_bar_after_t0']}")

print("HDFC")
print(f"Target: {hdfc['adaptive_target']}")
print(f"Risk: {hdfc['adaptive_risk']}")
print(f"1st bar TS: {hdfc['first_bar_after_t0']}")
