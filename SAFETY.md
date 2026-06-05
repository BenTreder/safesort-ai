# SafeSort AI — Safety Design

## Core Philosophy

> **AI explains. Rust safety engine decides.**

SafeSort AI is built on one non-negotiable principle:

**Never move anything it cannot prove is safe.**

## The Three Pillars

### 1. No Delete by Default

SafeSort AI never deletes files. Not during scanning, not during planning, not during apply. Deletion is not a feature — it's a last resort that requires explicit user consent and a verified backup.

### 2. No Move Without Proof

Before any file could ever be moved, SafeSort AI requires:

1. **Positive identification** — What is this file/folder?
2. **Dependency check** — Is anything referencing this path?
3. **Safety classification** — Is it SAFE, REVIEW, or LOCKED?
4. **User confirmation** — Has a human approved this specific move?
5. **Rollback manifest** — Can we undo this if something breaks?

### 3. Dry-Run First

Every operation starts as a dry run. The user sees exactly what *would* happen before anything *does* happen. The `apply` command remains disabled.

### 4. Impact Before Action

Every scan now reports an **impact level** derived from evidence:

| Impact | Trigger |
|---|---|
| 🔴 **CRITICAL** | `.env`, credentials, systemd references, cron references, live website folders |
| 🟠 **HIGH** | Symlinks, shell scripts with absolute path references |
| ⚠️ **MEDIUM** | Active project directories (`.git`, `Cargo.toml`, `package.json`, etc.) |
| 🟢 **LOW** | Loose media, documents, archives in Downloads/Desktop |
| ✅ **NONE** | Items with no dependency signals |

Safe Autopilot only ever considers items with **NONE** or **LOW** impact for auto-planning. MEDIUM/HIGH/CRITICAL items are always routed to human review.

### 5. Read-Only Custom Rule Files

Rule files let users add aliases, protected paths, and custom staging destinations. Safety guarantees enforced unconditionally:

- **Loaded on demand only** — never auto-loaded from `~/.safesort/` or any other path. Only activated when `--rule-file <FILE>` is explicitly passed.
- **No file operations** — rules never create, move, rename, delete, or copy any file. They produce recommendation text only.
- **No persistence** — rules are not written to disk. Each run loads fresh from the specified file. Engine state is not shared between runs.
- **No safety bypass** — rules cannot promote a LOCKED or REVIEW item to SAFE_CANDIDATE. Safety classification is always re-derived from evidence.
- **Destination validation** — custom staging destinations are checked before use. Destinations matching `/etc`, `/usr`, `/var`, `/boot`, `public_html`, `htdocs`, `www`, `live-site`, `live_site`, `webroot`, or other restricted patterns are rejected with a warning. The rejection reason is shown in the recommendation.
- **Protected paths** — paths listed under `[protected_paths]` are treated as LOCKED roots. Their children inherit REVIEW classification via the existing parent-risk inheritance pass. No filesystem changes occur.
- **Aliases** — only affect ownership detection in recommendations. They do not affect safety classification.
- **Safe Autopilot** — cannot auto-plan rule-protected items, items with MEDIUM/HIGH/CRITICAL impact, or items with risky custom destinations.

### 6. Depth and Exclude Controls

SafeSort AI supports two traversal controls for managing large or complex directories:

- **`--depth <N>`** limits filesystem traversal to N levels deep (default: 2). Items beyond the depth limit are never scanned, classified, or recommended for placement.
- **`--exclude <PATTERN>`** (repeatable) removes items from the pipeline entirely if their name or path substring matches the pattern. Excluded items are:
  - Counted in the `SKIPPED` line of the safety summary
  - Never classified as SAFE, REVIEW, or LOCKED
  - Never auto-plan eligible (Safe Autopilot cannot touch them)
  - Never shown in placement recommendations

Example: `--exclude node_modules --exclude target` keeps build artifacts out of all scan results.

### 6. Parent-Risk Inheritance

A file cannot be `SAFE_CANDIDATE` if its parent directory is not safe. SafeSort AI applies a second classification pass after scanning:

- **Child of LOCKED directory** → upgraded to `REVIEW` with `HIGH` impact
- **Child inside a live-site folder** (`public_html/`, `www/`, `htdocs/`, `webroot/`, `live-site/`, `live_site/`) → upgraded to `REVIEW` with `HIGH` impact
- **Directory containing `.env` or credentials** → upgraded to `LOCKED` (so its children then inherit REVIEW)

This ensures files like `public_html/index.php` or `ImportantApp/config.yml` are never presented as safe to auto-organize.

## Classification System

### 🔒 LOCKED — Never Move

A path is LOCKED when any of these conditions are true:

- **System-critical paths**: `/etc`, `/usr`, `/var`, `/opt`, `/boot`, `/srv`, `/run`, `/proc`, `/sys`, `/dev`
- **Sensitive home directories**: `~/.ssh`, `~/.gnupg`, `~/.aws`, `~/.config`, `~/.kube`, `~/.docker`, `~/.local/share`
- **Sensitive files**: `.env`, `id_rsa`, `id_ed25519`, `.npmrc`, `.pypirc`, `*.pem`, `*.key`, files containing "secret", "credential", "token", "private_key"
- **Private folders**: Any folder starting with `private_`
- **Symlink targets**: Anything that is the target of a symlink
- **Systemd-referenced**: Any path referenced in a systemd unit file (`ExecStart`, `WorkingDirectory`, `EnvironmentFile`, `ReadWritePaths`, `ReadOnlyPaths`, etc.)
- **Cron-referenced**: Any path referenced in a cron entry
- **Live websites**: Folders named `public_html`, `htdocs`, `www`, `website`, `site`
- **Mixed/unknown**: Directories with no recognizable markers (default to REVIEW, not SAFE)

### ⚠️ REVIEW — Human Decision Required

A path is marked REVIEW when:

- **Project markers found**: `.git`, `Cargo.toml`, `package.json`, `composer.json`, `pyproject.toml`, `requirements.txt`, `wp-config.php`, `Dockerfile`, `docker-compose.yml`, `Makefile`
- **Contains scripts**: Shell scripts, Python files, PHP files
- **Contains Docker files**: Dockerfile, docker-compose.yml
- **Unknown mixed contents**: Directories with no clear purpose
- **Symlinks**: The symlink itself (target is LOCKED)
- **Archives in projects**: ZIP/tar files inside active project directories

### ✅ SAFE CANDIDATE — Recommended for Organization

A path is SAFE only when:

- **Loose in safe zones**: File is directly inside `Downloads` or `Desktop`
- **Media files**: `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.mp3`, `.wav`, `.mp4`, `.mkv`, `.avi`, `.mov`
- **Documents**: `.pdf`, `.doc`, `.docx`, `.txt`, `.md`, `.csv`, `.xlsx`
- **Archives**: `.zip`, `.tar.gz`, `.tgz`, `.bak` (only in safe zones)
- **No project markers**: No `.git`, no `Cargo.toml`, no `package.json`, etc.
- **No sensitive content**: No `.env`, no secrets, no credentials

## The Seven Detectors

### 1. ProjectDetector
Scans directories for project markers: `.git`, `Cargo.toml`, `package.json`, `composer.json`, `pyproject.toml`, `requirements.txt`, `wp-config.php`, `docker-compose.yml`, `Dockerfile`, `Makefile`, `node_modules`, `vendor`, `target`, `venv`, `.venv`.

### 2. SensitivePathDetector
Detects sensitive directories (`.ssh`, `.gnupg`, `.aws`, `.config`, `.kube`, `.docker`, `.password-store`) and sensitive files (`.env`, `id_rsa`, `id_ed25519`, `.npmrc`, `.pypirc`, `*.pem`, `*.key`).

### 3. SymlinkDetector
Detects symlinks and marks them for review. Marks symlink targets as LOCKED — moving a target breaks the link.

### 4. ScriptPathDetector
Reads text/script/config files and detects absolute path references (`/home/`, `/var/www/`, `/srv/`, `/opt/`, `~/`). Does NOT edit files — only reports references.

### 5. SystemdDetector
Read-only scan of systemd unit directories:
- `/etc/systemd/system`
- `/usr/lib/systemd/system`
- `/lib/systemd/system`
- `~/.config/systemd/user`

Looks for: `ExecStart`, `WorkingDirectory`, `EnvironmentFile`, `ReadWritePaths`, `ReadOnlyPaths`, `CacheDirectory`, `LogsDirectory`, `RuntimeDirectory`, `StateDirectory`.

**Any referenced path is LOCKED.**

If permission is denied, skips safely and reports the skip.

### 6. CronDetector
Read-only scan of cron directories:
- `/etc/crontab`
- `/etc/cron.d`
- `/etc/cron.daily`
- `/etc/cron.hourly`
- `/etc/cron.weekly`
- `/etc/cron.monthly`

**Any referenced path is LOCKED.**

If permission is denied, skips safely and reports the skip.

### 7. ArchiveDetector
Detects archive files (`.zip`, `.tar`, `.tar.gz`, `.tgz`, `.bak`, `.old`) and backup folders. Archives in safe zones (Downloads/Desktop) are SAFE_CANDIDATE. Archives inside projects are REVIEW.

## Apply Preflight (Phase 3 — Implemented)

SafeSort AI includes a `preflight` command that validates every safety gate before any hypothetical apply step. Preflight **never moves, copies, renames, or deletes any file**.

### Preflight checks (all must pass before any future apply):
1. Manifest loads as valid JSON
2. `dry_run_only = true`
3. No LOCKED entries in the manifest
4. No MEDIUM/HIGH/CRITICAL impact entries
5. All source files still exist on disk
6. All SHA-256 checksums still match (file unchanged since planning)
7. All file sizes still match
8. All planned destinations are safe (no system paths, live-site paths, `/www/`, `/etc/`, etc.)

### Why preflight before apply?
This pattern ensures that by the time apply is ever enabled:
- The filesystem state matches the plan (no surprise changes)
- No safety gate has been bypassed
- The checksum infrastructure is proven and tested before it matters

### Hardened apply (MVP — still disabled):
Even when both `--confirm` and `--i-understand-this-moves-files` are provided, apply runs preflight internally, then refuses with:
> "Apply preflight passed, but real file movement is still disabled in this MVP build."

This means apply is demonstrably safe: it has all the gates, it just refuses to pull the trigger.

## Rollback Manifest (Phase 3 — Implemented)

SafeSort AI now generates a **dry-run rollback manifest** with SHA-256 checksums. The manifest is created before any hypothetical move and contains everything a future apply step would need to verify the operation is safe.

### Manifest safety invariants (always enforced):
- `dry_run_only: true` is hardcoded in all manifest structs — it cannot be set to false
- Only SAFE_CANDIDATE files with NONE/LOW impact appear as entries
- LOCKED and REVIEW files are excluded and counted in `excluded_for_safety`
- HIGH/MEDIUM/CRITICAL impact items are excluded even if classified as SAFE_CANDIDATE
- The manifest command writes **only the requested JSON output file** — it never touches scanned files
- `apply` remains disabled; the manifest is purely informational

### Manifest generation:
```bash
safesort manifest --path ~/Downloads --output manifest.json
safesort plan --path ~/Downloads --mode guided --manifest-output manifest.json
```

### Future rollback design (Phase 4):
When apply is eventually enabled, the workflow will be:
1. **Manifest phase** (current): SHA-256 checksums + planned destinations, nothing moved
2. **Verify phase**: Confirm checksums still match before applying
3. **Apply phase**: Atomic moves with full manifest audit trail
4. **Rollback command**: `safesort rollback <manifest>` undoes all moves

## The "Workspace Overlay" Concept

SafeSort AI can organize your mental map without physically moving dangerous folders:

```
Actual disk:
  ~/Projects/OptionsCommand/paper-options-command-center

SafeSort categorization:
  Workspace > Active Projects > Trading Tools

The folder stays exactly where it is.
```

This is the recommended approach for active projects, client work, and anything with external dependencies.

## Smart Placement Safety

The Smart Placement Engine adds intelligence while maintaining safety:

### Safe Staging Only
SafeSort **never** recommends placing files directly into live website roots (`public_html/`, `htdocs/`, `www/`). All destinations are safe staging areas:
- `~/Workspace/06_Business/Brand Assets/{Owner}/Logos/` — not the live site
- `~/Workspace/03_Websites/{Site}/Incoming Assets/` — not the document root
- `~/Workspace/04_WordPress/Plugins/{Plugin}/Assets/` — not `wp-content/plugins/`

### Confidence Gating
- **≥95% (GREEN):** Auto-planned only in safe-autopilot mode
- **80–94% (YELLOW):** Guided review question — user decides
- **50–79%:** Review needed — no automatic action
- **<50%:** Leave alone — no recommendation

### Why Direct Live-Site Moves Are Disabled
1. **Breaking changes:** Moving assets into a live site can break references, caches, and deployments
2. **Version control:** Live sites may be under Git — uncommitted changes cause problems
3. **Permissions:** Live sites often have specific ownership/permission requirements
4. **Deployment pipelines:** Staged assets should go through the proper deployment process
5. **Rollback complexity:** Direct live changes are harder to undo

**The SafeSort workflow:** Stage → Review → Deploy (via your normal process)

### Downloads Triage Rules
- Loose files in Downloads/Desktop → eligible for smart placement
- Folders in Downloads with project markers → REVIEW (not auto-moved)
- Files with `.env`, secrets, credentials → LOCKED
- Files inside active projects → penalized confidence (not auto-moved)

## What Is Intentionally Disabled

In this Phase 1+ / Phase 2 foundation build:

| Feature | Status |
|---|---|
| Read-only scanning | ✅ Enabled |
| Safety classification | ✅ Enabled |
| Smart Placement Engine | ✅ Enabled |
| Ownership detection | ✅ Enabled |
| Purpose detection | ✅ Enabled |
| Confidence scoring | ✅ Enabled |
| Guided review mode | ✅ Enabled |
| Safe autopilot mode | ✅ Enabled |
| Locked-down mode | ✅ Enabled |
| Profile detection | ✅ Enabled |
| Folder structure recommendations | ✅ Enabled |
| Terminal/JSON/Markdown reports | ✅ Enabled |
| `apply` command | 🔒 Stub only — refuses to run ("Nothing was moved.") |
| Safe Autopilot | 🟡 Plan-only — produces plan, never moves files |
| Guided Review | 🟡 Plan-only — question queue only, never moves files |
| Dependency graph | 🔵 Foundation only — analysis, not wired to apply |
| Demo fixture path | `./safesort_demo/` |
| File moving | 🔒 Disabled |
| File deletion | 🔒 Disabled |
| Direct live-site moves | 🔒 Always disabled |
| SHA-256 checksum engine | ✅ Enabled (read-only) |
| Rollback manifest (dry-run) | ✅ Enabled — `safesort manifest` / `--manifest-output` |
| `safesort preflight <MANIFEST>` | ✅ Enabled — validates all safety gates, moves nothing |
| Hardened apply stub | ✅ Enabled — requires both flags, runs preflight, then refuses |
| Rollback manifest apply | 🔒 Phase 4 — apply disabled |
| Checksum verification on apply | ✅ Implemented in preflight (Phase 3) |
| AI summary integration | 🔒 Phase 6 |
| Tauri desktop GUI | 🔒 Phase 7 |

## Security Considerations

- SafeSort AI does NOT read private file contents (only filenames and marker files)
- SafeSort AI does NOT send data anywhere — everything is local
- SafeSort AI does NOT require root (but can use it for broader scanning)
- Permission denied errors are handled gracefully — the scanner skips and reports
- The safety engine is conservative: when in doubt, LOCK it out
