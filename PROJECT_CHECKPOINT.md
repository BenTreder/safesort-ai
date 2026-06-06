# SafeSort AI — Project Checkpoint

**Date**: 2026-06-05 (Phase 5 hardened — v0.7.0 Downloads Apply-Safety Filtering)
**Version**: 0.7.0
**Phase**: Phase 5 hardened — downloads apply-safety filtering + classification improvements

## Safety Audit Summary (2026-06-05)

- **300 tests passing** (62 lib + 50 bin + 23 placement + 165 safety)
- **apply is GUARDED, not disabled** — real moves enabled with all safety flags
- **Safe Autopilot** — plan eligibility only; does not move files by itself
- **Guided Review** — question/review workflow only; does not move files
- **No destructive filesystem calls** outside `src/apply/engine.rs` (verified by grep)
- **Demo fixture path**: `./safesort_demo/` (gitignored)
- **Manual verification**: 15 demo files moved, 15 files restored, no real user files touched

## Phase 5 Hardened: Downloads Apply-Safety Filtering (v0.7.0)

| Component | Status |
|---|---|
| `src/placement/file_purpose.rs` — BookCover, BookKindle, BookPrint, SensitiveDocument variants | ✅ |
| CoverLetter restricted to explicit "coverletter"/"coverltr"/"applicationletter" tokens | ✅ |
| SensitiveDocument: creditReport, BOIR, CP_575, IRS_, backup_codes, recovery_codes, etc. | ✅ |
| BookCover: "cover" in name + image/pdf ext, not a cover letter | ✅ |
| BookKindle: "kindle", .epub, "mobi" → Books/{owner}/Kindle | ✅ |
| BookPrint: print_ready/printready or print+pdf+final → Books/{owner}/Print Files | ✅ |
| `src/placement/ownership.rs` — Big Win Jerky, Big Win Seasonings as Client aliases | ✅ |
| Book title aliases: Ghost Circuit, Break Build Blaze, Website Saved Main St, Noodles | ✅ |
| `src/placement/destination.rs` — routes for new FilePurpose variants | ✅ |
| Document/Report routing: Client owners → Client Work/{owner}/Documents | ✅ |
| SensitiveDocument → 99_Review Needed/Sensitive Documents | ✅ |
| `src/manifest/plan_manifest.rs` — "no destination computed" → auto_plan_eligible=false | ✅ |
| "99_Review Needed"/"Review Needed"/"/Unknown/" destinations → auto_plan_eligible=false | ✅ |
| SensitiveDocument purpose → auto_plan_eligible=false | ✅ |
| `src/placement/engine.rs` — risky parent folder (folder.ext) caps confidence to 50 | ✅ |
| `src/app.rs` — dry-run separates Would move / Would skip counts | ✅ |
| .bat, .cmd, .ps1 added to code extensions | ✅ |
| 19 new safety tests | ✅ |
| 300 tests passing | ✅ |

## Phase 5: Guarded Apply (v0.6.0)

| Component | Status |
|---|---|
| `src/apply/engine.rs` — core apply/rollback/status engine | ✅ |
| `src/apply/receipt.rs` — `ApplyReceipt`, `RollbackEntry`, `RollbackStatus` | ✅ |
| `src/apply/mod.rs` — module wiring | ✅ |
| Freeze-state backup before every move (`fs::copy`) | ✅ |
| Backup checksum verification | ✅ |
| Destination parent directory creation | ✅ |
| `fs::rename` atomic move | ✅ |
| Destination checksum verification | ✅ |
| Final destination = planned_dir + source filename | ✅ |
| Never appends filename twice if already present | ✅ |
| `--rollback-output` writes per-file receipt | ✅ |
| `safesort rollback <receipt>` — restore from backup | ✅ |
| Rollback never removes directories | ✅ |
| Rollback refuses if dest path is a directory | ✅ |
| `safesort apply-status <receipt>` — read-only | ✅ |
| `safesort apply --dry-run` — no flags required | ✅ |
| Safe zone files skip inside_project penalty | ✅ |
| Preflight runs before every real apply | ✅ |
| All 4 flags required for real apply | ✅ |
| LOCKED/REVIEW/MEDIUM/HIGH/CRITICAL never moved | ✅ |
| 277 tests passing before metadata update | ✅ |
| 281 tests passing after doctor/metadata tests | ✅ |

### Apply Safety Gates (all required)

1. Valid SafeSort manifest (`dry_run_only=true`)
2. All 8 preflight checks pass
3. `--backup` flag (freeze-state copy before every move)
4. `--apply-safe-only` flag (only `auto_plan_eligible` entries)
5. `--confirm` flag
6. `--i-understand-this-moves-files` flag
7. Backup checksum verified before move
8. Destination checksum verified after move

### What Changed in v0.6.0

- `src/apply/engine.rs` — full apply engine with freeze-state backup
- `src/apply/receipt.rs` — `final_destination_path` field added (`serde(default)`)
- `src/scan/classifier.rs` — safe zone detection fix (scan_root named Downloads/Desktop)
- `src/placement/engine.rs` — safe zone files skip inside_project penalty
- `src/app.rs` — dry-run flags, guarded apply path, doctor output update
- `Cargo.toml` — version 0.6.0
- `tests/safety_tests.rs` — 19 new apply/rollback/dest/doctor tests

## What Was Built (Cumulative)

### Phase 1: Read-Only Scanner (v0.1.0)
Complete safety-first scanner with 7 detectors, classification, and profiling.

### Phase 1+: Smart Placement Engine (v0.2.0)
Premium placement intelligence on top of safety classification.

### Phase 2: Dependency Graph + Impact Visibility (v0.3.0)
Impact levels, rule files, depth/exclude controls, parent-risk inheritance.

### Phase 3: Manifest + Preflight (v0.4.0)
SHA-256 checksum manifest, 8-check preflight engine, hardened apply infrastructure.

### Phase 4: Organize Workflow + Doctor (v0.5.0)
Premium `organize` workflow, upgraded doctor, auto-plan eligibility, demo fixture fix.

### Phase 5: Guarded Apply (v0.6.0)
Freeze-state backup, real file movement (gated), rollback, apply-status, final path resolution.

## What Is Intentionally Disabled for Safety

| Feature | Reason |
|---|---|
| LOCKED/REVIEW/MEDIUM/HIGH/CRITICAL moves | Always disabled — no flag can override |
| Overwriting existing destination files | Always disabled |
| Directory removal | Rollback removes only the exact moved file, never dirs |
| Direct live-site moves | Always disabled — staging only |
| File deletion (beyond rollback cleanup) | Always disabled |
| chmod/chown | Never implemented |
| systemd/cron/shell config edits | Never implemented |
| Rules persistence to disk | Phase 4 planned |
| AI summary integration | Phase 6 |
| Tauri desktop GUI | Phase 7 |

## Test Results (v0.7.0)

```
test result: ok. 62 passed   (lib unit tests)
test result: ok. 50 passed   (binary integration tests)
test result: ok. 23 passed   (placement tests)
test result: ok. 165 passed  (safety integration tests)
──────────────────────────────
Total: 300 tests, 0 failed
```
