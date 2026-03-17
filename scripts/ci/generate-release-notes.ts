#!/usr/bin/env bun

/**
 * Generate release notes body for GitHub Release
 *
 * Reads RELEASE_NOTE.md, generates commit log between tags, and assembles
 * a complete release body markdown file.
 *
 * Usage:
 *   bun scripts/ci/generate-release-notes.ts --version 0.3.4 --tag v0.3.4 --repo owner/repo
 *   bun scripts/ci/generate-release-notes.ts --version 0.3.4 --tag v0.3.4 --prev-tag v0.3.3 --repo owner/repo --output release_body.md
 *
 * Options:
 *   --version   <X.Y.Z>      Version string without 'v' prefix (required)
 *   --tag       <vX.Y.Z>     Current git tag (required)
 *   --prev-tag  <vX.Y.Z>     Previous git tag (auto-detected if omitted)
 *   --repo      <owner/repo> GitHub repository (required)
 *   --output    <path>       Output file path (default: release_body.md)
 */

import { PATHS } from "../lib/constants";
import {
  execCapture,
  file,
  logger,
  parseCliArgs,
  projectPath,
  requireArg,
} from "../lib/utils";

// ============================================================================
// Types
// ============================================================================

interface Args {
  version: string;
  tag: string;
  prevTag: string;
  repo: string;
  output: string;
}

// ============================================================================
// Argument Parsing
// ============================================================================

const HELP_TEXT = `Usage: bun scripts/ci/generate-release-notes.ts [options]

Options:
  --version   <X.Y.Z>      Version without 'v' prefix (required)
  --tag       <vX.Y.Z>     Current git tag (required)
  --prev-tag  <vX.Y.Z>     Previous git tag (auto-detected if omitted)
  --repo      <owner/repo> GitHub repository (required)
  --output    <path>       Output path (default: release_body.md)`;

function parseArgs(): Args {
  const raw = parseCliArgs(process.argv, { helpText: HELP_TEXT });

  const version = requireArg(raw, "version");
  const tag = requireArg(raw, "tag");
  const repo = requireArg(raw, "repo");
  const output =
    typeof raw.output === "string" ? raw.output : "release_body.md";

  // Auto-detect previous tag if not provided
  let prevTag = typeof raw["prev-tag"] === "string" ? raw["prev-tag"] : "";
  if (!prevTag) {
    prevTag = detectPreviousTag(tag);
  }

  return { output, prevTag, repo, tag, version };
}

// ============================================================================
// Git Helpers
// ============================================================================

function detectPreviousTag(currentTag: string): string {
  const previousTag = execCapture("git", [
    "describe",
    "--abbrev=0",
    "--tags",
    `${currentTag}^`,
  ]);
  if (!previousTag) {
    logger.warning(`No previous tag found before ${currentTag}.`);
  }
  return previousTag;
}

function generateCommitLog(prevTag: string, currentTag: string): string {
  if (prevTag) {
    return execCapture(
      "git",
      ["log", '--pretty=format:"- %s (%h)"', `${prevTag}..${currentTag}`],
      { warnOnError: true },
    );
  }
  // No previous tag: initial release — cap at 50 commits as a safety net
  return execCapture(
    "git",
    ["log", '--pretty=format:"- %s (%h)"', "-n", "50"],
    {
      warnOnError: true,
    },
  );
}

// ============================================================================
// Release Notes Assembly
// ============================================================================

function readReleaseNote(): string {
  const releaseNotePath = projectPath(PATHS.RELEASE_NOTE);
  if (file.exists(releaseNotePath)) {
    return file.read(releaseNotePath);
  }
  return "";
}

function assembleReleaseBody(
  version: string,
  tag: string,
  prevTag: string,
  repo: string,
): string {
  const lines: string[] = [];

  // Header
  lines.push(`# Version ${version}`);
  lines.push("");

  // Release notes content
  const releaseNote = readReleaseNote();
  if (releaseNote) {
    lines.push(releaseNote.trim());
    lines.push("");
    lines.push("---");
    lines.push("");
  } else {
    lines.push("---");
    lines.push("");
  }

  // Full changelog link
  if (prevTag) {
    lines.push(
      `**Full Changelog**: https://github.com/${repo}/compare/${prevTag}...${tag}`,
    );
    lines.push("");
    lines.push("---");
    lines.push("");
  }

  // Commit log
  const changesSince = prevTag || "Initial Release";
  lines.push(`## Changes Since ${changesSince}`);
  lines.push("");

  const commitLog = generateCommitLog(prevTag, tag);
  lines.push(commitLog || "_No commits found._");

  return lines.join("\n");
}

// ============================================================================
// Main
// ============================================================================

function main(): void {
  const { version, tag, prevTag, repo, output } = parseArgs();

  logger.banner("Generate Release Notes", `Version: v${version}`);

  if (prevTag) {
    logger.info(`Previous tag: ${prevTag}`);
  } else {
    logger.info("No previous tag found (initial release)");
  }

  const body = assembleReleaseBody(version, tag, prevTag, repo);
  const outputPath = projectPath(output);
  file.write(outputPath, body);

  logger.spacer();
  logger.section("Release body preview:");
  logger.log(body);
  logger.spacer();
  logger.success(`Release notes written to ${output}`);
}

main();
