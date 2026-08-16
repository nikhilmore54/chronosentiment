# Selected PolicyArtifact — Search #2

Sealed after selection on the selection slice. Not retuned against evaluation.

- artifact_hash: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- methodology_hash: `eff198957d799419035a5b86f6adceee6233bfa626f5ff2fee39d59132d99a99`
- genome identity: `50709d968b90bec17e6904d6f1daf9c16ba636a8b9dfea4fa6495482fd745839`
- n_rules: 7
- unmatched_action: Long
- development value: 0.005313 (traded 91, stood aside 0)
- selection value: 0.015103 (traded 91, stood aside 0)

Rules are the search result.

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
  },
  {
    "when": [
      {
        "concept": "Momentum",
        "present": true,
        "direction": "Positive"
      },
      {
        "concept": "Trend",
        "present": true,
        "direction": "Bullish"
      },
      {
        "concept": "Volatility",
        "present": true,
        "direction": null
      }
    ],
    "action": "LONG"
  },
  {
    "when": [
      {
        "concept": "Trend",
        "present": true,
        "direction": "Neutral"
      }
    ],
    "action": "NO_TRADE"
  },
  {
    "when": [
      {
        "concept": "Trend",
        "present": true,
        "direction": "Bullish"
      }
    ],
    "action": "SHORT"
  },
  {
    "when": [
      {
        "concept": "Momentum",
        "present": true,
        "direction": "Positive"
      },
      {
        "concept": "Volatility",
        "present": true,
        "direction": null
      },
      {
        "concept": "Volatility",
        "present": false,
        "direction": null
      }
    ],
    "action": "NO_TRADE"
  },
  {
    "when": [
      {
        "concept": "Trend",
        "present": true,
        "direction": "Bullish"
      },
      {
        "concept": "Volatility",
        "present": true,
        "direction": null
      }
    ],
    "action": "LONG"
  },
  {
    "when": [
      {
        "concept": "Volatility",
        "present": true,
        "direction": null
      }
    ],
    "action": "LONG"
  }
]
```
