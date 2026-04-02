/**
 * Default release pipeline stages
 *
 * Each function is an independent, composable stage that receives the
 * pipeline context and can modify its shared fields. Stages can be
 * freely added, removed, or reordered in `scripts/release-hooks.ts`.
 *
 * Available stages:
 *   updatePackageVersion  - Update package.json version
 *   updateTauriConfig     - Update tauri.conf.json version (skips if missing)
 *   updateChangelog       - Extract release notes, update CHANGELOG.md
 *   commit                - Stage files and create commit
 *   tag                   - Create version tag
 *   push                  - Push branch and tag to remote
 *   resetReleaseNote      - Reset RELEASE_NOTE.md to template
 */

import { MARKERS, PATHS, VCS } from "./constants";
import type { ReleasePipelineContext, ReleaseStage } from "./release-hooks";
import {
  anchor,
  confirm,
  execCapture,
  file,
  getDateString,
  logger,
  projectPath,
  updateJsonVersion,
} from "./utils";
import { getManualPushHint } from "./vcs";

// ============================================================================
// Version Stages
// ============================================================================

/**
 * Update package.json version
 */
export function updatePackageVersion(ctx: ReleasePipelineContext): void {
  updateJsonVersion(PATHS.PACKAGE_JSON, ctx.newVersion);
  ctx.modifiedFiles.push(PATHS.PACKAGE_JSON);
  ctx.filesToStage.push(PATHS.PACKAGE_JSON);
}

/**
 * Update tauri.conf.json version (skips if file does not exist)
 */
export function updateTauriConfig(ctx: ReleasePipelineContext): void {
  const configPath = projectPath(PATHS.TAURI_CONFIG);
  if (!file.exists(configPath)) {
    logger.warning(`${PATHS.TAURI_CONFIG} not found, skipping`);
    return;
  }
  updateJsonVersion(PATHS.TAURI_CONFIG, ctx.newVersion);
  ctx.modifiedFiles.push(PATHS.TAURI_CONFIG);
  ctx.filesToStage.push(PATHS.TAURI_CONFIG);
}

// ============================================================================
// Changelog Stage
// ============================================================================

/**
 * Extract release notes from RELEASE_NOTE.md and update CHANGELOG.md
 */
export function updateChangelog(ctx: ReleasePipelineContext): void {
  // Extract release notes
  const releaseNotePath = projectPath(PATHS.RELEASE_NOTE);
  if (!file.exists(releaseNotePath)) {
    throw new Error(`${PATHS.RELEASE_NOTE} not found. Please create it first.`);
  }

  const noteContent = file.read(releaseNotePath);
  if (!anchor.exists(noteContent, MARKERS.RELEASE_NOTE_SEPARATOR)) {
    logger.warning(
      `No separator comment found in ${PATHS.RELEASE_NOTE}, using all content`,
    );
    ctx.releaseNotes = noteContent.trim();
  } else {
    ctx.releaseNotes = anchor.getAfter(
      noteContent,
      MARKERS.RELEASE_NOTE_SEPARATOR,
    );
  }

  // Update changelog
  const date = getDateString();
  const changelogPath = projectPath(PATHS.CHANGELOG);
  const changelogContent = file.read(changelogPath);

  // Convert ## to ### for changelog format
  const changelogEntry = ctx.releaseNotes.replace(/^## /gm, "### ");
  const newEntry = `\n\n## ${ctx.newVersion} (${date})\n\n${changelogEntry}`;
  const updated = anchor.insertAfter(
    changelogContent,
    MARKERS.CHANGELOG_INSERT,
    newEntry,
  );

  file.write(changelogPath, updated);
  logger.success(`Updated ${PATHS.CHANGELOG}`);

  ctx.modifiedFiles.push(PATHS.CHANGELOG, PATHS.RELEASE_NOTE);
  ctx.filesToStage.push(PATHS.CHANGELOG, PATHS.RELEASE_NOTE);
}

// ============================================================================
// VCS Stages
// ============================================================================

/**
 * Stage files and create commit
 */
export function commit(ctx: ReleasePipelineContext): void {
  if (ctx.modifiedFiles.length > 0) {
    ctx.vcs.showDiff(ctx.modifiedFiles);
  }
  if (!confirm("Proceed with commit?")) {
    logger.warning("Commit aborted by user");
    return;
  }
  ctx.vcs.commit(ctx.commitMessage, [...ctx.filesToStage], ctx.stageAll);
  ctx.extra.commitHash = execCapture("git", ["rev-parse", "HEAD"]);
  logger.success("Created commit");
}

/**
 * Create version tag
 */
export function tag(ctx: ReleasePipelineContext): void {
  ctx.vcs.createTag(ctx.tagName);
  logger.success(`Created tag ${ctx.tagName}`);
}

/**
 * Push branch and tag to remote.
 * Skips if --no-push flag is set or user declines confirmation.
 */
export function push(ctx: ReleasePipelineContext): void {
  if (ctx.noPush) {
    logger.warning(
      `Push skipped (--no-push). Run manually:\n${getManualPushHint(ctx.vcsType, ctx.tagName)}`,
    );
    return;
  }

  if (!confirm("Push commit and tag to remote?")) {
    logger.warning(
      `Push skipped. Run manually:\n${getManualPushHint(ctx.vcsType, ctx.tagName)}`,
    );
    return;
  }

  ctx.vcs.pushBranch(VCS.DEFAULT_BRANCH);
  ctx.vcs.pushTag(ctx.tagName);
  ctx.extra.pushed = true;
  logger.success("Pushed changes and tag to remote");
}

// ============================================================================
// Cleanup Stage
// ============================================================================

/**
 * Reset RELEASE_NOTE.md to template after release
 */
export function resetReleaseNote(): void {
  const templatePath = projectPath(".github/RELEASE_NOTE_TEMPLATE.md");
  const targetPath = projectPath(PATHS.RELEASE_NOTE);
  if (!file.exists(templatePath)) {
    logger.warning(
      "Template not found: .github/RELEASE_NOTE_TEMPLATE.md — skipping reset",
    );
    return;
  }
  file.copy(templatePath, targetPath);
  logger.success("Reset RELEASE_NOTE.md to template");
}

// ============================================================================
// Exports
// ============================================================================

/** All default stage functions, importable for composition */
export const defaults = {
  commit,
  push,
  resetReleaseNote,
  tag,
  updateChangelog,
  updatePackageVersion,
  updateTauriConfig,
} as const;

/** Default stage pipeline (used when no user config exists) */
export const DEFAULT_STAGES: ReleaseStage[] = [
  updatePackageVersion,
  updateTauriConfig,
  updateChangelog,
  commit,
  tag,
  push,
  resetReleaseNote,
];
