import pandas as pd

df = pd.read_csv('validation_alignment.csv')

# Sort by Proxy (InternalFitness) - ascending (lower is better)
top_proxy = df.sort_values(by='InternalFitness').head(20)

# Sort by Official (ExternalFitness) - ascending (lower is better)
top_official = df.sort_values(by='ExternalFitness').head(20)

print("Top 20 by Proxy Score (Internal):")
print(top_proxy[['Generation', 'InternalFitness', 'ExternalFitness']].to_string(index=False))

print("\nTop 20 by Official Score (External):")
print(top_official[['Generation', 'InternalFitness', 'ExternalFitness']].to_string(index=False))
