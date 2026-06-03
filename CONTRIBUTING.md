# Contributing

## Infrastructure Philosophy

ChronoSentiment is developed as **deterministic financial infrastructure** rather than a feature‑velocity trading application.

Contributors must preserve:

- deterministic replay equivalence
- constitutional authority isolation
- orchestration ordering guarantees
- certification reproducibility
- semantic stability across migrations

Architectural integrity takes precedence over implementation velocity.

---

### How to Contribute

1. Fork the repository and create a feature branch.
2. Ensure all new code passes the constitutional CI workflow (`./scripts/ci_full.sh`).
3. Run the replay certification test before submitting a PR:
   ```bash
   cargo test replay --release -- --test-threads=1
   ```
4. Verify that no large experimental artefacts are added (respect the `.gitignore` patterns).
5. Submit a pull request targeting the `main` branch.

### Review Process

- All PRs are reviewed by the repository owner (`@nikhilmore54`).
- Changes that affect deterministic replay, orchestration ordering, or certification must be accompanied by updated tests and documentation.
- Any modification to files under `docs/constitution/`, `docs/certification/`, or `docs/migrations/` requires explicit justification in the PR description.

---

### Development Guidelines

- Use Rust 1.78 or later.
- Follow the project's coding style (run `cargo fmt`).
- Keep dependencies minimal; the CI includes a forbidden‑dependency scan.
- Do not commit generated files outside of the designated `fixtures/generated/` or `fixtures/runtime_dumps/` directories.
