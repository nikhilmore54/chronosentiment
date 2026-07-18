#!/usr/bin/env python3
"""
M6.1 — Architectural Invariant Gate for coralys-scheduling.

Enforces the following invariants:

  INV-1  coralys-scheduling must not depend on any other coralys-* crate.
         Allowed external deps: serde, chrono, uuid, thiserror (and their
         transitive std/proc-macro companions).

  INV-2  The optimization and resilience layers must not import legality
         *implementation* types directly.  The correct pattern is dependency
         injection: receive a &LegalityChecker parameter rather than
         constructing or importing legality rules internally.

         Allowed in optimization/ and resilience/ files:
           - use crate::legality::LegalityChecker          (oracle interface)
           - use crate::legality::test_helpers::*           (test code only)

         Forbidden (indicates embedded legality logic):
           - use crate::legality::LegalityRule
           - use crate::legality::LegalityEngine
           - use crate::legality::RuleSet
           - use crate::legality::rules::*  (any rules sub-module)
           - mod legality;  (re-declaring the module)

  INV-3  No coralys-scheduling source file may use std::process::exit,
         std::fs, or std::env directly (platform-independence invariant).

Exit codes:
  0  All invariants pass.
  1  One or more invariants violated — details printed to stdout.
"""

import sys
import re
import pathlib
import tomllib  # stdlib since Python 3.11; falls back to tomli

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
SCHEDULING_TOML = REPO_ROOT / "coralys-scheduling" / "Cargo.toml"
SCHEDULING_SRC  = REPO_ROOT / "coralys-scheduling" / "src"

# Crates that coralys-scheduling is allowed to depend on.
ALLOWED_DEPS = {
    "serde",
    "chrono",
    "uuid",
    "thiserror",
    # dev-dependencies
    "serde_json",
}

# Patterns forbidden in optimization/ and resilience/ source files.
#
# The rule: these layers may reference LegalityChecker (the oracle interface)
# and test_helpers, but must not import legality *implementation* types
# (rules, engine, rule sets) — that would embed legality logic in the layer.
#
# Each tuple is (pattern, description).
FORBIDDEN_IMPORT_PATTERNS = [
    (re.compile(r"use\s+crate::legality::LegalityRule"),
     "imports LegalityRule — embed legality logic via DI instead"),
    (re.compile(r"use\s+crate::legality::LegalityEngine"),
     "imports LegalityEngine — use LegalityChecker oracle instead"),
    (re.compile(r"use\s+crate::legality::RuleSet"),
     "imports RuleSet — use LegalityChecker oracle instead"),
    (re.compile(r"use\s+crate::legality::rules"),
     "imports legality rules sub-module — forbidden in optimization/resilience"),
    (re.compile(r"^\s*mod\s+legality\s*;", re.MULTILINE),
     "re-declares legality module — forbidden outside legality layer"),
]

# Patterns forbidden in ALL coralys-scheduling source files.
FORBIDDEN_PLATFORM_PATTERNS = [
    re.compile(r"std::process::exit"),
    re.compile(r"std::fs::"),
    re.compile(r"std::env::"),
]

violations: list[str] = []


# ---------------------------------------------------------------------------
# INV-1: Cargo.toml dependency check
# ---------------------------------------------------------------------------
def check_inv1() -> None:
    try:
        with open(SCHEDULING_TOML, "rb") as f:
            data = tomllib.load(f)
    except Exception as e:
        violations.append(f"INV-1: Could not parse {SCHEDULING_TOML}: {e}")
        return

    for section in ("dependencies", "dev-dependencies", "build-dependencies"):
        deps = data.get(section, {})
        for dep_name in deps:
            if dep_name not in ALLOWED_DEPS:
                violations.append(
                    f"INV-1: coralys-scheduling/{section} contains forbidden "
                    f"dependency '{dep_name}'. "
                    f"Allowed: {sorted(ALLOWED_DEPS)}"
                )


# ---------------------------------------------------------------------------
# INV-2: Layer boundary import check
# ---------------------------------------------------------------------------
def check_inv2() -> None:
    restricted_dirs = [
        SCHEDULING_SRC / "optimization",
        SCHEDULING_SRC / "resilience",
    ]
    for directory in restricted_dirs:
        if not directory.exists():
            continue
        for rs_file in directory.rglob("*.rs"):
            text = rs_file.read_text(encoding="utf-8")
            for pattern, description in FORBIDDEN_IMPORT_PATTERNS:
                if pattern.search(text):
                    rel = rs_file.relative_to(REPO_ROOT)
                    violations.append(
                        f"INV-2: {rel} {description}."
                    )


# ---------------------------------------------------------------------------
# INV-3: Platform-independence check
# ---------------------------------------------------------------------------
def check_inv3() -> None:
    if not SCHEDULING_SRC.exists():
        violations.append(f"INV-3: Source directory not found: {SCHEDULING_SRC}")
        return
    for rs_file in SCHEDULING_SRC.rglob("*.rs"):
        text = rs_file.read_text(encoding="utf-8")
        for pattern in FORBIDDEN_PLATFORM_PATTERNS:
            if pattern.search(text):
                rel = rs_file.relative_to(REPO_ROOT)
                violations.append(
                    f"INV-3: {rel} uses forbidden platform API "
                    f"(pattern: {pattern.pattern!r})."
                )


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main() -> int:
    print("M6.1 Architectural Invariant Gate — coralys-scheduling")
    print("=" * 60)

    check_inv1()
    check_inv2()
    check_inv3()

    if violations:
        print(f"\nFAILED — {len(violations)} violation(s):\n")
        for i, v in enumerate(violations, 1):
            print(f"  [{i}] {v}")
        print()
        return 1

    print("PASSED — all architectural invariants satisfied.")
    print(f"  INV-1: coralys-scheduling dependency boundary clean")
    print(f"  INV-2: optimization/resilience layers do not import legality directly")
    print(f"  INV-3: no forbidden platform APIs in scheduling source")
    return 0


if __name__ == "__main__":
    sys.exit(main())