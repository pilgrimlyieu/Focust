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
// Git Staging Configuration
// ============================================================================

/**
 * Files to stage for version release commits
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
// Git Configuration
// ============================================================================

/**
 * Git-related constants
 */
export const GIT = {
  /** Commit message template (use with version interpolation) */
  COMMIT_MESSAGE_TEMPLATE: "chore: bump version to v%s",
  /** Default branch name */
  DEFAULT_BRANCH: "main",
  /** Tag prefix */
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
