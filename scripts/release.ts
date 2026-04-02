#!/usr/bin/env bun
/**
 * Release automation script
 *
 * Executes a configurable pipeline of release stages.
 * Stages are loaded from `scripts/release-hooks.ts` (user config)
 * or fall back to `DEFAULT_STAGES` from `scripts/lib/default-release-hooks.ts`.
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

import {
  createPipelineContext,
  loadReleaseStages,
  type ReleasePipelineContext,
} from "./lib/release-hooks";
import {
  type BumpType,
  calculateNewVersion,
  confirm,
  exitWithError,
  getCurrentVersion,
  getErrorMessage,
  getProjectName,
  isValidVersion,
  logger,
} from "./lib/utils";
import { createVcsDriver, type VcsType } from "./lib/vcs";

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
// Pipeline Setup
// ============================================================================

/**
 * Resolve new version from CLI arguments
 */
function resolveNewVersion(args: Args, currentVersion: string): string {
  if (args.version) {
    if (!isValidVersion(args.version)) {
      exitWithError(`Invalid version format: ${args.version}. Expected: X.Y.Z`);
    }
    return args.version;
  }

  if (args.bumpType) {
    return calculateNewVersion(currentVersion, args.bumpType);
  }

  printUsage();
  process.exit(1);
}

/**
 * Initialize the pipeline: parse args, load stages, build context
 */
async function initialize(): Promise<ReleasePipelineContext> {
  const projectName = getProjectName();
  logger.banner(`🚀 Release Automation - ${projectName}`);
  logger.spacer();

  const args = parseArgs();
  const currentVersion = getCurrentVersion();
  const newVersion = resolveNewVersion(args, currentVersion);
  const vcs = createVcsDriver(args.vcsType);
  const stages = await loadReleaseStages();

  return createPipelineContext(currentVersion, newVersion, args, vcs, stages);
}

/**
 * Display release info and ask for confirmation
 */
function confirmRelease(ctx: ReleasePipelineContext): void {
  logger.multiline([
    `Current version: ${ctx.currentVersion}`,
    `New version:     ${ctx.newVersion}`,
    ctx.stageAll
      ? "Stage mode:      All changes"
      : "Stage mode:      Release files only",
  ]);
  logger.spacer();

  if (!confirm(`Continue with release v${ctx.newVersion}?`)) {
    logger.warning("Release cancelled");
    process.exit(0);
  }

  logger.spacer();
}

// ============================================================================
// Main
// ============================================================================

async function main(): Promise<void> {
  try {
    const ctx = await initialize();
    confirmRelease(ctx);

    for (const [index, stage] of ctx.stages.entries()) {
      const name = stage.stageName ?? stage.name;
      logger.step(index + 1, name);
      const result = await stage(ctx);
      if (result === false) {
        exitWithError(`Release aborted by stage: ${name}`);
      }
      logger.spacer();
    }

    logger.success(`Release v${ctx.newVersion} completed! 🎉`);
  } catch (error) {
    logger.spacer();
    exitWithError(`Release failed: ${getErrorMessage(error)}`);
  }
}

// ============================================================================
// Entry Point
// ============================================================================

main();
