# Scripts

Reusable automation scripts for Tauri projects.

## Quick Reference

| Script                         | Purpose                                           |
| ------------------------------ | ------------------------------------------------- |
| `release.ts`                   | Version bumping, changelog update, git tag & push |
| `setup-updater-signing.ts`     | Code signing key generation and configuration     |
| `ci/prepare-artifacts.ts`      | Collect & rename Tauri build outputs              |
| `ci/generate-manifest.ts`      | Generate Tauri updater manifest (`latest.json`)   |
| `ci/generate-release-notes.ts` | Assemble GitHub Release body from notes + git log |

## Usage

### Release

```bash
# Via Just (recommended)
just release-patch      # Patch bump: 0.2.1 -> 0.2.2
just release-minor      # Minor bump: 0.2.1 -> 0.3.0
just release-major      # Major bump: 0.2.1 -> 1.0.0
just release 1.2.3      # Exact version

# Direct (Bun)
bun scripts/release.ts --patch
bun scripts/release.ts 1.2.3 --no-push
bun scripts/release.ts --patch --all   # Stage all changes
```

### Code Signing Setup

```bash
bun scripts/setup-updater-signing.ts
```

### CI Scripts

These are primarily called by GitHub Actions but can also be run locally:

```bash
# Prepare artifacts after a Tauri build
bun scripts/ci/prepare-artifacts.ts --platform windows --version 0.3.4 --target x86_64-pc-windows-msvc

# Generate updater manifest from collected artifacts
bun scripts/ci/generate-manifest.ts --version 0.3.4 --tag v0.3.4 --repo owner/repo

# Generate release notes
bun scripts/ci/generate-release-notes.ts --version 0.3.4 --tag v0.3.4 --repo owner/repo
```

## Requirements

- Bun (recommended) or Node.js 18+
- Git configured
- For release: `RELEASE_NOTE.md` with content

## Options

### release.ts

| Option                          | Description                                |
| ------------------------------- | ------------------------------------------ |
| `--patch`, `--minor`, `--major` | Version bump type                          |
| `<X.Y.Z>`                       | Exact version                              |
| `--no-push`                     | Skip push to remote                        |
| `--all`, `-a`                   | Stage all changes (not just release files) |

## See Also

- [lib/README.md](lib/README.md) - Shared utilities documentation
- [ci/README.md](ci/README.md) - CI scripts documentation
