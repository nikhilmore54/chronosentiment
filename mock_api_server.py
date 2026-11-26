import random
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Optional, Any, Dict

app = FastAPI(title="ChronoSentiment Mock API", version="2026.1")
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000", "http://127.0.0.1:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

ASSETS = ["RELIANCE.NS","TCS.NS","INFY.NS","HDFCBANK.NS","ICICIBANK.NS",
          "AXISBANK.NS","BHARTIARTL.NS","SBIN.NS","ITC.NS","LT.NS"]
# IDs use short format: strat_{queue}_{base}_{tp}_{sl}
# parseable by parseStrategyParamsFromId() in strategyId.js
STRATEGY_IDS = ["strat_200_5_4_2","strat_150_3_3_1",
                "strat_300_8_6_3","strat_100_2_2_1",
                "strat_250_6_5_2"]

def make_strategy_eval(strategy_id, seed=42):
    rng = random.Random(hash(strategy_id + str(seed)) & 0xFFFFFFFF)
    avg_pnl = round(rng.uniform(-0.02, 0.08), 4)
    std_pnl = round(rng.uniform(0.01, 0.04), 4)
    return {
        "strategy_id": strategy_id,
        "execution_fitness": round(rng.uniform(0.45, 0.92), 6),
        "ga_fitness": round(rng.uniform(30, 95), 2),
        "avg": avg_pnl,
        "std": std_pnl,
        "sharpe": round(avg_pnl / (std_pnl + 1e-9), 3),
        "max_drawdown": round(rng.uniform(0.01, 0.12), 4),
        "fill_rate": round(rng.uniform(0.70, 0.99), 4),
        "slippage": round(rng.uniform(0.0001, 0.003), 5),
        "certification_state": "CERTIFIED",
        "certification_reason": "All guardrails passed. Execution fitness within bounds.",
        "classification": rng.choice(["ALPHA","BETA","GAMMA"]),
    }

def make_narrative_blocks(strategy_id, seed=42):
    rng = random.Random(hash(strategy_id + str(seed) + "nb") & 0xFFFFFFFF)
    groups = ["INTENT","QUEUE","EXECUTION","SETTLEMENT","GOVERNANCE"]
    templates = [
        "Signal threshold crossed. Entry mandate issued for {asset} at {price:.2f}.",
        "Order queued at priority level {level}. Queue depth: {depth}.",
        "Order partially filled: {pct:.0f}% at {price:.2f}. Slippage within bounds.",
        "Order fully executed at {price:.2f}. Fill latency: {lat}ns.",
        "Settlement confirmed. Net PnL: {pnl:+.4f}. Position closed.",
        "Governor state: NOMINAL. Throttle: OPEN. Cohort integrity verified.",
        "Causal chain validated. Trace signature committed.",
        "Queue progression: position advanced from slot {s1} to slot {s2}.",
        "Risk guardrail check passed. Spread: {spread:.4f}. Volume: {vol}.",
        "Execution mandate certified. Strategy fitness: {fit:.4f}.",
    ]
    asset = rng.choice(ASSETS)
    blocks = []
    for i in range(rng.randint(6, 12)):
        group = groups[min(i, len(groups)-1)] if i < len(groups) else rng.choice(groups)
        text = rng.choice(templates).format(
            asset=asset, price=round(rng.uniform(1000,4000),2),
            level=rng.randint(1,5), depth=rng.randint(1,20),
            pct=round(rng.uniform(30,100),1), lat=rng.randint(50000,500000),
            pnl=round(rng.uniform(-0.05,0.12),4), s1=i, s2=i+1,
            spread=round(rng.uniform(0.0001,0.003),4),
            vol=rng.randint(5000,500000), fit=round(rng.uniform(0.5,0.95),4),
        )
        is_key = rng.random() < 0.3
        blocks.append({
            "sequence_id": i+1,
            "parent_sequence_id": i if i > 0 else None,
            "group": group,
            "block_type": "PRIMARY" if i == 0 else rng.choice(["PRIMARY","DERIVED","CAUSAL_LINK"]),
            "narrative": text,
            "timestamp_ns": 1748500000000000000 + i*1_000_000_000 + rng.randint(0,999_999_999),
            "is_key_event": is_key,
            "key_event_marker": "FILL" if is_key else None,
            "divergence_score": None,
        })
    return blocks

def make_execution_trace(strategy_id, seed=42):
    rng = random.Random(hash(strategy_id + str(seed) + "et") & 0xFFFFFFFF)
    types = ["ORDER_PLACED","ORDER_FILLED","ORDER_CANCELLED","POSITION_OPENED","POSITION_CLOSED"]
    return [{
        "sequence_id": i+1,
        "type": rng.choice(types),
        "timestamp_ns": 1748500000000000000 + i*500_000_000,
        "payload": {"asset": rng.choice(ASSETS), "price": round(rng.uniform(1000,4000),2),
                    "quantity": rng.randint(1,100), "side": rng.choice(["BUY","SELL"])},
    } for i in range(rng.randint(8,16))]

def make_signals(seed=42):
    rng = random.Random(seed)
    signals = []
    for asset in rng.sample(ASSETS, k=rng.randint(5,9)):
        action = rng.choice(["BUY","SELL","HOLD"])
        if action == "HOLD":
            signals.append({"asset":asset,"action":"HOLD","confidence":0.0,
                            "composite_score":0.0,"strategy_id":rng.choice(STRATEGY_IDS)})
            continue
        base = rng.uniform(1000,4000)
        signals.append({
            "asset": asset, "action": action,
            "confidence": round(rng.uniform(0.4,0.95),4),
            "composite_score": round(rng.uniform(0.0,2.0),4),
            "entry_zone": [round(base*0.995,2), round(base*1.005,2)],
            "target": round(base*(1.02 if action=="BUY" else 0.98),2),
            "stop_loss": round(base*(0.99 if action=="BUY" else 1.01),2),
            "scenario_pnl": round(rng.uniform(-0.05,0.12),4),
            "strategy_id": rng.choice(STRATEGY_IDS),
        })
    return signals

def make_generation_history(seed=42, generations=20):
    rng = random.Random(seed)
    history = []
    fitness = rng.uniform(20,40)
    for g in range(generations):
        fitness = min(95.0, fitness + rng.uniform(-2,5))
        history.append({
            "generation": g,
            "ga_fitness": round(fitness,2),
            "execution_fitness": round(min(0.95, 0.3+g*0.03+rng.uniform(-0.02,0.04)),6),
            "avg": round(rng.uniform(-0.01,0.08),4),
            "strategy_id": rng.choice(STRATEGY_IDS),
        })
    return history

@app.get("/health")
def health():
    return {"status":"online","engine":"ChronoSentiment 2.0 (Phase C.1.6b)",
            "system_phase":"LIVE","throttle_state":"OPEN","cohort_id":"cohort-2026-A"}

@app.get("/observatory")
def observatory():
    return {
        "snapshot_sequence_id": 4821,
        "system_phase": "LIVE",
        "governor_state": {"throttle_state":"OPEN","cohort_id":"cohort-2026-A",
                           "active_cohort_size":12,"governor_version":"v3.2.1"},
        "kernel_state": {"queue_depth":3,"fill_latency_ns":142000,
                         "sync_ratio":0.9987,"events_per_second":847,"kernel_version":"v2.8.0"},
    }

@app.get("/ga/strategy-store")
def strategy_store():
    strategies = [make_strategy_eval(sid) for sid in STRATEGY_IDS]
    return {"strategies":strategies,"store_version":"2026-05-29T09:00:00Z","total":len(strategies)}

@app.get("/ga/global-ranking")
def global_ranking():
    rows = []
    for sid in STRATEGY_IDS:
        ev = make_strategy_eval(sid)
        rows.append({"strategy_id":sid,"ga_fitness":ev["ga_fitness"],
                     "execution_fitness":ev["execution_fitness"],"avg":ev["avg"],
                     "std":ev["std"],"classification":ev["classification"]})
    rows.sort(key=lambda r: r["ga_fitness"], reverse=True)
    for i,r in enumerate(rows): r["rank"] = i+1
    return {"rankings":rows,"total":len(rows)}

class RunGARequest(BaseModel):
    population_size: Optional[int] = 50
    generations: Optional[int] = 20
    mutation_rate: Optional[float] = 0.1
    seed: Optional[int] = 42

def _run_ga_impl(seed, gens):
    history = make_generation_history(seed=seed, generations=gens)
    best_entry = max(history, key=lambda e: e["ga_fitness"])
    best_idx = history.index(best_entry)
    global_best = make_strategy_eval(best_entry["strategy_id"], seed=seed)
    global_best["ga_fitness"] = best_entry["ga_fitness"]
    global_best["avg"] = best_entry["avg"]
    return {"global_best":global_best,"global_best_generation":best_idx,
            "final_generation_best":history[-1],"generation_history":history,
            "total_generations":gens,"seed":seed}

@app.get("/run_ga")
async def run_ga_get():
    return _run_ga_impl(42, 20)

@app.post("/run_ga")
async def run_ga_post(req: RunGARequest):
    return _run_ga_impl(req.seed or 42, req.generations or 20)

@app.get("/signals/latest")
def signals_latest():
    sigs = make_signals(seed=9371)
    return {"signals":sigs,"snapshot_ts":"2026-05-29T09:00:00Z","total":len(sigs)}

class InspectRequest(BaseModel):
    strategy_id: str
    seed: Optional[int] = 42

@app.post("/inspect_strategy")
def inspect_strategy(req: InspectRequest):
    seed = req.seed or 42
    ev = make_strategy_eval(req.strategy_id, seed=seed)
    rng = random.Random(hash(req.strategy_id + str(seed)) & 0xFFFFFFFF)
    return {
        "strategy_id": req.strategy_id, "seed": seed,
        "execution_fitness": ev["execution_fitness"], "ga_fitness": ev["ga_fitness"],
        "certification_state": ev["certification_state"],
        "certification_reason": ev["certification_reason"],
        "verdict": rng.choice(["EXECUTE","HOLD","REVIEW"]),
        "confidence": rng.choice(["HIGH","MEDIUM","LOW"]),
        "narrative_blocks": make_narrative_blocks(req.strategy_id, seed=seed),
        "execution_trace": make_execution_trace(req.strategy_id, seed=seed),
        "decision_trace": [], "event_sequence": [],
        "avg": ev["avg"], "std": ev["std"], "sharpe": ev["sharpe"],
        "max_drawdown": ev["max_drawdown"], "fill_rate": ev["fill_rate"],
        "slippage": ev["slippage"],
    }

class StrategyEntry(BaseModel):
    strategy_config: Dict[str, Any]

class CompareRequest(BaseModel):
    strategies: List[StrategyEntry]
    scenarios: Optional[List[Any]] = []
    seed: Optional[int] = 42

@app.post("/compare_strategies")
def compare_strategies(req: CompareRequest):
    seed = req.seed or 42
    results = []
    for entry in req.strategies:
        cfg = entry.strategy_config
        sid = (cfg.get("strategy_id") or cfg.get("id") or
               "strat_{}_w{}_sl{}_tp{}_seed{}".format(
                   cfg.get("lookback",200), cfg.get("window",5),
                   cfg.get("stop_loss",2), cfg.get("take_profit",4), seed))
        results.append(make_strategy_eval(sid, seed=seed))
    # Sort by execution_fitness descending for ranking table
    results.sort(key=lambda r: r["execution_fitness"], reverse=True)
    if len(results) >= 2:
        best = results[0]
        conclusion = ("Strategy '{}' achieves superior execution fitness ({:.4f}) "
                      "with Sharpe {:.3f}. Recommended for live deployment pending "
                      "governor approval.".format(
                          best["strategy_id"], best["execution_fitness"], best["sharpe"]))
    else:
        conclusion = "Insufficient strategies for comparison."
    return {
        # 'ranking' matches CompareStrategies.js line 158: comparisonResult.ranking
        "ranking": results,
        "seed": seed,
        # 'comparison_summary' matches CompareStrategies.js line 194
        "comparison_summary": {
            "reason": conclusion,
            "replay_integrity": "CERTIFIED",
            "timestamp_cohesion": "VALID",
            "sync_state": "NOMINAL",
            "governor_action": "APPROVED",
            "replay_certified": True,
            "metrics": [
                {"key": "fill_rate_avg", "expected": 0.85,
                 "observed": round(sum(r["fill_rate"] for r in results)/max(len(results),1),4),
                 "diverged": False},
                {"key": "slippage_avg", "expected": 0.001,
                 "observed": round(sum(r["slippage"] for r in results)/max(len(results),1),5),
                 "diverged": False},
                {"key": "sharpe_avg", "expected": 1.5,
                 "observed": round(sum(r["sharpe"] for r in results)/max(len(results),1),3),
                 "diverged": False},
            ],
        },
    }

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000, log_level="info")