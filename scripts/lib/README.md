# Script Library

Shared utilities for Tauri project automation scripts.

## Modules

### constants.ts

Centralized configuration values:

| Export                | Description                                                                                                       |
| --------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `PATHS`               | File paths (`PACKAGE_JSON`, `TAURI_CONFIG`, `CHANGELOG`, etc.)                                                    |
| `RELEASE_STAGE_FILES` | Files staged for release commits                                                                                  |
| `MARKERS`             | Content markers (changelog insert point, release note separator)                                                  |
| `VCS`                 | VCS settings: `DEFAULT_VCS` (manual config / "auto"), `DEFAULT_BRANCH`, `COMMIT_MESSAGE_TEMPLATE`, `TAG_TEMPLATE` |
| `SIGNING_KEY_*`       | Signing key path defaults                                                                                         |

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
