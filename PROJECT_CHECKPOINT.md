# SafeSort AI — Project Checkpoint

**Date**: 2026-06-05 (Phase 2 stabilization — parent-risk inheritance)
**Version**: 0.2.3
**Phase**: 2 in progress — impact now wired into scan and plan output

## Safety Audit Summary (2026-06-05)

- **153 tests passing** (51 lib + 39 bin + 23 placement + 40 safety)
- **apply still disabled** — prints "Nothing was moved." unconditionally
- **Safe Autopilot still plan-only** — no moves, no file operations
- **Guided Review still plan-only** — question queue only, no moves
- **No destructive filesystem calls** anywhere in src/ (verified by grep)
- **Demo fixture path**: `./safesort_demo/` (gitignored)
- **Workspace Overlay**: preferred for active projects

### Phase 2 stabilization: impact visibility (landed 2026-06-05)

Impact level is now derived from evidence and surfaced throughout the tool:

| Component | Status |
|---|---|
| `impact_from_evidence()` in `reports/mod.rs` | ✅ |
| `ItemResult.impact_level` field on every scan item | ✅ |
| `SafetySummary` impact counts (Critical/High/Medium/Low/None) | ✅ |
| Terminal scan output — impact summary block | ✅ |
| Terminal scan output — impact inline per example | ✅ |
| `PlacementRecommendation.impact_level` field | ✅ |
| Plan output — impact icon + level per recommendation | ✅ |
| Safe Autopilot explicit impact gate (MEDIUM+ excluded) | ✅ |
| Fake-systemd `scan_dir` for explain command | ✅ |
| 7 new impact-focused tests | ✅ |
| `.gitignore` ignores `target/` and `safesort_demo/` | ✅ |
| **Parent-risk inheritance (v0.2.3)** | |
| `EvidenceKind::InheritedRisk` | ✅ |
| `LIVE_SITE_FOLDER_NAMES` in config | ✅ |
| `detect_sensitive_in_dir` — dirs containing .env → LOCKED | ✅ |
| Second-pass inheritance in `Scanner::scan` | ✅ |
| `public_html/index.php` → REVIEW/HIGH (was SAFE) | ✅ |
| `ImportantApp/config.yml` → REVIEW/HIGH (was SAFE) | ✅ |
| 6 new inheritance tests (total 153 passing) | ✅ |

**Impact is display-only. The tool remains read-only. Nothing is moved.**

### Phase 2 dependency graph (foundation)

The `src/graph/` module provides:

| Component | Status |
|---|---|
| `DependencyGraph` node/edge model | ✅ |
| `ImpactLevel` enum (None/Low/Medium/High/Critical) | ✅ |
| `analyze_impact_from_evidence` | ✅ |
| `analyze_project_impact` (.git/Cargo.toml → Medium+) | ✅ |
| `analyze_sensitive_folder_impact` (.env → Critical) | ✅ |
| `SystemdDetector::scan_dir` for fake-systemd fixtures | ✅ |
| Service-bound impact in `explain` command | ✅ |
| Cross-reference to apply pipeline | ⬜ Phase 5 |

**The dependency graph explains what would break — it does not move anything.**

## What Was Built

### Phase 1: Read-Only Scanner (v0.1.0)
Complete safety-first scanner with 7 detectors, classification, and profiling.

### Phase 1+: Smart Placement Engine (v0.2.0)
Premium placement intelligence on top of safety classification.

#### New Commands

| Command | Description |
|---|---|
| `safesort plan --path <P>` | Smart placement plan (preview mode) |
| `safesort plan --path <P> --mode guided` | Interactive guided review |
| `safesort plan --path <P> --mode safe-autopilot` | Auto-plan ≥95% confidence |
| `safesort scan --path <P> --mode locked-down` | Extra conservative scan |

#### New Modules (8 files)

```
src/placement/
  mod.rs            — Module root + re-exports
  engine.rs         — SmartPlacementEngine (orchestrator + unit tests)
  ownership.rs      — OwnershipDetector (brand/project + unit tests)
  file_purpose.rs   — FilePurposeDetector (logo, banner, etc. + unit tests)
  destination.rs    — DestinationPlanner (safe staging + unit tests)
  confidence.rs     — ConfidenceScorer (0–100 + unit tests)
  rules.rs          — RulesEngine (user-defined rules + unit tests)
  question_queue.rs — QuestionQueue (guided review rendering)
```

#### Test Results

```
test result: ok. 121 total tests passing
  39 unit tests (lib.rs)
  39 unit tests (main.rs binary)
  23 placement integration tests
  20 safety integration tests
  0 doc-tests
```

### Files Changed (14 files modified/created)

**New:**
- `src/placement/{mod,engine,ownership,file_purpose,destination,confidence,rules,question_queue}.rs`
- `tests/placement_tests.rs`

**Modified:**
- `src/cli.rs` — Added `--mode` flag, `plan` command, `OrgMode` enum
- `src/app.rs` — Added `cmd_plan`, `render_placement_plan`, mode organization
- `src/main.rs` — Added `mod placement`
- `src/lib.rs` — Added `pub mod placement`
- `README.md` — Smart Placement Engine documentation
- `SAFETY.md` — Safe staging, confidence gating, live-site safety
- `ROADMAP.md` — Phase 1+ complete, Phase 2 planned
- `PROJECT_CHECKPOINT.md` — This file

### Example Output

```
$ safesort plan --path safesort_demo/Downloads --mode guided

  SafeSort AI — Smart Placement Plan
  Target: safesort_demo/Downloads
  Mode: guided

  Placement Summary:
    Total files scanned:    20
    🔒 Locked:              0
    🟡 Guided review:       0
    ⚠️  Review needed:       15
    ⬜ Leave alone:          5

  ┌─────────────────────────────────────────────
  │ File:       .../Downloads/bentreder_logo.png
  │ Owner:      Ben Treder Digital (BenTreder.com)
  │ Purpose:    Logo
  │ Type:       Image
  │ Risk:       GREEN
  │ Confidence: 94%
  │ Dest:       Brand Assets → BenTreder → Logos
  │ Path:       ~/Workspace/06_Business/Brand Assets/BenTreder/Logos
  │ Action:     GUIDED REVIEW
  └─────────────────────────────────────────────

  Nothing was moved.
```

### Commands to Try

```bash
# Build
cargo build --release

# Generate demo fixture with smart placement files
./target/release/safesort demo-fixture

# Smart placement plan (guided mode)
./target/release/safesort plan --path safesort_demo/Downloads --mode guided

# Safe autopilot mode
./target/release/safesort plan --path safesort_demo/Downloads --mode safe-autopilot

# Locked-down scan
./target/release/safesort scan --path safesort_demo --mode locked-down

# Export plan as JSON
./target/release/safesort plan --path safesort_demo/Downloads --output plan.json

# Run all tests
cargo test
```

## What Is Implemented

| Feature | Status |
|---|---|
| Read-only safety scanner | ✅ Complete |
| 7 safety detectors | ✅ Complete |
| Smart Placement Engine | ✅ Complete |
| Ownership detection (15+ aliases) | ✅ Complete |
| Purpose detection (25+ purposes) | ✅ Complete |
| Confidence scoring (0–100) | ✅ Complete |
| Safe staging destinations | ✅ Complete |
| 4 organization modes | ✅ Complete |
| Guided review question queue | ✅ Complete |
| Rules engine (in-memory) | ✅ Complete |
| Workspace Overlay | ✅ Complete |
| Downloads Triage | ✅ Complete |
| 133 passing tests | ✅ Complete |

## What Is Intentionally Disabled for Safety

| Feature | Reason |
|---|---|
| `apply` command | Phase 5 — needs rollback manifest first |
| File moving | Phase 5 — needs checksum verification first |
| File deletion | Never without explicit consent + backup |
| Direct live-site moves | Always disabled — staging only |
| Rules persistence to disk | Phase 3 — needs explicit opt-in flag |
| Rollback manifest | Phase 4 |
| Checksum verification | Phase 4 |
| AI summary integration | Phase 6 |
| Tauri desktop GUI | Phase 7 |

## Known Limitations (Phase 1+)

1. **Demo fixture inside project dir:** When running `demo-fixture` from within the Rust project, files inside the fixture inherit the parent project's `Cargo.toml`, triggering the `inside_project` penalty. In real usage (`~/Downloads`), this doesn't occur.

2. **Systemd/Cron detectors scan real paths:** The `demo-fixture` creates fake systemd units, but detectors scan real `/etc/systemd/system` etc.

3. **Rules are in-memory only:** Custom rules are lost between runs. Persistence via `~/.safesort/rules.toml` is planned for Phase 3.

4. **Alias coverage:** 15+ built-in aliases cover common brands/projects. Users can add custom aliases programmatically or via future rules file.

## Recommended Next Prompt

For **Phase 2** (dependency graph + apply infrastructure):
```
Build Phase 2 of SafeSort AI:
- Full dependency graph of scanned paths
- Cross-reference script/Docker/nginx references
- Impact analysis: "Moving X would break Y, Z"
- --depth and --exclude flags
- --rule-file flag for custom rules TOML
- Checksum and rollback manifest generation
- Keep apply as a stub
- Add tests for new features
```
