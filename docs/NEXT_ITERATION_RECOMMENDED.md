# Next Iteration Plan (All Features = Recommended)

This plan converts the 20 discovery responses into concrete product decisions and a focused execution scope.

## 1) Product goal for this iteration

Deliver a reliable **proposal output package** that includes:

1. Project description and core information
2. Hours breakdown
3. Bill of materials
4. All equipment in scope
5. Construction drawings with AutoMate overlays

Success is defined as consistent, professional **exported PDF output** for engineers and easy distribution to administrative staff.

---

## 2) Users and experience targets

### Primary user: Engineer
- Must be able to build and validate the full estimate/proposal package.
- Must trust overlay accuracy and object mapping.

### Secondary user: Administrative staff
- Must be able to open, review, and email final PDFs with minimal app interaction.

---

## 3) Scope policy (hard boundaries)

### In scope
- Output quality and completeness of proposal package
- Template-source-of-truth behavior
- Drawings overlay usability and performance
- Export reliability and standardized data structure

### Out of scope for this iteration
- Flow diagrams
- Full CAD drafting features
- Deep engineering-level CAD production workflow

> Product position remains: **estimation and proposal tool**, not a full engineering CAD suite.

---

## 4) Recommended decision answers to unresolved risk questions

### Q9: Technical debt to accept now
Accept temporarily:
- Single-threaded bottlenecks during PDF import where architecture change is non-trivial.
- Limited automated tests around non-critical UI styling.

Do not accept:
- Unbounded blocking behavior when loading drawing PDFs.
- Template-to-output inconsistencies.
- Non-standardized export data structures.

### Q11: Automated testing gap
Recommended minimum bar this iteration:
- Add/expand tests around export section assembly and schema field presence.
- Add regression checks for template field propagation to outputs.

### Q12: Data standardization
Define one canonical project schema version (e.g., `schema_version: 1`) and ensure all export surfaces reference it.

### Q19: Incident response (internal release)
- If export fails: provide visible error message + fallback to last known good autosave bundle.
- If overlay load stalls: allow cancel/retry and preserve unsaved work.

---

## 5) Feature recommendations (set to "Recommended")

| Feature Area | Recommended | Why | Priority |
|---|---|---|---|
| Proposal package composer (all required sections) | Yes | Core success metric for this cycle | P0 |
| Drawings overlay modernization | Yes | Biggest known risk and user pain | P0 |
| Template as source of truth | Yes | Removes output inconsistency and rework | P0 |
| Export PDF quality gate/checklist | Yes | Directly tied to release acceptance | P0 |
| PDF load progress indicator + non-blocking UX | Yes | Addresses app stall perception and usability | P0 |
| Export data structure standardization | Yes | Required for interoperability | P1 |
| Admin handoff mode (simple export/send workflow) | Yes | Supports secondary user outcomes | P1 |
| Save/load resiliency validation | Yes | Foundation for trust in workflow continuity | P1 |
| Project settings polish | Yes | Already decent; finish pass only | P2 |
| Software settings/about polish | Yes | Already complete; maintenance only | P3 |
| Full CAD-level detailing features | No (deferred) | Explicitly out of scope to avoid creep | Deferred |
| Flow-diagram subsystem | No (deferred) | Explicitly out of scope to avoid creep | Deferred |

---

## 6) Functional acceptance criteria

The iteration is accepted when all are true:

1. Exported proposal PDF always contains all five required sections.
2. Drawings overlays appear in export with expected placement and line routing.
3. Template-driven defaults are reflected in generated outputs unless user-overridden by design.
4. Loading a large drawing provides visible progress/feedback and does not appear frozen.
5. Export data fields follow one documented schema version and structure.
6. Administrative user can complete handoff using PDF-only workflow.

---

## 7) Delivery plan (three short milestones)

### Milestone A — Output reliability first
- Lock export section ordering and mandatory content checks.
- Add pre-export validation messages for missing required sections.

### Milestone B — Overlay performance and trust
- Add loading indicator/status stages for PDF ingestion.
- Ensure overlay object placement, scaling, and routing survive save/load/export.

### Milestone C — Template consistency + handoff
- Ensure template values propagate as source-of-truth defaults.
- Finalize admin-focused export/handoff workflow and release checklist.

---

## 8) Release checklist (internal)

- Proposal PDF generated for sample small, medium, and large projects.
- Overlay-heavy project opens without perceived freeze and exports correctly.
- Template change is reflected in subsequent generated outputs.
- JSON/CSV/PDF outputs show consistent field naming conventions.
- Engineer signoff completed on one realistic project package.

