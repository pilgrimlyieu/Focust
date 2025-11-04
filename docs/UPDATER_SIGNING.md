# Updater Signing Guide

This document explains how to set up code signing for Focust releases.

## Overview

Code signing ensures that:
- Users can verify the authenticity of the application
- Auto-updates are secure and tamper-proof
- The application is trusted by operating systems

## Prerequisites

- Access to the GitHub repository settings
- Rust and Tauri CLI installed locally

## Step 1: Generate Signing Keys

Generate a key pair using Tauri's built-in signer:

```bash
cd src-tauri
bunx tauri signer generate -w ~/.tauri/focust.key
```

This will output:
- **Private Key**: Saved to `~/.tauri/focust.key` (keep this secret!)
- **Public Key**: Printed to console (add to config)
- **Key Password**: Printed to console (if password-protected)

**Important:** 
- Never commit the private key to version control
- Store it securely (password manager, secure vault, etc.)

## Step 2: Update tauri.conf.json

Add the public key to `src-tauri/tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/pilgrimlyieu/Focust/releases/latest/download/latest.json"
      ],
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

Replace `YOUR_PUBLIC_KEY_HERE` with the public key from Step 1.

Commit and push this change:

```bash
git add src-tauri/tauri.conf.json
git commit -m "chore: add updater public key"
git push
```

## Step 3: Configure GitHub Secrets

Add the private key to GitHub repository secrets:

1. Go to your repository on GitHub
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**

### Required Secrets

#### TAURI_SIGNING_PRIVATE_KEY

The private key content (base64 encoded or raw).

To get the key content:

```bash
# On Linux/macOS
cat ~/.tauri/focust.key

# On Windows (PowerShell)
Get-Content ~\.tauri\focust.key -Raw
```

- **Name**: `TAURI_SIGNING_PRIVATE_KEY`
- **Value**: Paste the entire key content

#### TAURI_SIGNING_PRIVATE_KEY_PASSWORD (Optional)

If your key is password-protected:

- **Name**: `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- **Value**: The password for your private key

## Step 4: Verify Setup

### Local Testing

Test signing locally:

```bash
# Build with signing, just will read `.env` for secrets
just build
```

Check that `.sig` files are generated alongside your installers.

### CI/CD Testing

1. Create a test tag to trigger a release:

```bash
git tag v0.1.0-test
git push origin v0.1.0-test
```

2. Monitor the GitHub Actions workflow
3. Verify that:
   - Build completes successfully
   - `.sig` files are included in artifacts
   - `latest.json` is generated with signatures

## Step 5: Distribution

### Release Workflow

When you're ready to release:

1. Update version in `src-tauri/tauri.conf.json`
2. Update `CHANGELOG.md` and `RELEASE_NOTE.md`
3. Create and push a version tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

4. GitHub Actions will automatically:
   - Build signed binaries for all platforms
   - Generate signature files (`.sig`)
   - Create `latest.json` manifest
   - Create a GitHub Release with all artifacts

### Auto-Update Mechanism

The auto-updater works as follows:

1. App checks `latest.json` on startup (or when user clicks "Check for Updates")
2. If a new version is found:
   - Downloads the installer
   - Verifies the signature using the public key
   - Prompts user to install update
3. If signature verification fails, update is rejected

## Security Best Practices

### Key Management

- ✅ **DO**: Store private key in GitHub Secrets
- ✅ **DO**: Use a strong password for the key
- ✅ **DO**: Back up the key securely (encrypted vault)
- ❌ **DON'T**: Commit private key to repository
- ❌ **DON'T**: Share private key via insecure channels
- ❌ **DON'T**: Use the same key for multiple projects

### Key Rotation

If you need to rotate keys:

1. Generate a new key pair
2. Update public key in `tauri.conf.json`
3. Update `TAURI_SIGNING_PRIVATE_KEY` in GitHub Secrets
4. Release a new version
5. Note: Users on old versions will need to manually update once

### Revocation

If a key is compromised:

1. Immediately remove the secret from GitHub
2. Generate a new key pair
3. Update configuration and re-release
4. Consider notifying users through other channels

## Troubleshooting

### Build fails with "Invalid signature"

- Verify the private key is correctly set in GitHub Secrets
- Check that there are no extra spaces or line breaks
- Ensure the key matches the public key in config

### Auto-update not working

- Verify public key in `tauri.conf.json` is correct
- Check that `latest.json` is accessible at the configured URL
- Ensure signature in `latest.json` matches the installer
- Check browser console for error messages

### Signature file not generated

- Verify `TAURI_SIGNING_PRIVATE_KEY` is set
- Check build logs for signing errors
- Ensure Tauri updater plugin is properly configured

## Additional Resources

- [Tauri Updater Documentation](https://v2.tauri.app/plugin/updater/)
- [GitHub Actions Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)

## Support

If you encounter issues:

1. Check the [GitHub Actions logs](https://github.com/pilgrimlyieu/Focust/actions)
2. Review the [troubleshooting section](#troubleshooting)
3. Open an issue with:
   - Build logs (redact any sensitive information)
   - Steps to reproduce
   - Expected vs actual behavior
