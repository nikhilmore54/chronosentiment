const { spawn } = require('child_process');
const http = require('http');

const configs = [
  { name: 'A', t_size: 2, r_prob: 0.20, mutation: 1.00 },
  { name: 'B', t_size: 2, r_prob: 0.20, mutation: 0.50 },
  { name: 'C', t_size: 2, r_prob: 0.20, mutation: 0.25 },
];

async function fetchState() {
  return new Promise((resolve, reject) => {
    http.get('http://127.0.0.1:4002/api/state', (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          resolve(JSON.parse(data));
        } catch (e) {
          resolve(null);
        }
      });
    }).on('error', (e) => resolve(null));
  });
}

async function startServerExecution() {
  return new Promise((resolve) => {
    const req = http.request('http://127.0.0.1:4002/api/run', { method: 'POST' }, (res) => {
      resolve(res.statusCode === 200);
    });
    req.on('error', () => resolve(false));
    req.end();
  });
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function runConfig(config) {
  console.log(`\nRunning Config ${config.name} (Mutation: ${config.mutation}x)`);
  
  const server = spawn('cargo', ['run', '--bin', 'cvrp_server'], {
    env: { ...process.env, TOURNAMENT_SIZE: config.t_size, RANDOM_PARENT_PROB: config.r_prob, MUTATION_SCALE: config.mutation, FAST_MODE: "1" },
    cwd: '/Users/nikhil/ChronoSentiment_MEGA_FINAL/services/cvrp_server'
  });
  
  server.stdout.on('data', data => console.log(`stdout: ${data}`));
  server.stderr.on('data', data => console.error(`stderr: ${data}`));

  let reached = false;
  let finalState = null;
  let started = false;

  while (!reached) {
    await sleep(250);
    if (!started) {
      started = await startServerExecution();
      if (!started) continue;
    }
    const state = await fetchState();
    if (state && state.current_generation) {
      const gen = state.current_generation;
      if (gen.generation >= 10000) {
        finalState = gen;
        reached = true;
      }
    }
  }

  const { execSync } = require('child_process');
  try {
    execSync('pkill -f cvrp_server');
  } catch (e) {}
  
  console.log(`Results for Config ${config.name} at Gen ${finalState.generation}:`);
  console.log(`- Best Dist: ${finalState.best_distance.toFixed(2)}`);
  console.log(`- P10 Dist:  ${finalState.p10_distance.toFixed(2)}`);
  console.log(`- Med Dist:  ${finalState.median_distance.toFixed(2)}`);
  console.log(`- Elite Sim: ${(finalState.elite_similarity * 100).toFixed(1)}%`);
}

async function main() {
  for (const config of configs) {
    await runConfig(config);
    await sleep(2000); // wait for port to clear
  }
}

main();
