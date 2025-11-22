#!/usr/bin/env bun
/**
 * Release automation script
 *
 * Usage:
 *   bun scripts/release.ts 1.2.3      # Specify exact version
 *   bun scripts/release.ts --patch    # Bump patch: 0.2.11 -> 0.2.12
 *   bun scripts/release.ts --minor    # Bump minor: 0.2.11 -> 0.3.0
 *   bun scripts/release.ts --major    # Bump major: 0.2.11 -> 1.0.0
 *   bun scripts/release.ts --no-push  # Skip pushing to remote
 */

import * as path from "node:path";
import { confirm, execOrThrow, exitWithError, file, logger } from "./lib/utils";

// ============================================================================
// Types
// ============================================================================

interface PackageJson {
  version: string;
  [key: string]: unknown;
}

type BumpType = "major" | "minor" | "patch";

interface Args {
  version?: string;
  bumpType?: BumpType;
  noPush?: boolean;
}

// ============================================================================
// Version Management
// ============================================================================

function getCurrentVersion(): string {
  const packageJsonPath = path.join(process.cwd(), "package.json");
  const packageJson = JSON.parse(file.read(packageJsonPath)) as PackageJson;
  return packageJson.version;
}

function calculateNewVersion(current: string, bumpType: BumpType): string {
  const parts = current.split(".").map(Number);
  const [major, minor, patch] = parts;

  if (parts.length !== 3 || parts.some(Number.isNaN)) {
    throw new Error(`Invalid version format: ${current}`);
  }

  switch (bumpType) {
    case "major":
      return `${major + 1}.0.0`;
    case "minor":
      return `${major}.${minor + 1}.0`;
    case "patch":
      return `${major}.${minor}.${patch + 1}`;
  }
}

function isValidVersion(version: string): boolean {
  return /^\d+\.\d+\.\d+$/.test(version);
}

function updateJsonVersion(filePath: string, newVersion: string): void {
  const content = file.read(filePath);
  const updated = content.replace(
    /("version"\s*:\s*")([^"]+)(")/,
    `$1${newVersion}$3`,
  );
  file.write(filePath, updated);
  logger.success(`Updated ${filePath}`);
}

// ============================================================================
// Changelog Management
// ============================================================================

function extractReleaseNotes(): string {
  const releaseNotePath = path.join(process.cwd(), "RELEASE_NOTE.md");

  if (!file.exists(releaseNotePath)) {
    exitWithError("RELEASE_NOTE.md not found. Please create it first.");
  }

  const content = file.read(releaseNotePath);

  // Extract content after separator comment
  const separatorIndex = content.indexOf(
    "<!-- Release notes content starts here -->",
  );

  if (separatorIndex !== -1) {
    const afterSeparator = content.slice(
      separatorIndex + "<!-- Release notes content starts here -->".length,
    );
    return afterSeparator.trim();
  }

  logger.warning(
    "No separator comment found in RELEASE_NOTE.md, using all content",
  );
  return content.trim();
}

function updateChangelog(newVersion: string, releaseNotes: string): void {
  const now = new Date();
  const date = `${now.getFullYear()}.${now.getMonth() + 1}.${now.getDate()}`;

  const changelogPath = path.join(process.cwd(), "CHANGELOG.md");
  const content = file.read(changelogPath);

  // Convert ## to ### for changelog format
  const changelogEntry = releaseNotes.replace(/^## /gm, "### ");

  // Insert new version after [Unreleased]
  const newEntry = `\n\n## ${newVersion} (${date})\n\n${changelogEntry}`;
  const updated = content.replace(/(\[Unreleased\])/, `$1${newEntry}`);

  file.write(changelogPath, updated);
  logger.success("Updated CHANGELOG.md");
}

// ============================================================================
// Git Operations
// ============================================================================

function git(...args: string[]): void {
  execOrThrow("git", args);
}

function commitAndTag(version: string): void {
  git(
    "add",
    "package.json",
    "src-tauri/tauri.conf.json",
    "CHANGELOG.md",
    "RELEASE_NOTE.md",
  );
  logger.info("🔍 Verifying Staged Changes (Version files only):");
  git("diff", "--staged", "-U0", "package.json", "src-tauri/tauri.conf.json");
  confirm("Proceed with commit?") || exitWithError("Commit cancelled by user");
  git("commit", "-m", `"chore: bump version to v${version}"`);
  git("tag", `v${version}`);
  logger.success(`Created commit and tag v${version}`);
}

function pushToRemote(version: string): void {
  git("push", "origin", "main");
  git("push", "origin", "tag", `v${version}`);
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
      default:
        if (!arg.startsWith("--")) {
          result.version = arg;
        }
        break;
    }
  }

  return result;
}

// ============================================================================
// Main Logic
// ============================================================================

async function main(): Promise<void> {
  try {
    logger.banner("🚀 Focust Release Automation");
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
      exitWithError(
        `Usage: bun scripts/release.ts <version|--patch|--minor|--major>`,
      );
    }

    logger.multiline([
      `Current version: ${currentVersion}`,
      `New version:     ${newVersion}`,
    ]);
    logger.spacer();

    // Confirm with user
    if (!confirm(`Continue with release v${newVersion}?`)) {
      logger.warning("Release cancelled");
      process.exit(0);
    }

    logger.spacer();

    // Step 1: Update version numbers
    logger.step(1, "Updating version numbers");
    updateJsonVersion("package.json", newVersion);
    updateJsonVersion("src-tauri/tauri.conf.json", newVersion);
    logger.spacer();

    // Step 2: Update CHANGELOG
    logger.step(2, "Updating CHANGELOG.md");
    const releaseNotes = extractReleaseNotes();
    updateChangelog(newVersion, releaseNotes);
    logger.spacer();

    // Step 3: Commit and tag
    logger.step(3, "Creating commit and tag");
    commitAndTag(newVersion);
    logger.spacer();

    // Step 4: Push to remote
    if (!args.noPush) {
      if (confirm("Push commit and tag to remote?")) {
        logger.step(4, "Pushing to remote");
        pushToRemote(newVersion);
        logger.spacer();
      } else {
        logger.warning(
          `Push skipped. Run manually:\n  git push origin main && git push origin v${newVersion}`,
        );
        logger.spacer();
      }
    } else {
      logger.warning(
        `Push skipped (--no-push). Run manually:\n  git push origin main && git push origin v${newVersion}`,
      );
      logger.spacer();
    }

    // Success!
    logger.success(`Release v${newVersion} completed! 🎉`);
    logger.spacer();
    logger.info(
      "💡 Tip: Sign your commit with GPG if needed:\n   git commit --amend -S --no-edit && git push origin main --force-with-lease",
    );
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
