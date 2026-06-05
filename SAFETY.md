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

Every operation starts as a dry run. The user sees exactly what *would* happen before anything *does* happen. The `apply` command is disabled in Phase 1.

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

## Future Rollback Design

In Phase 4, SafeSort AI will implement a checksum and rollback manifest:

1. **Before any move**: Generate SHA-256 checksums of all files being moved
2. **Create manifest**: A JSON file recording every source → destination mapping
3. **Atomic moves**: Use filesystem-level move operations
4. **Verification**: After move, verify checksums match
5. **Rollback command**: `safesort rollback <manifest>` undoes all moves

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
| Rollback manifest | 🔒 Phase 4 |
| Checksum verification | 🔒 Phase 4 |
| AI summary integration | 🔒 Phase 6 |
| Tauri desktop GUI | 🔒 Phase 7 |

## Security Considerations

- SafeSort AI does NOT read private file contents (only filenames and marker files)
- SafeSort AI does NOT send data anywhere — everything is local
- SafeSort AI does NOT require root (but can use it for broader scanning)
- Permission denied errors are handled gracefully — the scanner skips and reports
- The safety engine is conservative: when in doubt, LOCK it out
