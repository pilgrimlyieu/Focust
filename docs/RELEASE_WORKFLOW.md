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
- `docs/CODE_SIGNING.md` - Comprehensive guide for setting up code signing
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
- `scripts/setup-signing.sh` - Bash script (Linux/macOS)
- `scripts/setup-signing.ps1` - PowerShell script (Windows)

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
   # Linux/macOS
   bash scripts/setup-signing.sh
   
   # Windows
   powershell scripts/setup-signing.ps1
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

1. **Update version**:
   - Edit `src-tauri/tauri.conf.json`
   - Update `CHANGELOG.md` and `RELEASE_NOTE.md`

2. **Create and push tag**:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

3. **Automatic process**:
   - GitHub Actions builds for all platforms
   - Installers are signed automatically
   - Release is created with:
     - Signed installers
     - Signature files (`.sig`)
     - Update manifest (`latest.json`)
     - Release notes

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
   cargo tauri signer generate -w test.key
   ```

2. **Test signing**:
   ```bash
   export TAURI_SIGNING_PRIVATE_KEY="$(cat test.key)"
   just build
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

## Additional Resources

- [Tauri Updater Plugin](https://v2.tauri.app/plugin/updater/)
- [Tauri Signer CLI](https://v2.tauri.app/reference/cli/#signer)
- [GitHub Actions Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [Focust Code Signing Guide](CODE_SIGNING.md)

## Support

For issues or questions:
1. Check documentation: `docs/CODE_SIGNING.md`
2. Review workflow logs: GitHub Actions tab
3. Open an issue with relevant logs (redact sensitive info)
