#!/usr/bin/env python3
"""
RP-608-REGIME-BREAKOUT-001-PROSPECTIVE-v1 — clock, DQ gates, namespace isolation.

Observation counting is blocked while authorization.authorized is false.
Integrity window (sessions 1–5) forbids strategy/threshold tuning.
Live order authority remains 0.0%.
"""

from __future__ import annotations

import hashlib
import json
import os
import tempfile
from dataclasses import asdict, dataclass, field
from datetime import date, datetime, time, timedelta, timezone
from enum import Enum
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Set, Tuple

IST = timezone(timedelta(hours=5, minutes=30))
START_RULE_VERSION = "1.0"
NSE_OPEN = time(9, 15)
NSE_CLOSE = time(15, 30)

WORKSPACE = Path(__file__).resolve().parent.parent
ARTIFACT_ROOT = WORKSPACE / "artifacts" / "rp608_regime_breakout"
PROSPECTIVE_NS = ARTIFACT_ROOT / "prospective_v1"
AUTH_PATH = ARTIFACT_ROOT / "prospective_v1_authorization.json"
GATE4_NS = ARTIFACT_ROOT / "historical_abc_research"
LEGACY_ARM_C_AUDIT = ARTIFACT_ROOT / "arm_c_qualification_readonly_audit.json"

PROTECTED_PATHS = (
    GATE4_NS,
    LEGACY_ARM_C_AUDIT,
    ARTIFACT_ROOT / "arm_c_qualification_readonly_audit_corrected_packing.json",
    ARTIFACT_ROOT / "gate4_reconciliation_note.md",
    ARTIFACT_ROOT / "arm_c_measurement_disposition.md",
)

PROTOCOL_ID = "RP-608-REGIME-BREAKOUT-001-PROSPECTIVE-v1"
BASE_WINDOW_SESSIONS = 20
INTEGRITY_THROUGH = 5
EXTENSION_BLOCK = 10
MAX_EXTENSIONS = 2
QUOTE_FRESHNESS_MS = 500.0
LABEL = "prospective_shadow"


class Market(str, Enum):
    NSE = "NSE"
    CRYPTO = "CRYPTO"


class Phase(str, Enum):
    NOT_STARTED = "NOT_STARTED"
    INTEGRITY = "INTEGRITY"  # 1-5
    COLLECTION = "COLLECTION"  # 6-10
    COMPLETION = "COMPLETION"  # 11-20(+ext)
    DECISION = "DECISION"
    INCONCLUSIVE_EXTENDABLE = "INCONCLUSIVE_EXTENDABLE"


class CommencementState(str, Enum):
    """Authoritative per-market commencement states (protocol §10)."""

    NOT_STARTED = "NOT_STARTED"
    STARTED = "STARTED"
    BLOCKED = "BLOCKED"


class Decision(str, Enum):
    PASS = "pass"
    FAIL = "fail"
    INCONCLUSIVE = "INCONCLUSIVE"
    PENDING = "PENDING"


@dataclass
class DataQualityGate:
    completed_bars_only: bool = True
    quote_age_ms: Optional[float] = None
    half_spread_bps: Optional[float] = None
    half_spread_cap_bps: Optional[float] = None
    dedupe_key_present: bool = False
    journal_complete: bool = False
    packing_canonical: bool = False

    def failures(self) -> List[str]:
        fails: List[str] = []
        if not self.completed_bars_only:
            fails.append("forming_bar_leakage")
        if self.quote_age_ms is not None and self.quote_age_ms > QUOTE_FRESHNESS_MS:
            fails.append("stale_quote")
        if (
            self.half_spread_bps is not None
            and self.half_spread_cap_bps is not None
            and self.half_spread_bps > self.half_spread_cap_bps
        ):
            fails.append("spread_cap")
        if not self.dedupe_key_present:
            fails.append("missing_dedupe")
        if not self.journal_complete:
            fails.append("incomplete_journal")
        if not self.packing_canonical:
            fails.append("non_canonical_packing")
        return fails

    @property
    def ok(self) -> bool:
        return not self.failures()


@dataclass
class MarketClock:
    market: Market
    completed_units: int = 0  # NSE sessions or crypto time-blocks
    extensions_used: int = 0
    integrity_review_done: bool = False
    tuning_forbidden: bool = True
    unit_ids: List[str] = field(default_factory=list)

    @property
    def window_target(self) -> int:
        return BASE_WINDOW_SESSIONS + self.extensions_used * EXTENSION_BLOCK

    def phase(self) -> Phase:
        n = self.completed_units
        if n <= 0:
            return Phase.NOT_STARTED
        if n <= INTEGRITY_THROUGH:
            return Phase.INTEGRITY
        if n <= 10:
            return Phase.COLLECTION
        if n < self.window_target:
            return Phase.COMPLETION
        return Phase.DECISION

    def may_count(self, *, authorized: bool, commencement_started: bool = False) -> Tuple[bool, str]:
        if not authorized:
            return False, "authorization_authorized_false"
        if not commencement_started:
            return False, "commencement_not_started"
        if not self.tuning_forbidden:
            return False, "tuning_flag_cleared_illegally"
        return True, "ok"

    def record_unit(
        self,
        unit_id: str,
        *,
        authorized: bool,
        dq: DataQualityGate,
        commencement_started: bool = False,
    ) -> Dict[str, Any]:
        allowed, reason = self.may_count(
            authorized=authorized, commencement_started=commencement_started
        )
        if not allowed:
            return {"accepted": False, "reason": reason, "completed_units": self.completed_units}
        if not dq.ok:
            return {
                "accepted": False,
                "reason": "data_quality_gate_failed",
                "failures": dq.failures(),
                "completed_units": self.completed_units,
            }
        if unit_id in self.unit_ids:
            return {"accepted": False, "reason": "duplicate_unit", "completed_units": self.completed_units}
        self.unit_ids.append(unit_id)
        self.completed_units += 1
        if self.completed_units == INTEGRITY_THROUGH:
            self.integrity_review_done = True
        return {
            "accepted": True,
            "unit_id": unit_id,
            "completed_units": self.completed_units,
            "phase": self.phase().value,
            "integrity_review_due": self.completed_units == INTEGRITY_THROUGH,
            "tuning_allowed": False,
        }

    def authorize_extension(self, *, decision: Decision) -> Dict[str, Any]:
        if decision != Decision.INCONCLUSIVE:
            return {"accepted": False, "reason": "extension_only_after_inconclusive"}
        if self.completed_units < self.window_target:
            return {"accepted": False, "reason": "window_not_complete"}
        if self.extensions_used >= MAX_EXTENSIONS:
            return {"accepted": False, "reason": "max_extensions_reached"}
        self.extensions_used += 1
        return {
            "accepted": True,
            "extension_index": self.extensions_used,
            "new_window_target": self.window_target,
            "criteria_relaxed": False,
        }


def load_authorization(path: Optional[Path] = None) -> Dict[str, Any]:
    p = AUTH_PATH if path is None else path
    if not p.exists():
        return {"authorized": False, "observation_counting": "NOT_STARTED"}
    return json.loads(p.read_text(encoding="utf-8"))


def is_authorized(auth: Optional[Dict[str, Any]] = None) -> bool:
    doc = auth if auth is not None else load_authorization()
    return bool(doc.get("authorized") is True)


def read_deployed_rev() -> str:
    stamped = PROSPECTIVE_NS / "DEPLOYED_REV.txt"
    if stamped.exists():
        return stamped.read_text(encoding="utf-8").strip()
    try:
        import subprocess

        return subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=str(WORKSPACE), text=True
        ).strip()
    except Exception:
        return ""


def load_preflight(path: Optional[Path] = None) -> Dict[str, Any]:
    p = path if path is not None else (ARTIFACT_ROOT / "prospective_v1_preflight.json")
    if not p.exists():
        return {}
    return json.loads(p.read_text(encoding="utf-8"))


def authorize_prospective_v1(
    *,
    authorized_by: str,
    intent: str,
    expected_deployed_rev: str,
    confirm: bool,
    live_order_authority_pct: float = 0.0,
) -> Dict[str, Any]:
    """
    Prescribed authorization path. Validates preflight + deployed rev, then
    persists authorized=true with human provenance.

    Does NOT start Session 1 / does NOT advance the experiment clock.
    observation_counting remains NOT_STARTED until session-start criteria are met.
    """
    if not confirm:
        raise ValueError("refusing authorize without confirm=True")
    if not (authorized_by or "").strip():
        raise ValueError("authorized_by is required")
    if not (intent or "").strip():
        raise ValueError("intent statement is required")
    if live_order_authority_pct != 0.0:
        raise ValueError("live order authority must remain 0.0%")

    ensure_namespace()
    prev = load_authorization()
    if prev.get("authorized") is True:
        raise ValueError("already authorized; refuse duplicate authorize without new protocol version")

    deployed = read_deployed_rev()
    expected = (expected_deployed_rev or "").strip().rstrip("…")
    if not expected:
        raise ValueError("expected_deployed_rev is required")
    if not deployed:
        raise ValueError("deployed revision unavailable (DEPLOYED_REV.txt / git)")
    # Require exact full-rev match, or unambiguous prefix of the deployed full rev.
    if deployed != expected and not deployed.startswith(expected):
        raise ValueError(
            f"deployed revision mismatch: deployed={deployed} expected={expected}"
        )
    if len(expected) < 12:
        raise ValueError("expected_deployed_rev must be at least 12 characters")

    preflight = load_preflight()
    if not preflight.get("preflight_pass"):
        raise ValueError("preflight_pass is not true; refuse authorize")
    if float(preflight.get("controls", {}).get("live_order_authority_pct", 0.0)) != 0.0:
        raise ValueError("preflight reports live authority != 0.0%")

    # Prefer exact full rev from stamp
    full_rev = deployed
    now = datetime.now(timezone.utc).isoformat()
    hashes = artifact_hashes()

    doc: Dict[str, Any] = dict(prev)
    doc.update(
        {
            "protocol_id": PROTOCOL_ID,
            "protocol_name": "RP-608-REGIME-BREAKOUT-001 Prospective V1",
            "authorized": True,
            "observation_counting": "NOT_STARTED",
            "session_1_started": False,
            "clock_start": "PENDING_SESSION_START_CRITERIA",
            "authorized_by": authorized_by.strip(),
            "authorized_at_utc": now,
            "authorization_intent": intent.strip(),
            "git_rev": full_rev,
            "deployed_revision": full_rev,
            "artifact_hashes": hashes,
            "fixture_attestation": {
                "preflight_pass": True,
                "preflight_artifact": str(
                    (ARTIFACT_ROOT / "prospective_v1_preflight.json").relative_to(WORKSPACE)
                ),
                "preflight_utc": preflight.get("preflight_utc"),
                "lifecycle_passed": (preflight.get("checks") or {})
                .get("lifecycle_fixtures", {})
                .get("passed"),
                "arm_a_passed": (preflight.get("checks") or {})
                .get("arm_a_invariance", {})
                .get("passed"),
            },
            "output_namespace": str(PROSPECTIVE_NS.relative_to(WORKSPACE)),
            "previous_status": {
                "authorized": False,
                "observation_counting": "NOT_STARTED",
                "snapshot": {
                    "authorized": prev.get("authorized"),
                    "observation_counting": prev.get("observation_counting"),
                },
            },
            "controls": {
                "arm_a": "FROZEN",
                "holdout_sessions_41_68": "SEALED",
                "live_order_authority_pct": 0.0,
                "dynamic_exits": "FORBIDDEN",
                "thresholds": "LOCKED",
                "gate4_artifacts": "IMMUTABLE_DO_NOT_OVERWRITE",
                "nse_crypto_pooling": "FORBIDDEN",
                "integrity_sessions_1_5_tuning": "FORBIDDEN",
            },
            "reason": (
                "Human authorization granted. Experiment clock remains NOT_STARTED "
                "until protocol session-start criteria are satisfied."
            ),
        }
    )

    assert_write_allowed(AUTH_PATH)
    AUTH_PATH.write_text(json.dumps(doc, indent=2) + "\n", encoding="utf-8")

    # Keep clocks at zero units; refresh authorized flag only
    clocks = load_clocks()
    save_clocks(clocks)

    validation = validate_authorization_record(doc)
    if not validation["ok"]:
        # roll back to unauthorized on failed validation
        prev["authorized"] = False
        AUTH_PATH.write_text(json.dumps(prev, indent=2) + "\n", encoding="utf-8")
        raise ValueError(f"authorization validation failed: {validation['errors']}")

    return {"ok": True, "authorization": doc, "validation": validation}


def validate_authorization_record(doc: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
    d = doc if doc is not None else load_authorization()
    errors: List[str] = []
    if d.get("authorized") is not True:
        errors.append("authorized_not_true")
    for key in (
        "authorized_by",
        "authorized_at_utc",
        "git_rev",
        "artifact_hashes",
        "fixture_attestation",
        "output_namespace",
        "clock_start",
    ):
        if not d.get(key):
            errors.append(f"missing_{key}")
    if d.get("controls", {}).get("live_order_authority_pct") != 0.0:
        errors.append("live_authority_not_zero")
    if d.get("controls", {}).get("holdout_sessions_41_68") != "SEALED":
        errors.append("holdout_not_sealed")
    if d.get("session_1_started") is True:
        errors.append("session_1_must_not_auto_start_on_authorize")
    if d.get("observation_counting") not in ("NOT_STARTED", "STARTED"):
        errors.append("invalid_observation_counting")
    # Counting may be STARTED only after a persisted per-market start event.
    if d.get("observation_counting") == "STARTED":
        started_markets = d.get("markets_started") or {}
        if not any(started_markets.values()):
            errors.append("counting_started_without_market_start_event")
    return {"ok": not errors, "errors": errors}


def ensure_namespace() -> Path:
    """Create prospective_v1 namespace; never touch Gate 4 paths."""
    PROSPECTIVE_NS.mkdir(parents=True, exist_ok=True)
    readme = PROSPECTIVE_NS / "README.md"
    if not readme.exists():
        readme.write_text(
            "\n".join(
                [
                    "# Prospective V1 output namespace",
                    "",
                    f"Protocol: `{PROTOCOL_ID}`",
                    "",
                    "Isolated from Gate 4 (`historical_abc_research/`) and legacy Arm C diagnostics.",
                    "Do not write prospective journals into Gate 4 or overwrite diagnostic audits.",
                    "Counting starts only when `prospective_v1_authorization.json` has `authorized: true`.",
                    "",
                    "Controls: Arm A frozen · holdout 41–68 sealed · live order authority 0.0% · no threshold tuning.",
                    "",
                ]
            ),
            encoding="utf-8",
        )
    marker = PROSPECTIVE_NS / ".namespace_isolation.json"

    def _rel(p: Path) -> str:
        try:
            return str(p.relative_to(WORKSPACE))
        except ValueError:
            return str(p)

    marker.write_text(
        json.dumps(
            {
                "protocol_id": PROTOCOL_ID,
                "namespace": _rel(PROSPECTIVE_NS),
                "forbidden_write_targets": [_rel(p) for p in PROTECTED_PATHS],
                "nse_crypto_pooling": "FORBIDDEN",
                "created_utc": datetime.now(timezone.utc).isoformat(),
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    return PROSPECTIVE_NS


def assert_write_allowed(target: Path) -> None:
    """Refuse writes that would overwrite Gate 4 / legacy diagnostics."""
    resolved = target.resolve()
    for protected in PROTECTED_PATHS:
        pref = protected.resolve()
        if pref.exists() and pref.is_dir():
            if resolved == pref or pref in resolved.parents:
                raise PermissionError(f"refusing write into protected namespace: {target}")
        else:
            # File path (may not exist yet) — block exact target
            if resolved == pref:
                raise PermissionError(f"refusing write to protected Gate4/diagnostic artifact: {target}")
    ns = PROSPECTIVE_NS.resolve()
    allowed_sidecar = {
        AUTH_PATH.resolve(),
        (ARTIFACT_ROOT / "prospective_v1_preflight.json").resolve(),
        (ARTIFACT_ROOT / "prospective_v1_clock_state.json").resolve(),
    }
    if resolved == ns or ns in resolved.parents or resolved in allowed_sidecar:
        return
    raise PermissionError(
        f"prospective outputs must use {PROSPECTIVE_NS} or approved sidecars; got {target}"
    )


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return h.hexdigest()


def artifact_hashes() -> Dict[str, str]:
    paths = [
        WORKSPACE / "scripts" / "breakout_lifecycle.py",
        WORKSPACE / "scripts" / "rp608_prospective_v1.py",
        WORKSPACE / "scripts" / "run_rp608_regime_breakout_arms_abc_historical.py",
        WORKSPACE / "tests" / "test_rp608_breakout_lifecycle_fixtures.py",
        WORKSPACE / "tests" / "test_rp608_arm_a_invariance.py",
    ]
    out: Dict[str, str] = {}
    for p in paths:
        if p.exists():
            out[str(p.relative_to(WORKSPACE))] = sha256_file(p)
    return out


def clock_state_path() -> Path:
    return ARTIFACT_ROOT / "prospective_v1_clock_state.json"


def load_clocks() -> Dict[str, MarketClock]:
    path = clock_state_path()
    if not path.exists():
        return {Market.NSE.value: MarketClock(market=Market.NSE), Market.CRYPTO.value: MarketClock(market=Market.CRYPTO)}
    raw = json.loads(path.read_text(encoding="utf-8"))
    clocks: Dict[str, MarketClock] = {}
    for key, mkt in ((Market.NSE.value, Market.NSE), (Market.CRYPTO.value, Market.CRYPTO)):
        d = raw.get(key) or {}
        clocks[key] = MarketClock(
            market=mkt,
            completed_units=int(d.get("completed_units") or 0),
            extensions_used=int(d.get("extensions_used") or 0),
            integrity_review_done=bool(d.get("integrity_review_done")),
            tuning_forbidden=bool(d.get("tuning_forbidden", True)),
            unit_ids=list(d.get("unit_ids") or []),
        )
    return clocks


def save_clocks(clocks: Dict[str, MarketClock]) -> None:
    path = clock_state_path()
    assert_write_allowed(path)
    payload = {
        "protocol_id": PROTOCOL_ID,
        "updated_utc": datetime.now(timezone.utc).isoformat(),
        "authorized": is_authorized(),
        Market.NSE.value: asdict(clocks[Market.NSE.value]),
        Market.CRYPTO.value: asdict(clocks[Market.CRYPTO.value]),
        "note": "NSE and CRYPTO clocks are independent; never pool results.",
    }
    # dataclasses with Enum need market as value
    for key in (Market.NSE.value, Market.CRYPTO.value):
        payload[key]["market"] = clocks[key].market.value
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def separate_market_eval_guard(nse_metrics: Dict[str, Any], crypto_metrics: Dict[str, Any]) -> None:
    """Refuse any caller that tries to merge markets into one pass/fail."""
    if nse_metrics is crypto_metrics:
        raise ValueError("NSE and crypto metrics must be distinct objects")
    if "pooled" in nse_metrics or "pooled" in crypto_metrics:
        raise ValueError("pooled metrics forbidden")


def integrity_review_actions_allowed() -> Sequence[str]:
    return (
        "verify_completed_bar_processing",
        "verify_regime_classifications",
        "verify_lifecycle_transitions",
        "verify_quote_freshness",
        "verify_deduplication",
        "verify_persistence",
        "verify_arm_a_invariance",
        "write_integrity_report",
    )


def integrity_review_actions_forbidden() -> Sequence[str]:
    return (
        "change_delta_buffer",
        "change_spread_cap",
        "change_net_rr_threshold",
        "change_freshness_ms",
        "enable_dynamic_exits",
        "retune_on_interim_pnl",
        "pool_nse_crypto",
        "unseal_holdout",
        "raise_live_authority",
    )


# ---------------------------------------------------------------------------
# Start-rule v1.0 — market commencement (does not alter strategy / Gate 4)
# ---------------------------------------------------------------------------


def start_events_dir() -> Path:
    d = PROSPECTIVE_NS / "start_events"
    d.mkdir(parents=True, exist_ok=True)
    return d


def start_event_path(market: Market) -> Path:
    return start_events_dir() / f"{market.value}.json"


def blocked_event_path(market: Market) -> Path:
    return start_events_dir() / f"{market.value}_blocked.jsonl"


def load_start_event(market: Market) -> Optional[Dict[str, Any]]:
    path = start_event_path(market)
    if not path.exists():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def commencement_state(market: Market) -> CommencementState:
    ev = load_start_event(market)
    if ev and ev.get("state") == CommencementState.STARTED.value:
        return CommencementState.STARTED
    return CommencementState.NOT_STARTED


def is_market_started(market: Market) -> bool:
    return commencement_state(market) == CommencementState.STARTED


def _atomic_write_json(path: Path, payload: Dict[str, Any]) -> None:
    assert_write_allowed(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    data = json.dumps(payload, indent=2) + "\n"
    fd, tmp_name = tempfile.mkstemp(prefix=".tmp_start_", dir=str(path.parent))
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as fh:
            fh.write(data)
            fh.flush()
            os.fsync(fh.fileno())
        os.replace(tmp_name, path)
    finally:
        if os.path.exists(tmp_name):
            try:
                os.unlink(tmp_name)
            except OSError:
                pass


def _append_blocked(market: Market, record: Dict[str, Any]) -> None:
    path = blocked_event_path(market)
    assert_write_allowed(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(record) + "\n")


def is_nse_trading_day(d: date, holidays: Optional[Set[date]] = None) -> bool:
    if d.weekday() >= 5:
        return False
    if holidays and d in holidays:
        return False
    return True


def next_nse_trading_day(d: date, holidays: Optional[Set[date]] = None) -> date:
    cur = d
    for _ in range(14):
        if is_nse_trading_day(cur, holidays):
            return cur
        cur += timedelta(days=1)
    raise RuntimeError("no NSE trading day found within 14 days")


def nse_session_open_utc(session_day: date) -> datetime:
    return datetime.combine(session_day, NSE_OPEN, tzinfo=IST).astimezone(timezone.utc)


def nse_session_close_utc(session_day: date) -> datetime:
    return datetime.combine(session_day, NSE_CLOSE, tzinfo=IST).astimezone(timezone.utc)


def nse_in_regular_session(now: datetime, holidays: Optional[Set[date]] = None) -> bool:
    local = now.astimezone(IST)
    d = local.date()
    if not is_nse_trading_day(d, holidays):
        return False
    t = local.timetz().replace(tzinfo=None)
    return NSE_OPEN <= t < NSE_CLOSE


def compute_nse_eligible_boundary(
    decision_utc: datetime,
    holidays: Optional[Set[date]] = None,
) -> Dict[str, Any]:
    """First complete NSE session whose open is strictly after decision_utc."""
    if decision_utc.tzinfo is None:
        decision_utc = decision_utc.replace(tzinfo=timezone.utc)
    local = decision_utc.astimezone(IST)
    d = local.date()
    # Session in progress at decision → cannot qualify
    if nse_in_regular_session(decision_utc, holidays):
        candidate = next_nse_trading_day(d + timedelta(days=1), holidays)
    else:
        open_today = nse_session_open_utc(d) if is_nse_trading_day(d, holidays) else None
        if open_today is not None and decision_utc < open_today:
            candidate = d
        else:
            candidate = next_nse_trading_day(d + timedelta(days=1), holidays)
    open_utc = nse_session_open_utc(candidate)
    # Late relative to candidate open → reject that day, take next
    late = decision_utc >= open_utc
    if late:
        candidate = next_nse_trading_day(candidate + timedelta(days=1), holidays)
        open_utc = nse_session_open_utc(candidate)
        late = False
    return {
        "market": Market.NSE.value,
        "eligible_session_boundary": candidate.isoformat(),
        "eligible_session_open_utc": open_utc.isoformat(),
        "eligible_session_open_ist": open_utc.astimezone(IST).isoformat(),
        "late_for_boundary": late,
        "observation_window_defined_by": "eligible_session_boundary",
    }


def compute_crypto_eligible_boundary(decision_utc: datetime) -> Dict[str, Any]:
    """First UTC day whose 00:00 UTC is strictly after decision_utc."""
    if decision_utc.tzinfo is None:
        decision_utc = decision_utc.replace(tzinfo=timezone.utc)
    day = decision_utc.astimezone(timezone.utc).date()
    midnight = datetime.combine(day, time(0, 0), tzinfo=timezone.utc)
    if decision_utc >= midnight:
        # Current UTC day already begun (or exactly at midnight → not strictly after)
        candidate = day + timedelta(days=1)
    else:
        candidate = day
    open_utc = datetime.combine(candidate, time(0, 0), tzinfo=timezone.utc)
    if decision_utc >= open_utc:
        candidate = candidate + timedelta(days=1)
        open_utc = datetime.combine(candidate, time(0, 0), tzinfo=timezone.utc)
    return {
        "market": Market.CRYPTO.value,
        "eligible_session_boundary": candidate.isoformat(),
        "eligible_session_open_utc": open_utc.isoformat(),
        "late_for_boundary": False,
        "observation_window_defined_by": "eligible_session_boundary",
    }


@dataclass
class PreStartGateResult:
    ok: bool
    checks: Dict[str, Any]
    failures: List[str] = field(default_factory=list)

    def as_dict(self) -> Dict[str, Any]:
        return {
            "ok": self.ok,
            "result": "PASS" if self.ok else "FAIL",
            "checks": self.checks,
            "failures": self.failures,
        }


def run_prestart_gates(
    *,
    market: Market,
    expected_deployed_rev: Optional[str] = None,
    require_preflight_pass: bool = True,
) -> PreStartGateResult:
    """Mandatory pre-start checks. Failures/unknowns block commencement."""
    checks: Dict[str, Any] = {}
    failures: List[str] = []

    auth = load_authorization()
    checks["authorized"] = auth.get("authorized") is True
    if not checks["authorized"]:
        failures.append("not_authorized")

    controls = auth.get("controls") or {}
    checks["arm_a_frozen"] = controls.get("arm_a") == "FROZEN"
    checks["holdout_sealed"] = controls.get("holdout_sessions_41_68") == "SEALED"
    checks["live_order_authority_0"] = float(controls.get("live_order_authority_pct", -1)) == 0.0
    if not checks["arm_a_frozen"]:
        failures.append("arm_a_not_frozen")
    if not checks["holdout_sealed"]:
        failures.append("holdout_not_sealed")
    if not checks["live_order_authority_0"]:
        failures.append("live_authority_nonzero")

    deployed = read_deployed_rev()
    checks["deployed_rev"] = deployed
    auth_rev = auth.get("deployed_revision") or auth.get("git_rev") or ""
    checks["authorized_rev_match"] = bool(deployed) and deployed == auth_rev
    if expected_deployed_rev:
        exp = expected_deployed_rev.strip().rstrip("…")
        checks["expected_rev_match"] = bool(deployed) and (
            deployed == exp or deployed.startswith(exp)
        )
        if not checks["expected_rev_match"]:
            failures.append("expected_rev_mismatch")
    if not checks["authorized_rev_match"]:
        failures.append("authorized_rev_mismatch")

    preflight = load_preflight()
    if require_preflight_pass:
        checks["preflight_pass"] = preflight.get("preflight_pass") is True
        if not checks["preflight_pass"]:
            failures.append("preflight_not_pass")
        life = (preflight.get("checks") or {}).get("lifecycle_fixtures") or {}
        arm = (preflight.get("checks") or {}).get("arm_a_invariance") or {}
        checks["lifecycle_fixtures_passed"] = life.get("passed") is True
        checks["arm_a_invariance_passed"] = arm.get("passed") is True
        if life.get("passed") is not True:
            failures.append("lifecycle_fixtures_unknown_or_failed")
        if arm.get("passed") is not True:
            failures.append("arm_a_invariance_unknown_or_failed")

    # Namespace isolation probe
    try:
        assert_write_allowed(GATE4_NS / "historical_abc_summary.json")
        checks["gate4_write_refused"] = False
        failures.append("gate4_write_not_refused")
    except PermissionError:
        checks["gate4_write_refused"] = True
    try:
        assert_write_allowed(PROSPECTIVE_NS / "start_events" / f"{market.value}.json")
        checks["prospective_write_allowed"] = True
    except PermissionError:
        checks["prospective_write_allowed"] = False
        failures.append("prospective_namespace_not_writable")

    checks["market"] = market.value
    checks["start_rule_version"] = START_RULE_VERSION
    return PreStartGateResult(ok=not failures, checks=checks, failures=failures)


def evaluate_start_eligibility(
    market: Market,
    *,
    decision_utc: datetime,
    holidays: Optional[Set[date]] = None,
) -> Dict[str, Any]:
    if market == Market.NSE:
        return compute_nse_eligible_boundary(decision_utc, holidays=holidays)
    if market == Market.CRYPTO:
        return compute_crypto_eligible_boundary(decision_utc)
    raise ValueError(f"unknown market {market}")


def commence_market(
    market: Market,
    *,
    operator: str,
    confirm: bool,
    decision_utc: Optional[datetime] = None,
    expected_deployed_rev: Optional[str] = None,
    holidays: Optional[Set[date]] = None,
    require_preflight_pass: bool = True,
) -> Dict[str, Any]:
    """
    Explicit operator start. Authorization / readiness alone cannot call this effectively
    without confirm + passing gates. Idempotent once STARTED.
    """
    if not confirm:
        return {"accepted": False, "reason": "confirm_required", "state": CommencementState.NOT_STARTED.value}
    if not (operator or "").strip():
        return {"accepted": False, "reason": "operator_required", "state": CommencementState.NOT_STARTED.value}

    ensure_namespace()
    existing = load_start_event(market)
    if existing and existing.get("state") == CommencementState.STARTED.value:
        return {
            "accepted": True,
            "idempotent": True,
            "reason": "already_started",
            "state": CommencementState.STARTED.value,
            "start_event": existing,
        }

    now = decision_utc or datetime.now(timezone.utc)
    if now.tzinfo is None:
        now = now.replace(tzinfo=timezone.utc)

    gates = run_prestart_gates(
        market=market,
        expected_deployed_rev=expected_deployed_rev,
        require_preflight_pass=require_preflight_pass,
    )
    if not gates.ok:
        blocked = {
            "protocol": PROTOCOL_ID,
            "start_rule_version": START_RULE_VERSION,
            "market": market.value,
            "state": CommencementState.BLOCKED.value,
            "start_decision_utc": now.isoformat(),
            "operator": operator.strip(),
            "prestart_gate_result": gates.as_dict(),
            "reason_codes": gates.failures,
            "live_order_authority": 0.0,
        }
        _append_blocked(market, blocked)
        return {
            "accepted": False,
            "reason": "prestart_gate_failed",
            "state": CommencementState.BLOCKED.value,
            "blocked": blocked,
        }

    eligibility = evaluate_start_eligibility(market, decision_utc=now, holidays=holidays)
    open_utc = datetime.fromisoformat(eligibility["eligible_session_open_utc"])
    if now >= open_utc:
        blocked = {
            "protocol": PROTOCOL_ID,
            "start_rule_version": START_RULE_VERSION,
            "market": market.value,
            "state": CommencementState.BLOCKED.value,
            "start_decision_utc": now.isoformat(),
            "operator": operator.strip(),
            "eligible_session_boundary": eligibility["eligible_session_boundary"],
            "reason_codes": ["late_invocation_mid_or_past_boundary"],
            "prestart_gate_result": gates.as_dict(),
            "live_order_authority": 0.0,
        }
        _append_blocked(market, blocked)
        return {
            "accepted": False,
            "reason": "late_invocation",
            "state": CommencementState.BLOCKED.value,
            "blocked": blocked,
            "eligibility": eligibility,
        }

    start_event = {
        "protocol": PROTOCOL_ID,
        "protocol_doc": "rp608_regime_breakout_001_prospective_observation",
        "start_rule_version": START_RULE_VERSION,
        "market": market.value,
        "state": CommencementState.STARTED.value,
        "eligible_session_boundary": eligibility["eligible_session_boundary"],
        "eligible_session_open_utc": eligibility["eligible_session_open_utc"],
        "start_event_utc": now.isoformat(),
        "start_event_ist": now.astimezone(IST).isoformat(),
        "start_decision_utc": now.isoformat(),
        "observation_window_defined_by": "eligible_session_boundary",
        "operator": operator.strip(),
        "deployed_revision": read_deployed_rev(),
        "prestart_gate_result": "PASS",
        "prestart_gate_evidence": gates.as_dict(),
        "live_order_authority": 0.0,
        "controls_attested": {
            "arm_a": "FROZEN",
            "holdout_sessions_41_68": "SEALED",
            "integrity_1_5_tuning": "FORBIDDEN",
            "nse_crypto_pooling": "FORBIDDEN",
        },
    }
    _atomic_write_json(start_event_path(market), start_event)

    # Update auth sidecar flags without rewriting provenance fields wholesale
    auth = load_authorization()
    markets_started = dict(auth.get("markets_started") or {})
    markets_started[market.value] = True
    auth["markets_started"] = markets_started
    if market == Market.NSE:
        auth["session_1_started"] = False  # Session 1 completes after boundary session ends
        auth["nse_commencement"] = CommencementState.STARTED.value
        auth["nse_eligible_session_boundary"] = start_event["eligible_session_boundary"]
    if market == Market.CRYPTO:
        auth["crypto_commencement"] = CommencementState.STARTED.value
        auth["crypto_eligible_session_boundary"] = start_event["eligible_session_boundary"]
    # Global observation_counting flips to STARTED only when at least one market commenced
    auth["observation_counting"] = "STARTED"
    auth["clock_start"] = {
        "rule": START_RULE_VERSION,
        "note": "Per-market start events define boundaries; no retroactive counting",
        "markets_started": markets_started,
    }
    assert_write_allowed(AUTH_PATH)
    AUTH_PATH.write_text(json.dumps(auth, indent=2) + "\n", encoding="utf-8")

    clocks = load_clocks()
    save_clocks(clocks)

    return {
        "accepted": True,
        "idempotent": False,
        "state": CommencementState.STARTED.value,
        "start_event": start_event,
        "eligibility": eligibility,
    }
