# Scripts

Reusable automation scripts for Tauri projects.

## Quick Reference

| Script                     | Purpose                                           |
| -------------------------- | ------------------------------------------------- |
| `release.ts`               | Version bumping, changelog update, git tag & push |
| `setup-updater-signing.ts` | Code signing key generation and configuration     |

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
