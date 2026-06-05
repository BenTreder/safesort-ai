# SafeSort AI — Safety-First Folder Organizer

> **AI explains. Rust safety engine decides.**

SafeSort AI is a premium Rust CLI application that organizes your folders *safely* — without breaking apps, scripts, projects, services, system files, or important paths.

## What It Does

SafeSort AI scans your filesystem and classifies every item into three safety categories:

| Classification | Meaning |
|---|---|
| 🔒 **LOCKED** | Never move. Protected by safety engine. System paths, secrets, `.ssh`, `.env` folders, private keys, and paths referenced by systemd/cron/scripts. |
| ⚠️  **REVIEW** | Needs human review. Git repos, project folders, Docker configs, mixed content, unknown directories. |
| ✅ **SAFE CANDIDATE** | Safe to recommend for organization. Loose screenshots, PDFs, media files, archives in Downloads/Desktop. |

### Smart Placement Engine

SafeSort AI doesn't just organize by file type — it organizes by **ownership, purpose, project, brand, and confidence**.

**Example:** If `~/Downloads/bentreder_logo.png` exists, SafeSort understands:
- "bentreder" → BenTreder.com / Ben Treder Digital
- "logo" → brand/logo asset
- ".png" → image asset
- **Recommended destination:** `~/Workspace/06_Business/Brand Assets/BenTreder/Logos/`

SafeSort **never** places files directly into live website roots by default. It uses safe staging destinations.

## Organization Modes

### Preview Mode (default)
```bash
safesort scan --path ~/Downloads
safesort scan --path ~/Downloads --mode preview
```
Shows safety classification and placement recommendations. Never moves anything.

### Guided Review Mode
```bash
safesort plan --path ~/Downloads --mode guided
```
Creates a question queue for uncertain files (80–94% confidence). Asks where questionable files should go. Allows creating future rules. Does not apply moves.

### Safe Autopilot Mode
```bash
safesort plan --path ~/Downloads --mode safe-autopilot
```
Only auto-plans files with ≥95% confidence (GREEN) **and NONE/LOW impact**. Never auto-plans items with MEDIUM, HIGH, or CRITICAL impact. Never moves LOCKED or REVIEW items. Only uses safe staging destinations. Produces a plan only — apply is disabled.

### Locked-Down Mode
```bash
safesort scan --home --mode locked-down
```
Extra conservative. Caps confidence at 80. Never recommends automatic movement. Ideal for first-time scans.

## Key Principles

### Read-Only First

**This build is 100% read-only.** It does NOT:
- Move, copy, or delete files
- Rename or chmod/chown anything
- Edit config files, systemd units, or cron jobs
- Touch real user files except to create the Rust project itself

The `apply` command exists as a stub that refuses to run.

### Impact Visibility

Every scan now reports an **impact level** per item and a summary count:

```
  Impact summary:
    🔴     CRITICAL      10   ← .env, credentials, systemd refs
    🟠         HIGH       1   ← symlinks, script path refs
    ⚠️        MEDIUM       4   ← active projects (.git, Cargo.toml…)
    🟢          LOW      19   ← loose media, docs, archives
    ✅         NONE      11   ← no dependency signals
```

Safe Autopilot only ever auto-plans **NONE/LOW** impact items.

### Safety-First Design

SafeSort AI will **refuse to move anything it cannot prove is safe**. The safety engine uses seven detectors:

1. **ProjectDetector** — Finds `.git`, `Cargo.toml`, `package.json`, `composer.json`, `pyproject.toml`, `Dockerfile`, `Makefile`, and more
2. **SensitivePathDetector** — Detects `.ssh`, `.gnupg`, `.aws`, `.config`, `.kube`, `.docker`, private keys, API token files
3. **SymlinkDetector** — Detects symlinks; marks symlink targets as LOCKED
4. **ScriptPathDetector** — Reads scripts/configs and detects absolute path references (`/home/`, `/var/www/`, `/srv/`, `~/`)
5. **SystemdDetector** — Scans systemd unit files for `ExecStart`, `WorkingDirectory`, `EnvironmentFile`, `ReadWritePaths`, etc.
6. **CronDetector** — Scans cron entries for path references
7. **ArchiveDetector** — Detects `.zip`, `.tar.gz`, `.bak`, backup folders

### Smart Placement Engine

The placement engine adds intelligence on top of safety classification:

- **OwnershipDetector** — Detects brand/project/owner from filename tokens and path context (e.g. "bentreder" → BenTreder.com)
- **FilePurposeDetector** — Detects purpose: logo, banner, screenshot, report, invoice, release zip, etc.
- **DestinationPlanner** — Recommends safe staging destinations based on ownership + purpose + profile
- **ConfidenceScorer** — Scores recommendations 0–100 based on signal strength
- **Rules System** — User-defined rules for custom placement patterns
- **QuestionQueue** — Interactive guided review for uncertain files

### Confidence Scoring

| Score | Band | Action |
|---|---|---|
| 95–100 | 🟢 AUTO-PLAN | Auto-planned in safe-autopilot mode |
| 80–94 | 🟡 GUIDED REVIEW | Question created for user decision |
| 50–79 | ⚠️ REVIEW NEEDED | Flagged for manual review |
| 0–49 | ⬜ LEAVE ALONE | No recommendation |

**Scoring factors:**
- Exact brand/project token match: +40
- Purpose token match: +25
- Safe file type: +10
- Source is Downloads/Desktop: +10
- Matching known project exists: +10
- Loose file (not in project): +5
- Extension signals purpose: +5
- Ambiguous multiple matches: −30
- Inside active project: −40
- Sensitive keyword: force LOCKED/REVIEW

### Workspace Overlay

SafeSort AI introduces a "Workspace Overlay" concept — it categorizes your files mentally **without physically moving dangerous folders**:

```
Actual path: ~/Projects/OptionsCommand/paper-options-command-center
SafeSort categorizes it as: Workspace > Active Projects > Trading Tools
The folder is untouched.
```

### Safe Staging Destinations

SafeSort **never** recommends placing files directly into live website roots. Instead, it uses safe staging areas:

```
~/Workspace/06_Business/Brand Assets/{Owner}/Logos/
~/Workspace/03_Websites/{Website}/Incoming Assets/
~/Workspace/04_WordPress/Plugins/{Plugin}/Assets/
~/Workspace/04_WordPress/Plugins/{Plugin}/Release Zips/
~/Workspace/09_Reports/Website Audits/
~/Workspace/08_Archives/ZIP Archives/
~/Workspace/99_Review Needed/
```

### Downloads Triage

Special handling for Downloads:
- Loose image/PDF/archive files → SAFE_CANDIDATE
- Downloaded app folders → REVIEW
- Extracted code folders → REVIEW
- Anything with `.env` → LOCKED
- Anything with scripts → REVIEW
- Anything with project markers → REVIEW
- Anything referenced elsewhere → LOCKED

### User Profile Detection

SafeSort AI infers your user type from folder names and project markers:

- Developer • WordPress Plugin Builder • Website Owner • AI Power User
- SEO/Content Creator • Client-Service Freelancer • Designer/Media Creator
- Business Owner • Data/Reports User • General User

Based on the detected profile, it recommends a beautiful folder structure.

## Installation

```bash
git clone https://github.com/safesort-ai/safesort.git
cd safesort
cargo build --release
```

The binary will be at `target/release/safesort`.

## Custom Rule Files

SafeSort AI supports optional, read-only rule files in TOML format that customize aliases, protected paths, and staging destination recommendations.

```bash
safesort scan --path ./safesort_demo --rule-file ./examples/safesort-rules.toml
safesort plan --path ./safesort_demo/Downloads --mode guided --rule-file ./examples/safesort-rules.toml
safesort explain ./safesort_demo/ImportantApp --rule-file ./examples/safesort-rules.toml
```

### What rule files can do
- **Aliases** — map filename tokens (e.g. `acme`) to a canonical owner/brand (`ACME Corp`)
- **Protected paths** — mark specific directories as LOCKED and never auto-plan eligible
- **Staging destinations** — override recommended destination paths per `{owner}.{purpose}` pair
- **Owner metadata** — provide display names, categories, and safe staging roots

### What rule files cannot do
- Move, rename, delete, copy, or modify any file
- Bypass LOCKED or REVIEW safety classifications
- Make unsafe items auto-plan eligible
- Persist to disk or auto-load from `~/.safesort/`
- Override risky destinations (live-site, system paths are rejected automatically)

### Rule file format

```toml
[aliases]
"mybrand" = "MyBrand"
"my-brand" = "MyBrand"

[owners."MyBrand"]
display   = "My Brand Inc."
category  = "Brand"
safe_root = "~/Workspace/MyBrand/Incoming"

[protected_paths]
paths = [
  "./ImportantApp",
  "/srv/production"
]

[staging_destinations]
"MyBrand.logo"       = "~/Workspace/Brand/MyBrand/Logos/"
"MyBrand.document"   = "~/Workspace/Brand/MyBrand/Docs/"
```

See `examples/safesort-rules.toml` for a fully annotated example.

## Commands

### `safesort doctor`
Run environment and permission diagnostics.

### `safesort demo-fixture`
Generate fake test fixtures for demonstration and testing.

### `safesort scan`
Scan a path and classify every item by safety.

```bash
safesort scan --path ~/Downloads
safesort scan --home
safesort scan --path ~/Downloads --mode preview      # default
safesort scan --path ~/Downloads --mode locked-down  # extra conservative
safesort scan --path ~/Downloads --format json
safesort scan --path ~/Downloads --format markdown --output report.md

# Limit traversal depth (default: 2)
safesort scan --path ~/Projects --depth 3

# Exclude paths matching a name or substring (repeatable)
safesort scan --path ~/Projects --exclude node_modules --exclude target
safesort scan --path ~/Sites --exclude wp-content
```

Excluded items appear in the `SKIPPED` count of the safety summary but are never classified, never recommended for placement, and never eligible for auto-planning.

### `safesort plan`
Generate a smart placement plan with recommendations.

```bash
safesort plan --path ~/Downloads --mode preview          # recommendations only
safesort plan --path ~/Downloads --mode guided           # interactive questions
safesort plan --path ~/Downloads --mode safe-autopilot   # auto-plan ≥95% confidence
safesort plan --path ~/Downloads --output plan.json      # export plan

# With depth and exclude controls
safesort plan --path ~/Projects --mode guided --depth 3
safesort plan --path ~/Sites --mode safe-autopilot --exclude wp-content --exclude node_modules

# Write a dry-run rollback manifest with SHA-256 checksums (nothing is moved)
safesort plan --path ~/Downloads --mode guided --manifest-output manifest.json
```

### `safesort manifest`
Generate a dry-run rollback manifest with SHA-256 checksums. Nothing is moved.

```bash
safesort manifest --path ~/Downloads
safesort manifest --path ~/Downloads --output manifest.json
safesort manifest --path ~/Projects --depth 3 --exclude node_modules
safesort manifest --path ~/Downloads --rule-file ./examples/safesort-rules.toml --output manifest.json
```

The manifest is a JSON document describing what a future apply step *would* do. It contains:
- SHA-256 checksums for each SAFE_CANDIDATE file (so a future apply can verify nothing changed)
- `dry_run_only: true` — always set, applies forever
- Only SAFE_CANDIDATE files with NONE/LOW impact appear as entries; LOCKED and REVIEW items are excluded and counted separately
- `excluded_for_safety` — count of files excluded from entries due to LOCKED/REVIEW/HIGH+ impact

### `safesort profile`
Analyze user profile and recommend folder structure.

### `safesort explain`
Explain the safety decision for a specific path.

```bash
safesort explain ./safesort_demo/ImportantApp
safesort explain ./safesort_demo/ImportantApp --rule-file ./examples/safesort-rules.toml
```

### `safesort apply`
**DISABLED in this safety-first build.**

## Examples

### Smart Placement Plan (Guided Mode)

```
  SafeSort AI — Smart Placement Plan
  Target: /home/user/Downloads
  Mode: guided

  Placement Summary:
    Total files scanned:    45
    🔒 Locked:              3
    🟡 Guided review:       5
    ⚠️  Review needed:       12
    ⬜ Leave alone:          25

  ┌─────────────────────────────────────────────
  │ File:       /home/user/Downloads/bentreder_logo.png
  │ Owner:      Ben Treder Digital (BenTreder.com)
  │ Purpose:    Logo
  │ Type:       Image
  │ Risk:       GREEN
  │ Confidence: 94%
  │ Dest:       Brand Assets → BenTreder → Logos
  │ Path:       ~/Workspace/06_Business/Brand Assets/BenTreder/Logos
  │ Why:        Filename matches brand/project 'Ben Treder Digital';
  │             Purpose detected: Logo; Source is Downloads/Desktop (safe zone);
  │             Confidence: 94%
  │ Action:     GUIDED REVIEW
  └─────────────────────────────────────────────

  Nothing was moved.
```

## Why Systemd/Cron/Scripts Matter

Many folders are silently referenced by:
- **Systemd services** — Moving `/opt/my-app` breaks a running service
- **Cron jobs** — Moving `/home/user/scripts/backup.sh` breaks nightly backups
- **Shell scripts** — Absolute paths in scripts break silently when files move
- **Docker mounts** — Moving a mounted volume breaks containers
- **Symlinks** — Moving a symlink target breaks the link

SafeSort AI scans for these references *before* anything is ever moved.

## Architecture

### Dependency Graph (Phase 2 Foundation)

SafeSort AI includes a Phase 2 foundation dependency graph in `src/graph/` that explains what *would* break before any future apply command is ever enabled:

- **`.git`, `Cargo.toml`** → Medium impact (active Rust project)
- **`package.json`** → Medium impact (active Node.js project)
- **`composer.json`** → Medium impact (active PHP/Composer project)
- **`pyproject.toml`** → Medium impact (active Python project)
- **`.env` file** → Critical impact (secret exposure risk)
- **Systemd/cron references** → Critical impact (would break services)

The graph is analysis-only. It feeds safety classification. It never moves anything.

**Workspace Overlay is the preferred approach** for active projects: SafeSort categorizes them mentally without touching the folder.

```
src/
  main.rs          — Entry point
  lib.rs           — Library root
  cli.rs           — Command-line interface (clap)
  app.rs           — Command implementations
  config.rs        — Constants and configuration
  error.rs         — Error types (thiserror)

  graph/           — Dependency graph (Phase 2 foundation)
    dependency_graph.rs — DependencyGraph with impact analysis
    impact.rs           — ImpactLevel enum + ImpactAnalysis
    node.rs             — Node types (Path, Service, Script, Project, Sensitive, Symlink)
    edge.rs             — Edge types + EdgeKind

  scan/            — Core scanning engine
    walker.rs      — Read-only filesystem walker
    item.rs        — Scan item representation
    classifier.rs  — Safety classification engine
    evidence.rs    — Evidence types from detectors
    risk.rs        — Risk scores and safety levels

  detectors/       — Seven safety detectors
    projects.rs    — Project marker detection
    sensitive.rs   — Sensitive path/file detection
    symlinks.rs    — Symlink detection
    scripts.rs     — Script path reference detection
    systemd.rs     — Systemd unit scanning
    cron.rs        — Cron entry scanning
    archives.rs    — Archive file detection

  placement/       — Smart Placement Engine
    engine.rs      — SmartPlacementEngine orchestrator
    ownership.rs   — OwnershipDetector (brand/project detection)
    file_purpose.rs— FilePurposeDetector (logo, banner, etc.)
    destination.rs — DestinationPlanner (safe staging paths)
    confidence.rs  — ConfidenceScorer (0–100 scoring)
    rules.rs       — RulesEngine (user-defined placement rules)
    question_queue.rs — QuestionQueue (guided review)

  profile/         — User profiling
    user_profile.rs         — Profile inference and scoring
    signals.rs              — Signal weights for profile detection
    folder_structure.rs     — Recommended folder structures

  safety/          — Safety policy
    policy.rs      — Top-level safety policy (aggregation)
    rules.rs       — Individual safety rules

  reports/         — Output formatting
    terminal.rs    — Premium terminal output
    json.rs        — JSON report generation
    markdown.rs    — Markdown report generation
```

## License

MIT

## See Also

- [SAFETY.md](SAFETY.md) — Safety design principles
- [ROADMAP.md](ROADMAP.md) — Development roadmap
- [PROJECT_CHECKPOINT.md](PROJECT_CHECKPOINT.md) — Current project status
