# SafeSort AI — Development Roadmap

## Phase 1: Read-Only Scanner ✅ COMPLETE

**Status**: Implemented in v0.1.0

- [x] Project structure with clean Rust modules
- [x] CLI with clap (doctor, demo-fixture, scan, profile, explain, apply)
- [x] Read-only filesystem walker
- [x] Safety classification system (LOCKED / REVIEW / SAFE_CANDIDATE)
- [x] Seven safety detectors
- [x] User profile detection with scoring
- [x] Folder structure recommendations per profile
- [x] Terminal output with premium formatting
- [x] JSON report output
- [x] Markdown report output
- [x] Comprehensive test suite (20 tests)
- [x] `apply` command stub (refuses to run)
- [x] README, SAFETY, ROADMAP, PROJECT_CHECKPOINT docs

## Phase 1+: Smart Placement Engine ✅ COMPLETE

**Status**: Implemented in v0.2.0

- [x] Smart Placement Engine architecture (`src/placement/`)
- [x] OwnershipDetector — brand/project detection from filename tokens
- [x] FilePurposeDetector — logo, banner, screenshot, report, invoice, release zip, etc.
- [x] DestinationPlanner — safe staging destinations (never live site roots)
- [x] ConfidenceScorer — 0–100 scoring with configurable bands
- [x] RulesEngine — user-defined placement rules (in-memory)
- [x] QuestionQueue — interactive guided review
- [x] 4 Organization Modes: Preview, Guided, Safe Autopilot, Locked-Down
- [x] `safesort plan` command with `--mode` flag
- [x] `--mode` flag on `safesort scan`
- [x] Workspace Overlay concept (categorize without moving)
- [x] Downloads Triage rules
- [x] Alias system with known brand/project mappings
- [x] Comprehensive test suite (23 additional tests)
- [x] All 121 tests passing (39 unit + 20 integration + 23 placement + 39 lib)

## Phase 2: Dependency Graph + Impact Visibility ⚠️ IN PROGRESS

**Status**: v0.3.0 — read-only custom rule files added (`--rule-file`); `--depth` and `--exclude` added; parent-risk inheritance complete; impact wired throughout.
`apply` remains disabled. Nothing is moved.

> **Demo fixture path**: `./safesort_demo/` (gitignored)
> **Apply**: still disabled — prints "Nothing was moved."
> **Safe Autopilot**: plan-only — MEDIUM/HIGH/CRITICAL impact items explicitly excluded.
> **Workspace Overlay**: preferred pattern for active projects (categorize without touching).
> **Dependency graph**: explains what *would* break before any future apply ever exists.

- [x] DependencyGraph struct with node/edge model (`src/graph/`)
- [x] ImpactLevel enum (None / Low / Medium / High / Critical)
- [x] `analyze_impact_from_evidence` — converts scan evidence to impact level
- [x] `analyze_project_impact` — .git / Cargo.toml / package.json → Medium+ impact
- [x] `analyze_sensitive_folder_impact` — .env → Critical impact
- [x] `SystemdDetector::scan_dir` — scan fake-systemd dirs for service bindings
- [x] `explain` command shows service-bound CRITICAL impact with service name and field
- [x] `impact_from_evidence()` in `reports/mod.rs` — evidence → impact string
- [x] `ItemResult.impact_level` on every scan item
- [x] `SafetySummary` impact counts (Critical/High/Medium/Low/None)
- [x] Scan terminal output: impact summary block + inline impact per example
- [x] `PlacementRecommendation.impact_level` + impact icon in plan output
- [x] Safe Autopilot explicit gate: MEDIUM/HIGH/CRITICAL impact → never auto-plan
- [x] `--rule-file <FILE>` flag on scan, plan, explain
- [x] `src/rules_file/` module: schema, loader, validation
- [x] Alias injection into OwnershipDetector from rule file
- [x] Protected paths from rule file → LOCKED + children inherit REVIEW
- [x] Custom staging destinations from rule file (with safety validation)
- [x] Risky custom destinations auto-rejected (system paths, live-site paths)
- [x] `PlacementRecommendation.rule_note` — shows rule-file influence in plan output
- [x] Explain command shows rule-file protected path + alias influence
- [x] Rules never bypass safety policy, never persist, never auto-load
- [x] 15 new rule-file tests (total 183 passing)
- [x] `examples/safesort-rules.toml` — fully annotated example
- [x] `--depth <N>` flag — limits traversal depth on scan and plan
- [x] `--exclude <PATTERN>` flag (repeatable) — skips items by name or path substring
- [x] `SafetySummary.skipped` count — excluded items tracked in summary
- [x] Excluded items never classified, never auto-plan eligible
- [x] 160 tests passing (47 safety integration tests)
- [x] `.gitignore` covers `target/` and `safesort_demo/`
- [ ] Cross-reference script/Docker/nginx references against scanned paths
- [ ] Detect Docker volume mounts and bind mounts
- [ ] Detect Nginx/Apache virtual host document roots

## Phase 2 Continued: Apply Infrastructure

**Status**: Planned

- [ ] Build a full dependency graph of scanned paths
- [ ] Cross-reference all script references against scanned paths
- [ ] Detect Docker volume mounts and bind mounts
- [ ] Detect Nginx/Apache virtual host document roots
- [ ] Detect Python virtualenv references
- [ ] Detect Node.js `NODE_PATH` references
- [ ] Detect shell profile references (`.bashrc`, `.zshrc`, `.profile`)
- [ ] Detect SSH config `IdentityFile` references
- [ ] Detect Git remote URLs pointing to local paths
- [ ] Impact analysis: "Moving X would break Y, Z"
- [ ] Per-file ownership overrides in scan results
- [ ] Export/import rules as TOML

## Phase 3: Plan Generation

**Status**: Planned

- [ ] Build a full dependency graph of scanned paths
- [ ] Cross-reference all script references against scanned paths
- [ ] Detect Docker volume mounts and bind mounts
- [ ] Detect Nginx/Apache virtual host document roots
- [ ] Detect Python virtualenv references
- [ ] Detect Node.js `NODE_PATH` references
- [ ] Detect shell profile references (`.bashrc`, `.zshrc`, `.profile`)
- [ ] Detect SSH config `IdentityFile` references
- [ ] Detect Git remote URLs pointing to local paths
- [ ] Impact analysis: "Moving X would break Y, Z"

## Phase 3: Plan Generation + Manifest + Preflight ✅ COMPLETE (MVP)

**Status**: Implemented in v0.4.0 — MVP complete.
`apply` still disabled. Nothing is moved.

- [x] `safesort manifest --path <PATH> [--output <FILE>]` command
- [x] `safesort plan --manifest-output <FILE>` option
- [x] SHA-256 checksums for all SAFE_CANDIDATE files (via `sha2` crate)
- [x] `ManifestEntry` with `dry_run_only: true` (hardcoded, cannot be false)
- [x] `RollbackManifest` with `dry_run_only: true` (hardcoded, cannot be false)
- [x] LOCKED and REVIEW items excluded from manifest entries
- [x] MEDIUM/HIGH/CRITICAL impact items excluded from manifest entries
- [x] `excluded_for_safety` counter tracks all excluded items
- [x] `safesort preflight <MANIFEST>` — 8-check preflight engine (moves nothing)
- [x] Preflight checks: JSON validity, dry_run_only, no LOCKED, no high-impact, source exists, checksum match, size match, safe destination
- [x] Hardened `apply` requires both `--confirm` and `--i-understand-this-moves-files`
- [x] Apply runs preflight internally, then refuses with "still disabled in MVP"
- [x] 217 tests passing (82 in safety_tests.rs, 12 new preflight tests)

## Phase 4: Checksum and Rollback Manifest (Advanced)

**Status**: Partially implemented — manifest generation and preflight done.
Remaining items for full apply:

- [x] SHA-256 checksums (implemented in Phase 3)
- [x] JSON manifest with source → destination mapping (implemented in Phase 3)
- [x] Post-plan checksum verification via preflight (implemented in Phase 3)
- [ ] Atomic filesystem moves (apply still disabled)
- [ ] `safesort rollback <manifest>` command
- [ ] Automatic manifest backup
- [ ] Manifest signing (detect tampering)

## Phase 5: Guarded Apply with Freeze-State Rollback ✅ COMPLETE

**Status**: Implemented in v0.6.0. Real file movement enabled — fully gated.

> **Safety invariants unchanged.** LOCKED / REVIEW / MEDIUM / HIGH / CRITICAL items are never moved.
> Apply cannot run without all 4 explicit flags + valid manifest + passing preflight.

- [x] Both-flag requirement (`--confirm` + `--i-understand-this-moves-files`)
- [x] Preflight runs before any file is touched
- [x] `--backup` flag required — freeze-state copy of every source before move
- [x] `--apply-safe-only` flag required — only `auto_plan_eligible` entries move
- [x] Backup checksum verified before move
- [x] Destination parent directory created automatically
- [x] `fs::rename` for atomic-ish move (only in `src/apply/engine.rs`)
- [x] Destination checksum verified after move
- [x] Final destination path = planned_dir + source filename (never truncates at dir)
- [x] `--rollback-output` writes freeze-state receipt with per-file checksums
- [x] `safesort rollback <receipt>` — restores from backup; never removes directories
- [x] `safesort apply-status <receipt>` — read-only status display
- [x] Safe zone detection fix: Downloads files not penalized for inside_project
- [x] Dry-run works without any confirmation flags — shows plan, moves nothing
- [x] 277 tests passing (10 new destination-resolution tests, 5 new doctor tests)
- [x] Manual verification: 15 demo files moved, 15 restored, no real user files touched

## Phase 6: AI Summary Integration

**Status**: Planned

- [ ] Natural language summaries of scan results
- [ ] "What should I do?" recommendations
- [ ] AI-powered conflict resolution suggestions
- [ ] Historical scan comparison ("what changed since last time?")
- [ ] Weekly organization reports
- [ ] Integration with Claude API / OpenRouter

## Phase 7: Tauri Desktop GUI

**Status**: Planned

- [ ] Desktop application with Tauri + web frontend
- [ ] Drag-and-drop folder organization
- [ ] Visual dependency graph
- [ ] Before/after folder tree comparison
- [ ] One-click rollback
- [ ] System tray integration
- [ ] Cross-platform (Linux, macOS, Windows)

## Phase 8: Premium/Pro Release

**Status**: Planned

- [ ] Advanced AI features (Phase 6)
- [ ] Desktop GUI (Phase 7)
- [ ] Cloud sync for scan history
- [ ] Team/organization features
- [ ] Audit logging
- [ ] Commercial licensing
- [ ] Priority support
