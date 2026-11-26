
import math
import sys
import os
from pathlib import Path

# Adjust path so we can import apps
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from apps.crypto_24h.rolling_state_engine import RollingStateEngine
from apps.crypto_24h.observation_store import ObservationStore

def create_mock_obs(count, start_ts=1000000, volume_fn=lambda i: 10.0):
    """Helper to create a list of mock observations."""
    obs = []
    for i in range(count):
        obs.append({
            "timestamp": start_ts + (i * 60000),
            "open": 100.0,
            "high": 105.0,
            "low": 95.0,
            "close": 100.0 + (i * 0.1), # Slight trend
            "volume": volume_fn(i)
        })
    return obs

def populate_store(store, obs_list):
    for obs in obs_list:
        store.add(obs["timestamp"], obs["open"], obs["high"], obs["low"], obs["close"], obs["volume"])

def test_warmup_gate():
    """Test that < 1440 observations yields None."""
    store = ObservationStore()
    engine = RollingStateEngine()
    
    # Add 1439 observations
    obs_list = create_mock_obs(1439)
    populate_store(store, obs_list[:-1])
    
    # Attempt update on 1438
    state = engine.update(store)
    assert state is None
    
    # Add the 1439th
    store.add(obs_list[-1]["timestamp"], obs_list[-1]["open"], obs_list[-1]["high"], obs_list[-1]["low"], obs_list[-1]["close"], obs_list[-1]["volume"])
    state = engine.update(store)
    assert state is None
    
    # Add one more (1440)
    store.add(obs_list[-1]["timestamp"] + 60000, 100.0, 105.0, 95.0, 100.0, 10.0)
    state = engine.update(store)
    assert state is not None

def test_exact_va_calculation():
    """Verify that volume_acceleration_60m exactly matches the frozen formula."""
    store = ObservationStore()
    engine = RollingStateEngine()
    
    # We need 1440 observations. 
    # The last 30 should have volume 20.0 (sum = 600)
    # The previous 30 should have volume 10.0 (sum = 300)
    # Expected VA = 600 / 300 = 2.0
    
    def vol_fn(i):
        if i >= 1440 - 30:
            return 20.0
        elif i >= 1440 - 60:
            return 10.0
        return 5.0
        
    obs_list = create_mock_obs(1440, volume_fn=vol_fn)
    populate_store(store, obs_list)
    
    state = engine.update(store)
    assert state is not None
    assert state.volume_acceleration_60m == 2.0

def test_zero_previous_volume_fallback():
    """Test that if vol_prev_30m == 0, VA gracefully falls back to 1.0."""
    store = ObservationStore()
    engine = RollingStateEngine()
    
    def vol_fn(i):
        if i >= 1440 - 30:
            return 20.0 # recent volume is non-zero
        elif i >= 1440 - 60:
            return 0.0 # previous 30m volume is exactly 0
        return 5.0
        
    obs_list = create_mock_obs(1440, volume_fn=vol_fn)
    populate_store(store, obs_list)
    
    state = engine.update(store)
    assert state is not None
    assert state.volume_acceleration_60m == 1.0

def test_no_future_leakage():
    """Test that state at time t only uses data <= t."""
    store = ObservationStore()
    engine = RollingStateEngine()
    
    obs_list = create_mock_obs(1450)
    
    # Populate exactly 1440
    populate_store(store, obs_list[:1440])
    state_t = engine.update(store)
    va_t = state_t.volume_acceleration_60m
    
    # Now add future data to the store and re-run on a NEW engine 
    # but only querying up to time t (wait, update always uses get_recent(1440), 
    # which gets the LAST 1440. So if we add future data, update() will move forward.
    # To test no future leakage, we can just assert that feeding 1450 items, the state 
    # history at index 0 (which was calculated at 1440) hasn't changed.
    
    engine2 = RollingStateEngine()
    for i in range(1450):
        store2 = ObservationStore()
        populate_store(store2, obs_list[:i+1])
        engine2.update(store2)
        
    # The first valid state emitted by engine2 should be identical to state_t
    assert len(engine2.state_history) == 11 # 1440 to 1450 inclusive
    assert engine2.state_history[0].timestamp == state_t.timestamp
    assert engine2.state_history[0].volume_acceleration_60m == va_t

def test_replay_determinism():
    """Ensure identically constructed state streams produce identical to_dict output."""
    obs_list = create_mock_obs(1445)
    
    store1 = ObservationStore()
    engine1 = RollingStateEngine()
    populate_store(store1, obs_list)
    state1 = engine1.update(store1)
    
    store2 = ObservationStore()
    engine2 = RollingStateEngine()
    populate_store(store2, obs_list)
    state2 = engine2.update(store2)
    
    assert state1.to_dict() == state2.to_dict()
