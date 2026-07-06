#!/bin/bash
pkill -f cvrp_server

run_config() {
  local T=$1
  local R=$2
  echo "--- Config T=$T R=$R ---"
  TOURNAMENT_SIZE=$T RANDOM_PARENT_PROB=$R FAST_MODE=1 cargo run --bin cvrp_server > server.log 2>&1 &
  PID=$!
  sleep 4
  curl -s http://127.0.0.1:4002/api/state | jq '.current_generation | {generation, best_distance, p10_distance, median_distance, elite_similarity}'
  kill -9 $PID
  sleep 1
}

run_config 2 0.20
run_config 2 0.10
run_config 2 0.05
run_config 3 0.10
