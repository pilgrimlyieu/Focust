# CI Scripts

Reusable TypeScript scripts for GitHub Actions workflows. These scripts extract complex inline workflow logic into testable, maintainable modules.

## Design Principles

- **Project-agnostic**: All names read from `package.json` / `tauri.conf.json` at runtime — no hardcoded project names
- **No `node_modules` required**: Scripts import only `node:*` builtins, so they run with `bun` alone (no `bun install`)
- **Shared utilities**: Common helpers live in `scripts/lib/utils.ts` (`logger`, `parseCliArgs`, `execCapture`, etc.)
- **Consistent logging**: All output goes through `logger` — no bare `console.log`

## Scripts

### `prepare-artifacts.ts`

Collects Tauri build outputs (installers, portable archives, updater signatures) into a standardized `artifacts/` directory.

```bash
bun scripts/ci/prepare-artifacts.ts --platform windows --version 0.3.4 --target x86_64-pc-windows-msvc
bun scripts/ci/prepare-artifacts.ts --platform linux   --version 0.3.4 --target x86_64-unknown-linux-gnu
bun scripts/ci/prepare-artifacts.ts --platform macos   --version 0.3.4 --target universal-apple-darwin
```

| Option | Required | Default | Description |
|--------|----------|---------|-------------|
| `--platform` | Yes | — | `windows`, `linux`, or `macos` |
| `--version` | Yes | — | Version string (e.g. `0.3.4`) |
| `--target` | Yes | — | Rust target triple |
| `--output` | No | `artifacts` | Output directory |

### `generate-manifest.ts`

Generates `latest.json` for Tauri's built-in updater plugin by scanning artifacts.

```bash
bun scripts/ci/generate-manifest.ts --version 0.3.4 --tag v0.3.4 --repo owner/repo
```

| Option | Required | Default | Description |
|--------|----------|---------|-------------|
| `--version` | Yes | — | Version string |
| `--tag` | Yes | — | Git tag with `v` prefix |
| `--repo` | Yes | — | GitHub repository (`owner/repo`) |
| `--artifacts-dir` | No | `artifacts` | Directory containing build artifacts |
| `--output` | No | `artifacts/latest.json` | Output file path |

### `prepare-debug-artifacts.ts`

Archives release binaries, symbols, frontend maps and final packages. CI sets `CARGO_PROFILE_RELEASE_DEBUG=1`, `CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=packed`, and `CARGO_PROFILE_RELEASE_STRIP=none` for builds and cache warming; local Cargo profiles are unchanged.

Run its stages in this order:

1. Build the frontend with `FOCUST_DEBUG_ARTIFACTS=true`, then run `--stage frontend`. This saves assets and hidden source maps in `.debug-build/frontend`, then removes maps from `dist` before Tauri embeds it. Start without an existing `.debug-build` directory.
2. Run `tauri build --no-bundle` with `build.beforeBuildCommand` set to an empty string, then run `--stage native --platform <platform> --target <target>`. This saves binaries and symbols before bundling can modify them. Missing or empty symbols fail the build.
3. On Linux, run `strip --strip-debug` on the distribution ELF after staging: split DWARF still leaves debug sections in the executable.
4. Run `tauri bundle` and `prepare-artifacts.ts`, then `--stage archive --platform <platform> --target <target> --version <version>` to include final packages and write build metadata and SHA-256 checksums.

Output: `debug-artifacts/*_debug.tar.gz`, preserving Unix permissions and dSYM contents. See [Downloading Release Debugging Artifacts](../../docs/RELEASE_WORKFLOW.md#downloading-release-debugging-artifacts) for contents and usage.

### `generate-release-notes.ts`

Assembles a GitHub Release body from `RELEASE_NOTE.md` + git commit log.

```bash
bun scripts/ci/generate-release-notes.ts --version 0.3.4 --tag v0.3.4 --repo owner/repo
```

| Option | Required | Default | Description |
|--------|----------|---------|-------------|
| `--version` | Yes | — | Version string |
| `--tag` | Yes | — | Current git tag |
| `--prev-tag` | No | auto-detected | Previous git tag |
| `--repo` | Yes | — | GitHub repository |
| `--output` | No | `release_body.md` | Output file path |

## Shared Module: `constants.ts`

Exports platform mappings, bundle paths, and project config utilities:

- `getProjectConfig()` — reads name/productName/resources from config files (cached)
- `artifactName(version, platform, suffix)` — builds consistent artifact filenames
- `getPortableResourceDirs()` — resolves `bundle.resources` globs to directories for portable packaging
- `UPDATER_PLATFORMS` — maps platform names to Tauri updater identifiers
- `BUNDLE_PATHS` — relative paths to Tauri bundle outputs

## Adding New Scripts

1. Create `scripts/ci/your-script.ts` with `#!/usr/bin/env bun` shebang
2. Use `parseCliArgs()` + `requireArg()` for option parsing
3. Use `logger` for all output
4. Read project names from `getProjectConfig()` instead of hardcoding
5. Add `--help` support via `parseCliArgs` options
