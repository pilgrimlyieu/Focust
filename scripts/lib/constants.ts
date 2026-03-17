/**
 * Constants for release scripts
 */

// ============================================================================
// File Paths (relative to project root)
// ============================================================================

/**
 * Core project configuration files
 */
export const PATHS = {
  /** Project changelog */
  CHANGELOG: "CHANGELOG.md",
  /** Environment variables file */
  ENV: ".env",
  /** NPM package configuration */
  PACKAGE_JSON: "package.json",
  /** Release notes for current release */
  RELEASE_NOTE: "RELEASE_NOTE.md",
  /** Tauri configuration */
  TAURI_CONFIG: "src-tauri/tauri.conf.json",
  /** Tauri source directory */
  TAURI_DIR: "src-tauri",
} as const;

// ============================================================================
// Release Staging Configuration
// ============================================================================

/**
 * Files to include in version release commits
 */
export const RELEASE_STAGE_FILES = [
  PATHS.PACKAGE_JSON,
  PATHS.TAURI_CONFIG,
  PATHS.CHANGELOG,
  PATHS.RELEASE_NOTE,
] as const;

// ============================================================================
// Markers and Patterns
// ============================================================================

/**
 * Content markers used in files
 */
export const MARKERS = {
  /** Insert point in CHANGELOG.md for new versions */
  CHANGELOG_INSERT: "<!-- CHANGELOG_INSERT -->",
  /** Separator in RELEASE_NOTE.md after which content is extracted */
  RELEASE_NOTE_SEPARATOR: "<!-- Release notes content starts here -->",
} as const;

// ============================================================================
// VCS Configuration
// ============================================================================

/**
 * VCS-related constants (shared by Git and JJ)
 */
export const VCS = {
  /** Commit message template (use with version interpolation) */
  COMMIT_MESSAGE_TEMPLATE: "chore: bump version to v%s",
  /** Default branch/bookmark name */
  DEFAULT_BRANCH: "main",
  /**
   * VCS backend to use for release operations.
   *
   * Priority chain (highest to lowest):
   *   CLI --vcs arg > this setting (if not "auto") > auto-detect via .jj directory
   *
   * Set to "git" or "jj" to pin the backend; "auto" enables auto-detection.
   */
  DEFAULT_VCS: "auto" as "auto" | "git" | "jj",
  /** Tag template (use with version interpolation) */
  TAG_TEMPLATE: "v%s",
} as const;

// ============================================================================
// Signing Key Configuration
// ============================================================================

/**
 * Default signing key filename (project name will be prepended)
 */
export const SIGNING_KEY_EXTENSION = ".key";

/**
 * Default directory for signing keys
 */
export const SIGNING_KEY_DIR = ".tauri";
