#!/usr/bin/env bun
/**
 * Release automation script for Tauri projects
 *
 * Supports both Git and Jujutsu (JJ) as version control backends.
 * Auto-detects VCS type (checks for .jj directory), or use --vcs to force.
 *
 * Usage:
 *   bun scripts/release.ts 1.2.3      # Specify exact version
 *   bun scripts/release.ts --patch    # Bump patch: 0.2.1 -> 0.2.2
 *   bun scripts/release.ts --minor    # Bump minor: 0.2.1 -> 0.3.0
 *   bun scripts/release.ts --major    # Bump major: 0.2.1 -> 1.0.0
 *   bun scripts/release.ts --no-push  # Skip pushing to remote
 *   bun scripts/release.ts --all      # Stage all changes (not just release files)
 *   bun scripts/release.ts --vcs jj   # Force Jujutsu mode (overrides config)
 *   bun scripts/release.ts --vcs git  # Force Git mode (overrides config)
 *
 * VCS resolution: --vcs arg > VCS.DEFAULT_VCS in constants.ts > auto-detect (.jj dir)
 */

import { MARKERS, PATHS, RELEASE_STAGE_FILES, VCS } from "./lib/constants";
import {
  createReleaseHookContext,
  loadReleaseHooks,
  runReleaseHook,
} from "./lib/release-hooks";
import {
  anchor,
  type BumpType,
  calculateNewVersion,
  confirm,
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
import {
  createVcsDriver,
  getManualPushHint,
  type VcsDriver,
  type VcsType,
} from "./lib/vcs";

// ============================================================================
// Types
// ============================================================================

interface Args {
  version?: string;
  bumpType?: BumpType;
  noPush?: boolean;
  stageAll?: boolean;
  vcsType?: VcsType;
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
// VCS Operations
// ============================================================================

function commitAndTag(
  vcs: VcsDriver,
  version: string,
  stageAll: boolean,
): void {
  const tag = renderTemplate(VCS.TAG_TEMPLATE, version);
  const commitMsg = renderTemplate(VCS.COMMIT_MESSAGE_TEMPLATE, version);

  vcs.showDiff([PATHS.PACKAGE_JSON, PATHS.TAURI_CONFIG]);
  confirm("Proceed with commit?") || exitWithError("Commit cancelled by user");

  vcs.commit(commitMsg, [...RELEASE_STAGE_FILES], stageAll);
  vcs.createTag(tag);
  logger.success(`Created commit and tag ${tag}`);
}

function pushToRemote(vcs: VcsDriver, version: string): void {
  const tag = renderTemplate(VCS.TAG_TEMPLATE, version);
  vcs.pushBranch(VCS.DEFAULT_BRANCH);
  vcs.pushTag(tag);
  logger.success("Pushed changes and tag to remote");
}

// ============================================================================
// Argument Parsing
// ============================================================================

function parseArgs(): Args {
  const args = process.argv.slice(2);
  const result: Args = {};

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
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
      case "--vcs": {
        const next = args[i + 1];
        if (next === "git" || next === "jj") {
          result.vcsType = next;
          i++;
        } else {
          exitWithError(`Invalid --vcs value: ${next}. Expected: git | jj`);
        }
        break;
      }
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
    "  --patch      Bump patch version (0.2.1 -> 0.2.2)",
    "  --minor      Bump minor version (0.2.1 -> 0.3.0)",
    "  --major      Bump major version (0.2.1 -> 1.0.0)",
    "",
    "Options:",
    "  --no-push    Skip pushing to remote",
    "  --all, -a    Stage all changes (not just release files)",
    "  --vcs <type> Force VCS backend: git | jj (overrides VCS.DEFAULT_VCS and auto-detect)",
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

    // Load hooks
    await loadReleaseHooks();

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

    // Initialize VCS driver: CLI arg > VCS.DEFAULT_VCS > auto-detect
    const vcs = createVcsDriver(args.vcsType);

    // Create hook context
    const hookCtx = createReleaseHookContext(currentVersion, newVersion, {
      noPush: args.noPush,
      stageAll: args.stageAll,
    });

    logger.multiline([
      `Current version: ${currentVersion}`,
      `New version:     ${newVersion}`,
      args.stageAll
        ? "Stage mode:      All changes"
        : "Stage mode:      Release files only",
    ]);
    logger.spacer();

    if (!confirm(`Continue with release v${newVersion}?`)) {
      logger.warning("Release cancelled");
      process.exit(0);
    }

    logger.spacer();

    // Hook: preRelease
    if (!(await runReleaseHook("preRelease", hookCtx))) {
      process.exit(1);
    }

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

    // Hook: preCommit
    if (!(await runReleaseHook("preCommit", hookCtx))) {
      process.exit(1);
    }

    // Step 3: Commit and tag
    logger.step(3, "Creating commit and tag");
    commitAndTag(vcs, newVersion, args.stageAll ?? false);
    logger.spacer();

    // Hook: postCommit
    await runReleaseHook("postCommit", hookCtx);

    // Step 4: Push to remote
    const tag = renderTemplate(VCS.TAG_TEMPLATE, newVersion);
    if (!args.noPush) {
      if (confirm("Push commit and tag to remote?")) {
        logger.step(4, "Pushing to remote");
        pushToRemote(vcs, newVersion);
        logger.spacer();

        // Hook: postPush
        await runReleaseHook("postPush", hookCtx);
      } else {
        logger.warning(
          `Push skipped. Run manually:\n${getManualPushHint(vcs.type, tag)}`,
        );
        logger.spacer();
      }
    } else {
      logger.warning(
        `Push skipped (--no-push). Run manually:\n${getManualPushHint(vcs.type, tag)}`,
      );
      logger.spacer();
    }

    // Hook: postRelease
    await runReleaseHook("postRelease", hookCtx);

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
