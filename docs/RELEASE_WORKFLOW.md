# Release Workflow with Code Signing

## Overview

The Focust project now supports automated code signing in CI/CD pipelines. This ensures secure auto-updates and builds trust with users.

## What's New

### 1. ✅ GitHub Actions Workflow Enhanced

**File**: `.github/workflows/release.yml`

**Changes**:
- Added code signing setup step
- Configured environment variables for signing keys
- Automatic generation of `.sig` signature files
- Generation of `latest.json` manifest for auto-updates
- Support for all platforms (Windows, Linux, macOS)

**Features**:
- 🔐 Builds are automatically signed when keys are configured
- 📦 Signature files (`.sig`) are included in releases
- 🔄 `latest.json` is generated with platform-specific update info
- ⚡ Falls back gracefully if keys are not configured

### 2. 📚 Documentation

**Files**:
- `docs/UPDATER_SIGNING.md` - Comprehensive guide for setting up updater signing
- `README.md` - Updated with signing information
- `.env.example` - Example environment variables

**Topics Covered**:
- Key generation process
- Local development setup
- GitHub Secrets configuration
- Security best practices
- Troubleshooting guide

### 3. 🛠 Helper Scripts

**Files**:
- `scripts/setup-updater-signing.sh` - Bash script (Linux/macOS)
- `scripts/setup-updater-signing.ps1` - PowerShell script (Windows)

**Features**:
- Interactive key generation
- Automatic configuration updates
- Creates `.env` file for local development
- Provides GitHub setup instructions

### 4. 🔧 Build Commands

**File**: `justfile`

**New Command**:
```bash
just build
```

Builds the application with code signing enabled (requires key setup).

## Quick Start for Maintainers

### First-Time Setup

1. **Generate signing keys**:
   ```bash
   # Using Bun (recommended, cross-platform)
   bun scripts/setup-updater-signing.ts
   
   # Using Node.js
   npx tsx scripts/setup-updater-signing.ts
   
   # Or via Just command
   just setup-signing
   ```

2. **Add secrets to GitHub**:
   - Go to repository Settings → Secrets → Actions
   - Add `TAURI_SIGNING_PRIVATE_KEY` (content of your private key)
   - Optionally add `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` if key is protected

3. **Commit and push**:
   ```bash
   git add src-tauri/tauri.conf.json
   git commit -m "chore: configure code signing public key"
   git push
   ```

### Making a Release

#### Automated Release (Recommended)

Use the release scripts for streamlined version management:

1. **Prepare release notes**:
   - Edit `RELEASE_NOTE.md` with your changes

2. **Run release script**:
   ```bash
   # Via Just (recommended)
   just release-patch    # Bump patch version (0.2.1 -> 0.2.2)
   just release-minor    # Bump minor version (0.2.1 -> 0.3.0)
   just release-major    # Bump major version (0.2.1 -> 1.0.0)
   just release 1.2.3    # Specify exact version
   
   # Direct execution with Bun (cross-platform)
   bun scripts/release.ts --patch
   bun scripts/release.ts --minor
   bun scripts/release.ts --major
   bun scripts/release.ts 1.2.3
   
   # Using Node.js
   npx tsx scripts/release.ts --patch
   ```

3. **What the script does**:
   - ✅ Updates `package.json` and `tauri.conf.json` version numbers
   - ✅ Extracts content from `RELEASE_NOTE.md` and adds to `CHANGELOG.md`
   - ✅ Commits changes with message: `chore: bump version to vX.Y.Z`
   - ✅ Creates Git tag: `vX.Y.Z`
   - ✅ Pushes commit and tag to remote (with confirmation)

4. **(Optional) GPG sign the commit**:
   ```bash
   git commit --amend -S --no-edit
   git push origin main --force-with-lease
   ```

5. **Automatic CI/CD process**:
   - GitHub Actions builds for all platforms
   - Installers are signed automatically
   - Release is created with:
     - Signed installers
     - Signature files (`.sig`)
     - Update manifest (`latest.json`)
     - Release notes from CHANGELOG.md

#### Manual Release (Legacy)

If you prefer manual control:

1. **Update version**:
   - Edit `package.json` version
   - Edit `src-tauri/tauri.conf.json` version
   - Update `CHANGELOG.md` and `RELEASE_NOTE.md`

2. **Create and push tag**:
   ```bash
   git add package.json src-tauri/tauri.conf.json CHANGELOG.md
   git commit -m "chore: bump version to vX.Y.Z"
   git tag vX.Y.Z
   git push origin main
   git push origin vX.Y.Z
   ```

3. **Automatic process**:
   - GitHub Actions builds for all platforms
   - Installers are signed automatically
   - Release is created with artifacts

## How It Works

### Build Process

```
1. GitHub Actions triggered by tag push
   ↓
2. Setup environment (Bun, Rust, dependencies)
   ↓
3. Load signing key from secrets
   ↓
4. Build application (tauri build)
   ↓
5. Generate signature files (.sig)
   ↓
6. Package artifacts
   ↓
7. Generate latest.json manifest
   ↓
8. Create GitHub Release
```

### Auto-Update Flow

```
1. App checks for updates
   ↓
2. Fetches latest.json from GitHub
   ↓
3. Compares version
   ↓
4. Downloads installer (if newer)
   ↓
5. Verifies signature with public key
   ↓
6. Installs if signature valid
   ↓
7. Rejects if signature invalid
```

### latest.json Structure

```json
{
  "version": "v0.1.0",
  "notes": "Release notes URL",
  "pub_date": "2025-11-03T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "base64_signature_here",
      "url": "https://github.com/.../installer.msi"
    },
    "linux-x86_64": {
      "signature": "base64_signature_here",
      "url": "https://github.com/.../app.AppImage"
    },
    "darwin-x86_64": {
      "signature": "base64_signature_here",
      "url": "https://github.com/.../app.dmg"
    },
    "darwin-aarch64": {
      "signature": "base64_signature_here",
      "url": "https://github.com/.../app.dmg"
    }
  }
}
```

## Security Considerations

### Protected Information

**Never Commit**:
- ❌ Private key file (`*.key`)
- ❌ `.env` file (may contain key path/password)
- ❌ Key passwords

**Safe to Commit**:
- ✅ Public key (in `tauri.conf.json`)
- ✅ `.env.example` (template only)
- ✅ Workflow files
- ✅ Documentation

### GitHub Secrets

Secrets are encrypted and only exposed to workflows:
- Not visible in logs
- Not accessible in pull requests from forks
- Can be rotated without changing code

### Key Management

- Store private key securely (password manager, vault)
- Back up the key in a secure location
- Use password protection for the key
- Rotate keys periodically
- Revoke and regenerate if compromised

## Troubleshooting

### Common Issues

**Issue**: Build succeeds but no `.sig` files

**Solution**: 
- Verify `TAURI_SIGNING_PRIVATE_KEY` is set in GitHub Secrets
- Check workflow logs for signing errors
- Ensure key format is correct (no extra spaces/newlines)

---

**Issue**: Auto-update fails with "Invalid signature"

**Solution**:
- Verify public key in `tauri.conf.json` matches private key
- Check that `latest.json` signature matches installer
- Ensure no corruption during download

---

**Issue**: `latest.json` not generated

**Solution**:
- Check that signature files exist
- Verify workflow step ran successfully
- Review workflow logs for errors

## Testing

### Local Testing

1. **Test key generation**:
   ```bash
   bunx tauri signer generate -w test.key
   ```

2. **Test signing**:
   ```bash
   just build # Just will read `.env` for secrets
   ```

3. **Verify signature files**:
   ```bash
   find src-tauri/target/release/bundle -name "*.sig"
   ```

### CI/CD Testing

1. **Test release workflow**:
   ```bash
   git tag v0.0.1-test
   git push origin v0.0.1-test
   ```

2. **Monitor workflow**: Check GitHub Actions tab

3. **Verify artifacts**:
   - Download release assets
   - Confirm `.sig` files present
   - Check `latest.json` format

## Release Script Reference

### Scripts Location
- `scripts/release.ts` - TypeScript (cross-platform, runs with Bun/Node.js)
- `scripts/setup-updater-signing.ts` - TypeScript signing setup

### Usage Examples

**Bump versions**:
```bash
# Patch: 0.2.11 -> 0.2.12
just release-patch
just release --patch

# Minor: 0.2.11 -> 0.3.0
just release-minor
just release --minor

# Major: 0.2.11 -> 1.0.0
just release-major
just release --major
```

**Specify exact version**:
```bash
just release 1.2.3
```

**Skip push (manual push later)**:
```bash
# Using Bun
bun scripts/release.ts 1.2.3 --no-push

# Using Node.js
npx tsx scripts/release.ts 1.2.3 --no-push
```

### Script Features

✅ **Cross-platform**: TypeScript runs on any platform with Bun/Node.js  
✅ **Safe**: Requires confirmation before executing  
✅ **Automatic**: Updates versions, CHANGELOG, commits, tags, and pushes  
✅ **Semantic versioning**: Supports major/minor/patch bumps  
✅ **Flexible**: Can specify exact version or bump type  
✅ **Type-safe**: Written in TypeScript for better reliability  

### RELEASE_NOTE.md Format

The script automatically extracts content from `RELEASE_NOTE.md` after the separator comment:

```markdown
> [!WARNING]
> These admonitions will be kept in RELEASE_NOTE.md

> [!NOTE]
> But not included in CHANGELOG.md

<!-- Release notes content starts here -->

## 🎉 Features
- This content will be extracted

## 🐛 Bug Fixes
- And added to CHANGELOG.md
```

The content is converted from `##` headers to `###` headers for CHANGELOG format, preserving all other formatting.

## Additional Resources

- [Tauri Updater Plugin](https://v2.tauri.app/plugin/updater/)
- [Tauri Signer CLI](https://v2.tauri.app/reference/cli/#signer)
- [GitHub Actions Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [Focust Updater Signing Guide](UPDATER_SIGNING.md)
- [Release Scripts](../scripts/) - Automated release automation

## Support

For issues or questions:
1. Check documentation: [docs/UPDATER_SIGNING.md](docs/UPDATER_SIGNING.md)
2. Review workflow logs: GitHub Actions tab
3. Open an issue with relevant logs (redact sensitive info)
