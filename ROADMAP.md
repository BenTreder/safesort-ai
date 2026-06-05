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

## Phase 2: Dependency Graph — Foundation ⚠️ PARTIAL (see below)

## Phase 2: Dependency Graph — Foundation ⚠️ IN PROGRESS (foundation only)

**Status**: Foundation landed in v0.2.1 — graph types, impact analysis, and evidence-based
classification exist, but no cross-referencing or wiring to `apply`. `apply` remains disabled.

> **Demo fixture path**: `./safesort_demo/`
> **Apply**: still disabled — prints "Nothing was moved."
> **Safe Autopilot**: plan-only — no moves, no apply.
> **Workspace Overlay**: preferred pattern for active projects (categorize without touching).
> **Dependency graph**: explains what *would* break before any future apply ever exists.

- [x] DependencyGraph struct with node/edge model (`src/graph/`)
- [x] ImpactLevel enum (None / Low / Medium / High / Critical)
- [x] `analyze_impact_from_evidence` — converts scan evidence to impact level
- [x] `analyze_project_impact` — .git / Cargo.toml / package.json → Medium+ impact
- [x] `analyze_sensitive_folder_impact` — .env → Critical impact
- [x] 11 tests covering project markers (.git, Cargo.toml, package.json, composer.json, pyproject.toml), .env → Critical, systemd → Critical, active projects → Medium+
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
- [ ] `--depth` and `--exclude` flags on scan command
- [ ] `--rule-file` flag for custom alias/destination rules
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

## Phase 3: Plan Generation

**Status**: Planned

- [ ] Generate a detailed move plan (JSON format)
- [ ] Show exactly what would move where
- [ ] Show what would NOT move and why
- [ ] Show what needs human review
- [ ] Interactive plan review (step through each item)
- [ ] Plan validation (check for conflicts before showing)
- [ ] Workspace Overlay mapping (categorize without moving)
- [ ] Export plan as JSON for later apply

## Phase 4: Checksum and Rollback Manifest

**Status**: Planned

- [ ] SHA-256 checksums before any move
- [ ] JSON manifest with source → destination mapping
- [ ] Atomic filesystem moves
- [ ] Post-move checksum verification
- [ ] `safesort rollback <manifest>` command
- [ ] Automatic manifest backup
- [ ] Manifest signing (detect tampering)

## Phase 5: Carefully Gated Apply Command

**Status**: Planned

- [ ] Multi-step confirmation for each move batch
- [ ] Dry-run is always the default
- [ ] `--yes-i-really-mean-it` flag (with 10-second delay)
- [ ] Per-item confirmation mode
- [ ] Apply with automatic rollback on failure
- [ ] Progress bar during apply
- [ ] Post-apply verification scan
- [ ] Email/Slack notification on completion (optional)

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
