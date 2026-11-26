#!/bin/bash
set -e

BASE="http://localhost:3001"

echo "=== STEP 1: Health ==="
curl -s "$BASE/api/health"
echo ""

echo ""
echo "=== STEP 2: Import — Staff Data (30 nurses) ==="
curl -s "$BASE/api/nurses" | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'Nurses loaded: {len(d[\"nurses\"])}')"

echo ""
echo "=== STEP 3: Export Formats ==="
curl -s "$BASE/api/export/formats" | python3 -c "
import sys,json
d=json.load(sys.stdin)
for f in d:
    print(f'  {f[\"id\"]}: {f[\"name\"]}')
"

echo ""
echo "=== STEP 4: Generate Schedule ==="
cat > /tmp/sched_req.json << 'ENDJSON'
{
  "workers": [
    {"id": 1, "skills": [{"0": "Nurse"}]},
    {"id": 2, "skills": [{"0": "Nurse"}]},
    {"id": 3, "skills": [{"0": "Nurse"}]},
    {"id": 4, "skills": [{"0": "Nurse"}]},
    {"id": 5, "skills": [{"0": "Nurse"}]}
  ],
  "shifts": [
    {"id": 101, "start_hour": 0,  "duration_hours": 8, "required_skill": {"0": "Nurse"}},
    {"id": 102, "start_hour": 8,  "duration_hours": 8, "required_skill": {"0": "Nurse"}},
    {"id": 103, "start_hour": 16, "duration_hours": 8, "required_skill": {"0": "Nurse"}},
    {"id": 104, "start_hour": 24, "duration_hours": 8, "required_skill": {"0": "Nurse"}},
    {"id": 105, "start_hour": 32, "duration_hours": 8, "required_skill": {"0": "Nurse"}}
  ],
  "generation_limit": 100
}
ENDJSON

SCHED_RESP=$(curl -s -X POST "$BASE/api/schedule" \
  -H "Content-Type: application/json" \
  -d @/tmp/sched_req.json)

echo "$SCHED_RESP" | python3 -c "
import sys,json
d=json.load(sys.stdin)
print(f'Response keys: {list(d.keys())}')
cr = d.get('constraint_report', {})
print(f'  is_valid={cr.get(\"is_valid\")}')
print(f'  hard_violations={cr.get(\"hard_violations\")}')
print(f'  fitness={cr.get(\"fitness\")}')
sched = d.get('schedule', {})
print(f'  assignments in schedule: {len(sched)}')
recs = d.get('recommendations', [])
print(f'  recommendations: {len(recs)}')
if recs:
    print(f'  first rec: {recs[0].get(\"explanation\",\"\")[:80]}')
"

echo "$SCHED_RESP" > /tmp/sched_resp.json

echo ""
echo "=== STEP 5a: Export as CSV ==="
curl -s -X POST "$BASE/api/export/csv" \
  -H "Content-Type: application/json" \
  -d @/tmp/sched_resp.json | head -c 500
echo ""

echo ""
echo "=== STEP 5b: Export as JSON ==="
curl -s -X POST "$BASE/api/export/json" \
  -H "Content-Type: application/json" \
  -d @/tmp/sched_resp.json | head -c 300
echo ""

echo ""
echo "=== STEP 5c: Export as ICS ==="
curl -s -X POST "$BASE/api/export/ics" \
  -H "Content-Type: application/json" \
  -d @/tmp/sched_resp.json | head -c 300
echo ""

echo ""
echo "=== BONUS: Dashboard ==="
curl -s "$BASE/api/dashboard" | python3 -c "
import sys,json
d=json.load(sys.stdin)
print(f'Dashboard keys: {list(d.keys())}')
rh = d.get('roster_health', {})
print(f'  legality_score={rh.get(\"legality_score\")}')
print(f'  coverage_score={rh.get(\"coverage_score\")}')
alerts = d.get('alerts', [])
print(f'  alerts: {len(alerts)}')
"

echo ""
echo "=== ALL STEPS COMPLETE ==="