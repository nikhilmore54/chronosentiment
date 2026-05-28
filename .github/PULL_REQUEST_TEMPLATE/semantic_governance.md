## Semantic Governance Hardening PR Template

**Please fill out the following checklist** before merging. This ensures that any change respects the constitutional governance boundaries.

---

1. **New constitutional term?**  
   - [ ] Yes  
   - [ ] No
   - If yes, add the term and definition to `docs/constitution/glossary.md` and link it from affected docs.
2. **Modifies authority‑critical files?** (`AUTHORITY_MAP.md`, `architecture.md`, `topology.md`, `glossary.md`)
   - [ ] Yes  
   - [ ] No
3. **Affects replay hash or certification?**
   - [ ] Yes  
   - [ ] No
4. **Adds or modifies large artifacts** (`*.jsonl`, `*.tar.gz`, `node_modules/`)
   - [ ] Yes  
   - [ ] No
5. **Introduces operational or experimental terminology**
   - [ ] Yes  
   - [ ] No

---

**Reviewer Checklist** (to be completed during review):
- Verify that any new constitutional term has a clear, canonical definition and is added to the glossary.
- Confirm that changes to authority‑critical files are justified and documented.
- Ensure replay‑related changes have been re‑certified (run the replay test locally).
- Confirm no prohibited large artifacts were introduced.
- Validate that the PR description addresses each item above.

*By merging this PR you acknowledge that the changes respect the constitutional governance model.*
