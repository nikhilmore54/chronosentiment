import json
import random

def generate_ticks(asset, num_events, start_price):
    events = []
    current_price = start_price
    timestamp = 1710000000000
    
    # Pre-generate some price path with volatility
    price_path = [start_price]
    for _ in range(num_events):
        # Increased volatility: 0.1% to 0.5% moves
        change = current_price * random.uniform(-0.005, 0.005)
        current_price += change
        price_path.append(current_price)
        
    for i in range(num_events):
        timestamp += 100 # 100ms between events
        price = price_path[i]
        
        # Depth event
        spread = price * 0.0001 # 1 bp spread
        bid = price - spread/2
        ask = price + spread/2
        
        depth_payload = {
            "e": "depthUpdate",
            "E": timestamp,
            "s": asset,
            "U": i*2,
            "u": i*2 + 1,
            "b": [[f"{bid:.2f}", "1.0"]],
            "a": [[f"{ask:.2f}", "1.0"]]
        }
        
        events.append({
            "asset": asset,
            "type": "depth",
            "payload": json.dumps(depth_payload)
        })
        
        # Trade event
        timestamp += 50
        trade_payload = {
            "e": "trade",
            "E": timestamp,
            "s": asset,
            "t": i,
            "p": f"{price:.2f}",
            "q": f"{random.uniform(100, 1000):.4f}",
            "T": timestamp,
            "m": random.choice([True, False])
        }
        
        events.append({
            "asset": asset,
            "type": "trade",
            "payload": json.dumps(trade_payload)
        })
        
    return events

if __name__ == "__main__":
    btc_events = generate_ticks("BTCUSDT", 5000, 65000.0)
    with open("test_assets/binance_ticks.jsonl", "w") as f:
        for e in btc_events:
            f.write(json.dumps(e) + "\n")
    print(f"Generated {len(btc_events)} events for BTCUSDT")
