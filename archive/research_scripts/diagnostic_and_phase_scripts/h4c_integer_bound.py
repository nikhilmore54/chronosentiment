x = 12
n = 20
hours = [136]*x + [128]*(n-x)
mean = sum(hours)/n
var = sum((h-mean)**2 for h in hours)/n
sc1 = var * 10
fitness = 10000 - sc1
print(f"Integer 8h constraint: {x} workers at 136h, {n-x} workers at 128h")
print(f"Mean={mean}, Variance={var:.4f}, SC1={sc1:.4f}")
print(f"Implied max fitness (SC2=0): {fitness:.4f}")
print()
print(f"TRUE lower bound for UB-001 with 8h shifts (HC1 only, no HC2/HC3/rest):")
print(f"  SC1 = {sc1:.4f}, max fitness = {fitness:.4f}")
print()
ga_best = 9918.4
gap = fitness - ga_best
print(f"GA best: {ga_best}")
print(f"Gap (integer bound - GA): {gap:.4f} fitness units")
if gap > 0.1:
    print("=> GA has NOT reached the integer-constrained optimum.")
    print(f"   There is a gap of {gap:.2f} fitness units to close.")
else:
    print("=> GA is at or above the integer-constrained bound.")