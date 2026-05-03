#!/bin/bash
cargo build --release --example live_engine
# Use head to limit ticks
python3 scripts/mock_streamer.py | head -n 1000 | MOMENTUM_VOTER_BOOTSTRAP=1 GA_BOOTSTRAP=1 REC_CONFIRM_DELTA=1 ./target/release/examples/live_engine > /tmp/run_final_eval.txt
