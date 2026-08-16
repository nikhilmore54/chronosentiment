# CS-P-006-N decision-value harness

Measurement only. No evolution. C.3 is not authorized.

- artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- rows: 273
- C.3 authorized: true
- protocol V (all): 0.005817
- protocol V (development, search-admissible): 0.005313
- protocol V (selection, search-admissible): 0.015103
- protocol V (evaluation, diagnostic): -0.002964

## Table A — Decision distribution by symbol

| Symbol | Slice | LONG | SHORT | NO_TRADE | Total | % LONG | % SHORT | % NO_TRADE |
|--------|-------|-----:|------:|---------:|------:|-------:|--------:|-----------:|
| HDFCBANK.NS | development | 10 | 3 | 0 | 13 | 76.9% | 23.1% | 0.0% |
| HDFCBANK.NS | selection | 10 | 3 | 0 | 13 | 76.9% | 23.1% | 0.0% |
| HDFCBANK.NS | evaluation | 11 | 2 | 0 | 13 | 84.6% | 15.4% | 0.0% |
| HDFCBANK.NS | all | 31 | 8 | 0 | 39 | 79.5% | 20.5% | 0.0% |
| ICICIBANK.NS | development | 11 | 2 | 0 | 13 | 84.6% | 15.4% | 0.0% |
| ICICIBANK.NS | selection | 11 | 2 | 0 | 13 | 84.6% | 15.4% | 0.0% |
| ICICIBANK.NS | evaluation | 10 | 3 | 0 | 13 | 76.9% | 23.1% | 0.0% |
| ICICIBANK.NS | all | 32 | 7 | 0 | 39 | 82.1% | 17.9% | 0.0% |
| INFY.NS | development | 11 | 2 | 0 | 13 | 84.6% | 15.4% | 0.0% |
| INFY.NS | selection | 11 | 2 | 0 | 13 | 84.6% | 15.4% | 0.0% |
| INFY.NS | evaluation | 12 | 1 | 0 | 13 | 92.3% | 7.7% | 0.0% |
| INFY.NS | all | 34 | 5 | 0 | 39 | 87.2% | 12.8% | 0.0% |
| RELIANCE.NS | development | 13 | 0 | 0 | 13 | 100.0% | 0.0% | 0.0% |
| RELIANCE.NS | selection | 11 | 2 | 0 | 13 | 84.6% | 15.4% | 0.0% |
| RELIANCE.NS | evaluation | 11 | 2 | 0 | 13 | 84.6% | 15.4% | 0.0% |
| RELIANCE.NS | all | 35 | 4 | 0 | 39 | 89.7% | 10.3% | 0.0% |
| TCS.NS | development | 12 | 1 | 0 | 13 | 92.3% | 7.7% | 0.0% |
| TCS.NS | selection | 9 | 4 | 0 | 13 | 69.2% | 30.8% | 0.0% |
| TCS.NS | evaluation | 10 | 3 | 0 | 13 | 76.9% | 23.1% | 0.0% |
| TCS.NS | all | 31 | 8 | 0 | 39 | 79.5% | 20.5% | 0.0% |
| IDEA.NS | development | 11 | 2 | 0 | 13 | 84.6% | 15.4% | 0.0% |
| IDEA.NS | selection | 13 | 0 | 0 | 13 | 100.0% | 0.0% | 0.0% |
| IDEA.NS | evaluation | 10 | 3 | 0 | 13 | 76.9% | 23.1% | 0.0% |
| IDEA.NS | all | 34 | 5 | 0 | 39 | 87.2% | 12.8% | 0.0% |
| MAHABANK.NS | development | 11 | 2 | 0 | 13 | 84.6% | 15.4% | 0.0% |
| MAHABANK.NS | selection | 11 | 2 | 0 | 13 | 84.6% | 15.4% | 0.0% |
| MAHABANK.NS | evaluation | 11 | 2 | 0 | 13 | 84.6% | 15.4% | 0.0% |
| MAHABANK.NS | all | 33 | 6 | 0 | 39 | 84.6% | 15.4% | 0.0% |

## Table B — Decision value by symbol

| Symbol | Slice | n | Mean V | Median V | Mean regret | Unique-best | Opp. cost | Mean V when acted |
|--------|-------|--:|-------:|---------:|------------:|------------:|----------:|------------------:|
| HDFCBANK.NS | development | 13 | 1.2230% | 2.4500% | 2.3047% | 9 (69.2%) | 0.0000% | 1.2230% |
| HDFCBANK.NS | selection | 13 | 1.4535% | 1.2597% | 1.1024% | 8 (61.5%) | 0.0000% | 1.4535% |
| HDFCBANK.NS | evaluation | 13 | 0.5846% | 0.2756% | 3.7046% | 7 (53.8%) | 0.0000% | 0.5846% |
| HDFCBANK.NS | all | 39 | 1.0870% | 1.2597% | 2.3706% | 24 (61.5%) | 0.0000% | 1.0870% |
| ICICIBANK.NS | development | 13 | 0.0505% | -0.8399% | 5.0541% | 6 (46.2%) | 0.0000% | 0.0505% |
| ICICIBANK.NS | selection | 13 | -0.6263% | -0.2866% | 3.6841% | 5 (38.5%) | 0.0000% | -0.6263% |
| ICICIBANK.NS | evaluation | 13 | 1.0004% | 0.2538% | 2.0475% | 7 (53.8%) | 0.0000% | 1.0004% |
| ICICIBANK.NS | all | 39 | 0.1415% | -0.2866% | 3.5952% | 18 (46.2%) | 0.0000% | 0.1415% |
| INFY.NS | development | 13 | 0.8938% | 1.9868% | 4.6782% | 9 (69.2%) | 0.0000% | 0.8938% |
| INFY.NS | selection | 13 | 1.0826% | 2.9829% | 3.4035% | 9 (69.2%) | 0.0000% | 1.0826% |
| INFY.NS | evaluation | 13 | 1.7588% | 0.9574% | 2.8386% | 8 (61.5%) | 0.0000% | 1.7588% |
| INFY.NS | all | 39 | 1.2451% | 1.9868% | 3.6401% | 26 (66.7%) | 0.0000% | 1.2451% |
| RELIANCE.NS | development | 13 | -0.3571% | 0.0510% | 4.3390% | 7 (53.8%) | 0.0000% | -0.3571% |
| RELIANCE.NS | selection | 13 | 1.8037% | 1.7532% | 1.8367% | 9 (69.2%) | 0.0000% | 1.8037% |
| RELIANCE.NS | evaluation | 13 | -0.9855% | -0.4038% | 4.1778% | 6 (46.2%) | 0.0000% | -0.9855% |
| RELIANCE.NS | all | 39 | 0.1537% | 0.5573% | 3.4511% | 22 (56.4%) | 0.0000% | 0.1537% |
| TCS.NS | development | 13 | 0.5889% | 1.7850% | 2.5833% | 8 (61.5%) | 0.0000% | 0.5889% |
| TCS.NS | selection | 13 | -0.3721% | -0.5787% | 4.1782% | 5 (38.5%) | 0.0000% | -0.3721% |
| TCS.NS | evaluation | 13 | 1.5587% | 2.4181% | 1.8856% | 9 (69.2%) | 0.0000% | 1.5587% |
| TCS.NS | all | 39 | 0.5919% | 1.5946% | 2.8823% | 22 (56.4%) | 0.0000% | 0.5919% |
| IDEA.NS | development | 13 | -0.7996% | -1.1429% | 8.5902% | 6 (46.2%) | 0.0000% | -0.7996% |
| IDEA.NS | selection | 13 | 2.2617% | 0.7194% | 5.6764% | 7 (53.8%) | 0.0000% | 2.2617% |
| IDEA.NS | evaluation | 13 | -4.9173% | -6.9686% | 16.1706% | 5 (38.5%) | 0.0000% | -4.9173% |
| IDEA.NS | all | 39 | -1.1517% | -1.1429% | 10.1458% | 18 (46.2%) | 0.0000% | -1.1517% |
| MAHABANK.NS | development | 13 | 2.1194% | 1.0417% | 7.4738% | 8 (61.5%) | 0.0000% | 2.1194% |
| MAHABANK.NS | selection | 13 | 4.9693% | 3.7500% | 4.8843% | 9 (69.2%) | 0.0000% | 4.9693% |
| MAHABANK.NS | evaluation | 13 | -1.0749% | -1.6129% | 6.4999% | 5 (38.5%) | 0.0000% | -1.0749% |
| MAHABANK.NS | all | 39 | 2.0046% | 0.9804% | 6.2860% | 22 (56.4%) | 0.0000% | 2.0046% |

## Aggregate protocol scalar (mean of seven instrument means of V)

| Slice | Search-admissible | Protocol V | Mean regret | Unique-best |
|-------|-------------------|-----------:|------------:|------------:|
| development | true | 0.005313 | 0.050033 | 58.2% |
| selection | true | 0.015103 | 0.035379 | 57.1% |
| evaluation | false | -0.002964 | 0.053321 | 51.6% |
| all | false | 0.005817 | 0.046245 | 55.7% |

Table A is what the policy recommended. Table B is what those recommendations subsequently generated.
Regret and unique-best are diagnostics. They cannot construct ProtocolValue. C.3 is not authorized.
