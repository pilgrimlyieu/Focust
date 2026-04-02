/**
 * Release pipeline types and stage loader
 *
 * Defines the pipeline context and stage types for the release process.
 * Stages are loaded from `scripts/release-hooks.ts` (user config).
 *
 * Usage:
 *   1. Define stages in `scripts/release-hooks.ts`
 *   2. Export a `stages` array of `ReleaseStage` functions
 *   3. Stages are automatically loaded and executed in order during release
 */

import { VCS } from "./constants";
import {
  file,
  getErrorMessage,
  logger,
  projectPath,
  renderTemplate,
} from "./utils";
import type { VcsDriver, VcsType } from "./vcs";

// ============================================================================
// Types
// ============================================================================

/**
 * Pipeline context that flows through all release stages.
 *
 * Readonly fields are set at pipeline start and should not be modified.
 * Shared fields accumulate data across stages.
 * Use `extra` for stage-specific data that doesn't belong in the core interface.
 */
export interface ReleasePipelineContext {
  /** Current version before release */
  readonly currentVersion: string;
  /** New version being released */
  readonly newVersion: string;
  /** Whether --no-push flag was used */
  readonly noPush: boolean;
  /** Whether --all flag was used (stage all changes) */
  readonly stageAll: boolean;
  /** VCS driver instance */
  readonly vcs: VcsDriver;
  /** VCS backend type */
  readonly vcsType: VcsType;
  /** Tag name for this release (e.g. "v1.2.3") */
  readonly tagName: string;
  /** Commit message for the release commit */
  readonly commitMessage: string;
  /** Ordered stage list to execute */
  readonly stages: ReleaseStage[];

  /** Files modified by stages (for diff display) */
  modifiedFiles: string[];
  /** Files to stage for the release commit */
  filesToStage: string[];
  /** Release notes extracted from RELEASE_NOTE.md */
  releaseNotes: string;

  /** Custom data bag for user stages */
  extra: Record<string, unknown>;
}

/**
 * A release pipeline stage function.
 * Return false to abort the pipeline.
 */
export type ReleaseStage = ((
  ctx: ReleasePipelineContext,
  // biome-ignore lint/suspicious/noConfusingVoidType: Allow void for stages that don't need to return anything
) => void | boolean | Promise<void | boolean>) & {
  /** Stage name for logging (falls back to function name) */
  stageName?: string;
};

// ============================================================================
// Stage Loader
// ============================================================================

/**
 * Load release stages from user config (`scripts/release-hooks.ts`).
 * Falls back to DEFAULT_STAGES if no user config exists.
 */
export async function loadReleaseStages(): Promise<ReleaseStage[]> {
  const hookPath = projectPath("scripts/release-hooks.ts");

  if (file.exists(hookPath)) {
    try {
      const module = await import(hookPath);
      const stages = module.stages ?? module.default;
      if (Array.isArray(stages) && stages.length > 0) {
        logger.info(`Loaded ${stages.length} release stages from ${hookPath}`);
        return stages;
      }
      logger.warning("No stages found in release-hooks.ts, using defaults");
    } catch (error) {
      logger.warning(
        `Failed to load stages from ${hookPath}: ${getErrorMessage(error)}`,
      );
    }
  }

  // Lazy import to avoid circular dependency
  const { DEFAULT_STAGES } = await import("./default-release-hooks");
  return DEFAULT_STAGES;
}

// ============================================================================
// Context Factory
// ============================================================================

/**
 * Create the initial pipeline context
 */
export function createPipelineContext(
  currentVersion: string,
  newVersion: string,
  options: { noPush?: boolean; stageAll?: boolean },
  vcs: VcsDriver,
  stages: ReleaseStage[],
): ReleasePipelineContext {
  return {
    commitMessage: renderTemplate(VCS.COMMIT_MESSAGE_TEMPLATE, newVersion),
    currentVersion,

    extra: {},
    filesToStage: [],

    modifiedFiles: [],
    newVersion,
    noPush: options.noPush ?? false,
    releaseNotes: "",
    stageAll: options.stageAll ?? false,
    stages,
    tagName: renderTemplate(VCS.TAG_TEMPLATE, newVersion),
    vcs,
    vcsType: vcs.type,
  };
}
