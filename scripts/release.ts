#!/usr/bin/env bun
/**
 * Release automation script for Tauri projects
 *
 * Usage:
 *   bun scripts/release.ts 1.2.3      # Specify exact version
 *   bun scripts/release.ts --patch    # Bump patch: 0.2.11 -> 0.2.12
 *   bun scripts/release.ts --minor    # Bump minor: 0.2.11 -> 0.3.0
 *   bun scripts/release.ts --major    # Bump major: 0.2.11 -> 1.0.0
 *   bun scripts/release.ts --no-push  # Skip pushing to remote
 *   bun scripts/release.ts --all      # Stage all changes (not just release files)
 */

import { GIT, MARKERS, PATHS, RELEASE_STAGE_FILES } from "./lib/constants";
import {
  anchor,
  type BumpType,
  calculateNewVersion,
  confirm,
  execOrThrow,
  exitWithError,
  file,
  getCurrentVersion,
  getDateString,
  getProjectName,
  isValidVersion,
  logger,
  projectPath,
  renderTemplate,
  updateJsonVersion,
} from "./lib/utils";

// ============================================================================
// Types
// ============================================================================

interface Args {
  version?: string;
  bumpType?: BumpType;
  noPush?: boolean;
  stageAll?: boolean;
}

// ============================================================================
// Changelog Management
// ============================================================================

function extractReleaseNotes(): string {
  const releaseNotePath = projectPath(PATHS.RELEASE_NOTE);

  if (!file.exists(releaseNotePath)) {
    exitWithError(`${PATHS.RELEASE_NOTE} not found. Please create it first.`);
  }

  const content = file.read(releaseNotePath);

  if (!anchor.exists(content, MARKERS.RELEASE_NOTE_SEPARATOR)) {
    logger.warning(
      `No separator comment found in ${PATHS.RELEASE_NOTE}, using all content`,
    );
    return content.trim();
  }

  return anchor.getAfter(content, MARKERS.RELEASE_NOTE_SEPARATOR);
}

function updateChangelog(newVersion: string, releaseNotes: string): void {
  const date = getDateString();
  const changelogPath = projectPath(PATHS.CHANGELOG);
  const content = file.read(changelogPath);

  // Convert ## to ### for changelog format
  const changelogEntry = releaseNotes.replace(/^## /gm, "### ");

  // Insert new version after changelog insert marker
  const newEntry = `\n\n## ${newVersion} (${date})\n\n${changelogEntry}`;
  const updated = anchor.insertAfter(
    content,
    MARKERS.CHANGELOG_INSERT,
    newEntry,
  );

  file.write(changelogPath, updated);
  logger.success(`Updated ${PATHS.CHANGELOG}`);
}

// ============================================================================
// Git Operations
// ============================================================================

function git(...args: string[]): void {
  execOrThrow("git", args);
}

function commitAndTag(version: string, stageAll: boolean): void {
  const tag = renderTemplate(GIT.TAG_TEMPLATE, version);
  const commitMsg = renderTemplate(GIT.COMMIT_MESSAGE_TEMPLATE, version);

  git("diff", "-U0", PATHS.PACKAGE_JSON, PATHS.TAURI_CONFIG);
  confirm("Proceed with commit?") || exitWithError("Commit cancelled by user");
  if (stageAll) {
    logger.info("🔍 Staging all changes");
    git("add", "-A");
  } else {
    logger.info("🔍 Staging specific changes");
    git("add", ...RELEASE_STAGE_FILES);
  }

  git("commit", "-m", `"${commitMsg}"`);
  git("tag", tag);
  logger.success(`Created commit and tag ${tag}`);
}

function pushToRemote(version: string): void {
  const tag = renderTemplate(GIT.TAG_TEMPLATE, version);
  git("push", "origin", GIT.DEFAULT_BRANCH);
  git("push", "origin", "tag", tag);
  logger.success("Pushed changes and tag to remote");
}

// ============================================================================
// Argument Parsing
// ============================================================================

function parseArgs(): Args {
  const args = process.argv.slice(2);
  const result: Args = {};

  for (const arg of args) {
    switch (arg) {
      case "--patch":
        result.bumpType = "patch";
        break;
      case "--minor":
        result.bumpType = "minor";
        break;
      case "--major":
        result.bumpType = "major";
        break;
      case "--no-push":
        result.noPush = true;
        break;
      case "--all":
      case "-a":
        result.stageAll = true;
        break;
      default:
        if (!arg.startsWith("--") && !arg.startsWith("-")) {
          result.version = arg;
        }
        break;
    }
  }

  return result;
}

function printUsage(): void {
  logger.multiline([
    "Usage: bun scripts/release.ts <version|--patch|--minor|--major> [options]",
    "",
    "Version:",
    "  <X.Y.Z>      Specify exact version",
    "  --patch      Bump patch version (0.2.11 -> 0.2.12)",
    "  --minor      Bump minor version (0.2.11 -> 0.3.0)",
    "  --major      Bump major version (0.2.11 -> 1.0.0)",
    "",
    "Options:",
    "  --no-push    Skip pushing to remote",
    "  --all, -a    Stage all changes (not just release files)",
  ]);
}

// ============================================================================
// Main Logic
// ============================================================================

async function main(): Promise<void> {
  try {
    const projectName = getProjectName();
    logger.banner(`🚀 Release Automation - ${projectName}`);
    logger.spacer();

    // Parse arguments and determine version
    const args = parseArgs();
    const currentVersion = getCurrentVersion();
    let newVersion = "";

    if (args.version) {
      if (!isValidVersion(args.version)) {
        exitWithError(
          `Invalid version format: ${args.version}. Expected: X.Y.Z`,
        );
      }
      newVersion = args.version;
    } else if (args.bumpType) {
      newVersion = calculateNewVersion(currentVersion, args.bumpType);
    } else {
      printUsage();
      process.exit(1);
    }

    logger.multiline([
      `Current version: ${currentVersion}`,
      `New version:     ${newVersion}`,
      args.stageAll
        ? "Stage mode:      All changes"
        : "Stage mode:      Release files only",
    ]);
    logger.spacer();

    if (!confirm(`Continue with release v${newVersion}?`)) {
      // Confirm with user
      logger.warning("Release cancelled");
      process.exit(0);
    }

    logger.spacer();

    // Step 1: Update version numbers
    logger.step(1, "Updating version numbers");
    updateJsonVersion(PATHS.PACKAGE_JSON, newVersion);
    updateJsonVersion(PATHS.TAURI_CONFIG, newVersion);
    logger.spacer();

    // Step 2: Update CHANGELOG
    logger.step(2, `Updating ${PATHS.CHANGELOG}`);
    const releaseNotes = extractReleaseNotes();
    updateChangelog(newVersion, releaseNotes);
    logger.spacer();

    // Step 3: Commit and tag
    logger.step(3, "Creating commit and tag");
    commitAndTag(newVersion, args.stageAll ?? false);
    logger.spacer();

    // Step 4: Push to remote
    const tag = renderTemplate(GIT.TAG_TEMPLATE, newVersion);
    if (!args.noPush) {
      if (confirm("Push commit and tag to remote?")) {
        logger.step(4, "Pushing to remote");
        pushToRemote(newVersion);
        logger.spacer();
      } else {
        logger.warning(
          `Push skipped. Run manually:\n  git push origin ${GIT.DEFAULT_BRANCH} && git push origin tag ${tag}`,
        );
        logger.spacer();
      }
    } else {
      logger.warning(
        `Push skipped (--no-push). Run manually:\n  git push origin ${GIT.DEFAULT_BRANCH} && git push origin tag ${tag}`,
      );
      logger.spacer();
    }

    // Success!
    logger.success(`Release v${newVersion} completed! 🎉`);
  } catch (error) {
    logger.spacer();
    exitWithError(
      `Release failed: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

// ============================================================================
// Entry Point
// ============================================================================

main();
