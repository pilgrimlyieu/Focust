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

- `release.ts` - TypeScript script (cross-platform, runs with Bun/Node.js)
- `setup-updater-signing.ts` - TypeScript script to setup code signing for updater

## Requirements

1. Bun (recommended) or Node.js 18+
2. Write release notes in `RELEASE_NOTE.md` before running
3. Be on `main` branch with clean working directory
4. Have Git configured properly

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

**Using Bun (recommended):**
```bash
bun scripts/release.ts --patch
bun scripts/release.ts --minor
bun scripts/release.ts --major
bun scripts/release.ts 1.2.3
bun scripts/release.ts 1.2.3 --no-push  # Skip push
```

**Using Node.js:**
```bash
node --loader ts-node/esm scripts/release.ts --patch
# Or with tsx:
npx tsx scripts/release.ts --patch
```

## GPG Signing

To sign the commit with GPG after release:

```bash
git commit --amend -S --no-edit
git push origin main --force-with-lease
```

## Full Documentation

See [docs/RELEASE_WORKFLOW.md](../docs/RELEASE_WORKFLOW.md) for complete workflow documentation.
