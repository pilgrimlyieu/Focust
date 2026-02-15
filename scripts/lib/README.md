# Script Library

Shared utilities for Tauri project automation scripts.

## Modules

### constants.ts

Centralized configuration values:

| Export | Description |
|--------|-------------|
| `PATHS` | File paths (`PACKAGE_JSON`, `TAURI_CONFIG`, `CHANGELOG`, etc.) |
| `RELEASE_STAGE_FILES` | Files staged for release commits |
| `MARKERS` | Content markers (changelog insert point, release note separator) |
| `GIT` | Git settings (branch, commit template, tag template) |
| `SIGNING_KEY_*` | Signing key path defaults |

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
