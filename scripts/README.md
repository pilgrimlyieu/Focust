# Release Scripts

Automated release scripts for Focust version management.

## Quick Start

```bash
# Recommended: Use Just commands
just release-patch    # 0.2.11 -> 0.2.12
just release-minor    # 0.2.11 -> 0.3.0
just release-major    # 0.2.11 -> 1.0.0
just release 1.2.3    # Specify exact version
```

## Scripts

- `release.ps1` - PowerShell script (Windows, cross-platform)
- `release.sh` - Bash script (Linux, macOS)

## Requirements

1. Write release notes in `RELEASE_NOTE.md` before running
2. Be on `main` branch with clean working directory
3. Have Git configured properly

## What It Does

1. Updates version in `package.json` and `tauri.conf.json`
2. Extracts content from `RELEASE_NOTE.md` and adds to `CHANGELOG.md`
3. Commits changes: `chore: bump version to vX.Y.Z`
4. Creates Git tag: `vX.Y.Z`
5. Pushes to remote (with confirmation)

## Usage Examples

### Via Just (Recommended)

```bash
# Bump versions
just release-patch
just release-minor
just release-major

# Specify version
just release 1.2.3
```

### Direct Execution

**PowerShell (Windows):**
```powershell
.\scripts\release.ps1 --Patch
.\scripts\release.ps1 --Minor
.\scripts\release.ps1 --Major
.\scripts\release.ps1 1.2.3
.\scripts\release.ps1 1.2.3 -NoPush  # Skip push
```

**Bash (Linux/macOS):**
```bash
./scripts/release.sh --patch
./scripts/release.sh --minor
./scripts/release.sh --major
./scripts/release.sh 1.2.3
./scripts/release.sh 1.2.3 --no-push  # Skip push
```

## GPG Signing

To sign the commit with GPG after release:

```bash
git commit --amend -S --no-edit
git push origin main --force-with-lease
```

## Full Documentation

See [docs/RELEASE_WORKFLOW.md](../docs/RELEASE_WORKFLOW.md) for complete workflow documentation.
