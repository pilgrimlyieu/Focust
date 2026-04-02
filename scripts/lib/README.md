# Script Library

Shared utilities for Tauri project automation scripts.

## Modules

### constants.ts

Centralized configuration values:

| Export                | Description                                                                                                       |
| --------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `PATHS`               | File paths (`PACKAGE_JSON`, `TAURI_CONFIG`, `CHANGELOG`, etc.)                                                    |
| `MARKERS`             | Content markers (changelog insert point, release note separator)                                                  |
| `VCS`                 | VCS settings: `DEFAULT_VCS` (manual config / "auto"), `DEFAULT_BRANCH`, `COMMIT_MESSAGE_TEMPLATE`, `TAG_TEMPLATE` |
| `SIGNING_KEY_*`       | Signing key path defaults                                                                                         |

### release-hooks.ts

Release pipeline types and stage loader:

| Export                   | Description                                                                    |
| ------------------------ | ------------------------------------------------------------------------------ |
| `ReleasePipelineContext` | Context flowing through all stages (readonly inputs + shared data + extra bag) |
| `ReleaseStage`           | Stage function signature `(ctx) => void \| boolean \| Promise<...>`            |
| `loadReleaseStages`      | Load stage list from `scripts/release-hooks.ts`, fallback to `DEFAULT_STAGES`  |
| `createPipelineContext`  | Create initial context with version, flags, VCS driver, and stages             |

**Pipeline Context** fields:

| Category        | Fields                                                                                                       |
| --------------- | ------------------------------------------------------------------------------------------------------------ |
| Readonly inputs | `currentVersion`, `newVersion`, `noPush`, `stageAll`, `vcs`, `vcsType`, `tagName`, `commitMessage`, `stages` |
| Shared data     | `modifiedFiles`, `filesToStage`, `releaseNotes`                                                              |
| Custom          | `extra` — generic `Record<string, unknown>` for stage-specific data                                          |

### default-release-hooks.ts

Default release pipeline stages:

| Stage                  | Description                                          |
| ---------------------- | ---------------------------------------------------- |
| `updatePackageVersion` | Update `package.json` version                        |
| `updateTauriConfig`    | Update `tauri.conf.json` version (skips if missing)  |
| `updateChangelog`      | Extract release notes, update `CHANGELOG.md`         |
| `commit`               | Stage files and create commit                        |
| `tag`                  | Create version tag                                   |
| `push`                 | Push branch and tag to remote (respects `--no-push`) |
| `resetReleaseNote`     | Reset `RELEASE_NOTE.md` to template                  |

| Export           | Description                                        |
| ---------------- | -------------------------------------------------- |
| `defaults`       | Object with all stages, importable for composition |
| `DEFAULT_STAGES` | Ordered array of all default stages                |

### vcs.ts

VCS abstraction layer (Git + Jujutsu):

| Export              | Description                                                                     |
| ------------------- | ------------------------------------------------------------------------------- |
| `VcsType`           | VCS type: `"git"` \| `"jj"`                                                     |
| `VcsDriver`         | Interface for VCS operations (showDiff, commit, createTag, pushBranch, pushTag) |
| `gitDriver`         | Git driver implementation                                                       |
| `jjDriver`          | Jujutsu driver implementation (colocated mode)                                  |
| `detectVcsType`     | Auto-detect VCS type (checks for `.jj` directory)                               |
| `resolveVcsType`    | Resolve effective VCS type using priority chain (CLI > config > auto-detect)    |
| `createVcsDriver`   | Factory: create driver using `resolveVcsType`                                   |
| `getManualPushHint` | Generate manual push command hint for skipped pushes                            |

### utils.ts

Common utilities:

| Category    | Exports                                                                            |
| ----------- | ---------------------------------------------------------------------------------- |
| **Logging** | `logger` (banner, info, error, success, warning, step, multiline, spacer, section) |
| **Input**   | `prompt`, `confirm`                                                                |
| **Shell**   | `exec`, `execOrThrow`, `exitWithError`, `exitWithSuccess`                          |
| **Files**   | `file` (read, write, exists, copy, mkdir)                                          |
| **Project** | `getProjectInfo`, `getProjectName`, `getCurrentVersion`, `getTauriConfig`          |
| **Version** | `calculateNewVersion`, `isValidVersion`, `updateJsonVersion`                       |
| **Date**    | `getDateString`                                                                    |
| **Chore**   | `renderTemplate`, `projectPath`                                                    |
| **Anchor**  | `anchor` (insertAfter, insertBefore, replace, getAfter, getBetween, exists)        |

#### Anchor Operations

The `anchor` utility handles text markers for content manipulation:

| Method         | Type    | Description                                        |
| -------------- | ------- | -------------------------------------------------- |
| `insertAfter`  | Insert  | Add content after anchor (anchor preserved)        |
| `insertBefore` | Insert  | Add content before anchor (anchor preserved)       |
| `replace`      | Replace | Replace anchor with content (anchor removed)       |
| `getAfter`     | Marker  | Extract content after anchor (optional end anchor) |
| `getBetween`   | Marker  | Extract content between two anchors                |
| `exists`       | Query   | Check if anchor exists in content                  |

**Anchor Types:**
- **Insert**: Content added relative to anchor, anchor remains in place
- **Replace**: Anchor is replaced by new content, anchor disappears
- **Marker**: Defines content boundaries for extraction
