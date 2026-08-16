# Selected PolicyArtifact

Sealed after **selection** on the selection slice. Immutable. Not retuned against evaluation.

- policy: `coralys.rulelist.discovered@v0`
- discovery_engine: `coralys.moga.rulelist.v0`
- discovery_run_id: `coralys.rulelist.6e92ef3e097d52f9`
- artifact_hash: `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`
- methodology_hash: `6e92ef3e097d52f923b6028258f6442bcb5de6163c45a94628dead9aa954e3a5`
- genome identity: `d8363a93e5afe518b7a4cbb8f5c3ac59efcf396f0d318ccdae0dd683e9d730d3`
- n_rules: 1
- unmatched_action: NoTrade
- allowed_information_timestamp: 2022-10-31 15:30:00 UTC
- development mean signed traded return: 0.016325 (traded 49, stood aside 42)
- selection mean signed traded return: 0.019938 (traded 39, stood aside 52)

Rules are the search result, not a hand-written mapping.

```json
[
  {
    "when": [
      {
        "concept": "Trend",
        "present": true,
        "direction": "Bearish"
      }
    ],
    "action": "LONG"
  }
]
```
