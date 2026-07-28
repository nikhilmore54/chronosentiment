# UltraCrew — Demo Quick-Start Guide

Get the full-stack pilot portal running locally in under five minutes.

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust / cargo | stable (≥ 1.75) | https://rustup.rs |
| Node.js | ≥ 16 | https://nodejs.org |
| npm | ≥ 8 (bundled with Node) | — |

---

## macOS / Linux — one command

```bash
# 1. Clone and enter the repo
git clone <repo-url> ultracrew && cd ultracrew

# 2. Make the launcher executable (one-time)
chmod +x start-demo.sh

# 3. Launch
./start-demo.sh
```

The script will:
1. Check that `cargo` and `node` are available.
2. Run `npm install` in the frontend directory on first launch.
3. Build and start the Rust backend (`cargo run --release`) on port **3001**.
4. Start the React frontend (`npm start`) on port **3000**.
5. Poll the backend health endpoint until it responds (up to 120 s for the first Rust build).
6. Open `http://localhost:3000` in your default browser.

Press **Ctrl+C** to stop both processes cleanly.

### Custom backend port

```bash
PORT=8080 ./start-demo.sh
```

---

## Windows — one command (PowerShell)

```powershell
# 1. Clone and enter the repo
git clone <repo-url> ultracrew; cd ultracrew

# 2. Allow script execution for this session
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass

# 3. Launch
.\start-demo.ps1
```

Behaviour is identical to the bash script. Logs are written to
`%TEMP%\ultracrew-backend.log` and `%TEMP%\ultracrew-frontend.log`.

Press **Ctrl+C** to stop both processes.

### Custom backend port

```powershell
$env:PORT = "8080"; .\start-demo.ps1
```

---

## Manual start (without the launcher)

### Backend

```bash
cd services/ultracrew_server
PORT=3001 cargo run --release
```

### Frontend

```bash
cd apps/ultracrew-pilot-portal
npm install   # first time only
npm start
```

Then open `http://localhost:3000`.

---

## Verifying the backend is up

```bash
curl http://localhost:3001/api/health
# → {"status":"ok"}
```

---

## Log files

| Process | macOS / Linux | Windows |
|---------|---------------|---------|
| Backend | `/tmp/ultracrew-backend.log` | `%TEMP%\ultracrew-backend.log` |
| Frontend | `/tmp/ultracrew-frontend.log` | `%TEMP%\ultracrew-frontend.log` |

Tail the backend log in real time:

```bash
tail -f /tmp/ultracrew-backend.log
```

---

## Troubleshooting

**Backend build fails** — ensure you have the latest stable Rust toolchain:
```bash
rustup update stable
```

**Port already in use** — kill the existing process or use a different port:
```bash
lsof -ti:3001 | xargs kill   # macOS / Linux
```

**Frontend `npm install` errors** — delete `node_modules` and retry:
```bash
rm -rf apps/ultracrew-pilot-portal/node_modules
./start-demo.sh
```

**Simulation endpoints return 503** — this is expected. The simulation
endpoints (`/api/state`, `/api/balance`, `/api/dashboard`, `/api/nurses`)
are not used by the pilot portal. The portal uses `/api/schedule`,
`/api/pairings`, `/api/duties`, and `/api/pilot/session` — all of which
work immediately on startup.

---

## Demo flow (facilitator guide)

| Step | Screen | Customer question answered |
|------|--------|---------------------------|
| 1 | Identity & context | Who are you and what operation are we looking at? |
| 2 | Run optimizer | What does your current schedule look like? |
| 3 | Review schedule | What changes did the system make? |
| 4 | Recommendations | Why did it choose those assignments? |
| 5 | Disruption recovery | What happens when something goes wrong? |
| 6 | Session debrief | What is this worth and what are the next steps? |

---

## Project structure (relevant paths)

```
ultracrew/
├── start-demo.sh              # macOS / Linux launcher
├── start-demo.ps1             # Windows PowerShell launcher
├── DEMO_QUICKSTART.md         # this file
├── services/
│   └── ultracrew_server/      # Rust / Axum backend (port 3001)
│       └── src/main.rs
└── apps/
    └── ultracrew-pilot-portal/ # React / TypeScript frontend (port 3000)
        └── src/App.js