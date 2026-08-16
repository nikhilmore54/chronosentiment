# CS-P-006-N decision-value harness

Measurement only. No evolution. C.3 is not authorized.

- artifact: `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`
- rows: 273
- C.3 authorized: false
- protocol V (all): 0.002819
- protocol V (development, search-admissible): 0.007559
- protocol V (selection, search-admissible): 0.006724
- protocol V (evaluation, diagnostic): -0.005825

## Table A — Decision distribution by symbol

| Symbol | Slice | LONG | SHORT | NO_TRADE | Total | % LONG | % SHORT | % NO_TRADE |
|--------|-------|-----:|------:|---------:|------:|-------:|--------:|-----------:|
| HDFCBANK.NS | development | 7 | 0 | 6 | 13 | 53.8% | 0.0% | 46.2% |
| HDFCBANK.NS | selection | 7 | 0 | 6 | 13 | 53.8% | 0.0% | 46.2% |
| HDFCBANK.NS | evaluation | 4 | 0 | 9 | 13 | 30.8% | 0.0% | 69.2% |
| HDFCBANK.NS | all | 18 | 0 | 21 | 39 | 46.2% | 0.0% | 53.8% |
| ICICIBANK.NS | development | 5 | 0 | 8 | 13 | 38.5% | 0.0% | 61.5% |
| ICICIBANK.NS | selection | 7 | 0 | 6 | 13 | 53.8% | 0.0% | 46.2% |
| ICICIBANK.NS | evaluation | 1 | 0 | 12 | 13 | 7.7% | 0.0% | 92.3% |
| ICICIBANK.NS | all | 13 | 0 | 26 | 39 | 33.3% | 0.0% | 66.7% |
| INFY.NS | development | 5 | 0 | 8 | 13 | 38.5% | 0.0% | 61.5% |
| INFY.NS | selection | 7 | 0 | 6 | 13 | 53.8% | 0.0% | 46.2% |
| INFY.NS | evaluation | 5 | 0 | 8 | 13 | 38.5% | 0.0% | 61.5% |
| INFY.NS | all | 17 | 0 | 22 | 39 | 43.6% | 0.0% | 56.4% |
| RELIANCE.NS | development | 8 | 0 | 5 | 13 | 61.5% | 0.0% | 38.5% |
| RELIANCE.NS | selection | 5 | 0 | 8 | 13 | 38.5% | 0.0% | 61.5% |
| RELIANCE.NS | evaluation | 6 | 0 | 7 | 13 | 46.2% | 0.0% | 53.8% |
| RELIANCE.NS | all | 19 | 0 | 20 | 39 | 48.7% | 0.0% | 51.3% |
| TCS.NS | development | 10 | 0 | 3 | 13 | 76.9% | 0.0% | 23.1% |
| TCS.NS | selection | 3 | 0 | 10 | 13 | 23.1% | 0.0% | 76.9% |
| TCS.NS | evaluation | 5 | 0 | 8 | 13 | 38.5% | 0.0% | 61.5% |
| TCS.NS | all | 18 | 0 | 21 | 39 | 46.2% | 0.0% | 53.8% |
| IDEA.NS | development | 9 | 0 | 4 | 13 | 69.2% | 0.0% | 30.8% |
| IDEA.NS | selection | 6 | 0 | 7 | 13 | 46.2% | 0.0% | 53.8% |
| IDEA.NS | evaluation | 7 | 0 | 6 | 13 | 53.8% | 0.0% | 46.2% |
| IDEA.NS | all | 22 | 0 | 17 | 39 | 56.4% | 0.0% | 43.6% |
| MAHABANK.NS | development | 5 | 0 | 8 | 13 | 38.5% | 0.0% | 61.5% |
| MAHABANK.NS | selection | 4 | 0 | 9 | 13 | 30.8% | 0.0% | 69.2% |
| MAHABANK.NS | evaluation | 5 | 0 | 8 | 13 | 38.5% | 0.0% | 61.5% |
| MAHABANK.NS | all | 14 | 0 | 25 | 39 | 35.9% | 0.0% | 64.1% |

## Table B — Decision value by symbol

| Symbol | Slice | n | Mean V | Median V | Mean regret | Unique-best | Opp. cost | Mean V when acted |
|--------|-------|--:|-------:|---------:|------------:|------------:|----------:|------------------:|
| HDFCBANK.NS | development | 13 | 0.0755% | 0.0000% | 3.4522% | 4 (30.8%) | 2.8908% | 0.1401% |
| HDFCBANK.NS | selection | 13 | 1.0576% | 0.0000% | 1.4983% | 4 (30.8%) | 1.9176% | 1.9642% |
| HDFCBANK.NS | evaluation | 13 | 1.0282% | 0.0000% | 3.2610% | 3 (23.1%) | 4.3548% | 3.3417% |
| HDFCBANK.NS | all | 39 | 0.7204% | 0.0000% | 2.7372% | 11 (28.2%) | 3.2402% | 1.5609% |
| ICICIBANK.NS | development | 13 | 0.9527% | 0.0000% | 4.1519% | 3 (23.1%) | 3.7868% | 2.4771% |
| ICICIBANK.NS | selection | 13 | 0.5768% | 0.0000% | 2.4809% | 4 (30.8%) | 3.7947% | 1.0712% |
| ICICIBANK.NS | evaluation | 13 | 0.5801% | 0.0000% | 2.4678% | 1 (7.7%) | 2.6735% | 7.5415% |
| ICICIBANK.NS | all | 39 | 0.7032% | 0.0000% | 3.0335% | 8 (20.5%) | 3.2748% | 2.1097% |
| INFY.NS | development | 13 | 0.7024% | 0.0000% | 4.8697% | 3 (23.1%) | 5.4391% | 1.8261% |
| INFY.NS | selection | 13 | 0.4133% | 0.0000% | 4.0728% | 5 (38.5%) | 4.1041% | 0.7676% |
| INFY.NS | evaluation | 13 | 1.0375% | 0.0000% | 3.5600% | 4 (30.8%) | 4.6952% | 2.6974% |
| INFY.NS | all | 39 | 0.7177% | 0.0000% | 4.1675% | 12 (30.8%) | 4.8045% | 1.6465% |
| RELIANCE.NS | development | 13 | 0.5723% | 0.0000% | 3.4096% | 5 (38.5%) | 4.8764% | 0.9300% |
| RELIANCE.NS | selection | 13 | 0.1841% | 0.0000% | 3.4563% | 3 (23.1%) | 4.1755% | 0.4786% |
| RELIANCE.NS | evaluation | 13 | -1.1914% | 0.0000% | 4.3836% | 2 (15.4%) | 1.8712% | -2.5813% |
| RELIANCE.NS | all | 39 | -0.1450% | 0.0000% | 3.7498% | 10 (25.6%) | 3.5442% | -0.2976% |
| TCS.NS | development | 13 | -0.0890% | 0.0000% | 3.2612% | 6 (46.2%) | 3.2382% | -0.1157% |
| TCS.NS | selection | 13 | 0.7432% | 0.0000% | 3.0629% | 2 (15.4%) | 3.7007% | 3.2206% |
| TCS.NS | evaluation | 13 | 1.1222% | 0.0000% | 2.3221% | 4 (30.8%) | 3.1145% | 2.9178% |
| TCS.NS | all | 39 | 0.5922% | 0.0000% | 2.8820% | 12 (30.8%) | 3.4113% | 1.2830% |
| IDEA.NS | development | 13 | 1.5852% | 0.0000% | 6.2054% | 4 (30.8%) | 9.1475% | 2.2897% |
| IDEA.NS | selection | 13 | -0.7635% | 0.0000% | 8.7017% | 2 (15.4%) | 9.9168% | -1.6543% |
| IDEA.NS | evaluation | 13 | -4.3404% | 0.0000% | 15.5938% | 2 (15.4%) | 8.1328% | -8.0608% |
| IDEA.NS | all | 39 | -1.1729% | 0.0000% | 10.1670% | 8 (20.5%) | 9.1061% | -2.0792% |
| MAHABANK.NS | development | 13 | 1.4924% | 0.0000% | 8.1009% | 4 (30.8%) | 10.9163% | 3.8802% |
| MAHABANK.NS | selection | 13 | 2.4950% | 0.0000% | 7.3587% | 3 (23.1%) | 10.1063% | 8.1086% |
| MAHABANK.NS | evaluation | 13 | -2.3140% | 0.0000% | 7.7390% | 1 (7.7%) | 4.8356% | -6.0165% |
| MAHABANK.NS | all | 39 | 0.5578% | 0.0000% | 7.7329% | 8 (20.5%) | 8.6789% | 1.5538% |

## Aggregate protocol scalar (mean of seven instrument means of V)

| Slice | Search-admissible | Protocol V | Mean regret | Unique-best |
|-------|-------------------|-----------:|------------:|------------:|
| development | true | 0.007559 | 0.047787 | 31.9% |
| selection | true | 0.006724 | 0.043759 | 25.3% |
| evaluation | false | -0.005825 | 0.056182 | 18.7% |
| all | false | 0.002819 | 0.049243 | 25.3% |

Table A is what the policy recommended. Table B is what those recommendations subsequently generated.
Regret and unique-best are diagnostics. They cannot construct ProtocolValue. C.3 is not authorized.
