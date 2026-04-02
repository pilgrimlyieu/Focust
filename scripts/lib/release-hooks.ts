/**
 * Release hooks system
 *
 * A minimal, extensible hook system for the release process.
 * Hooks are executed at specific points during the release workflow.
 *
 * Usage:
 *   1. Define hooks in `scripts/release-hooks.ts`
 *   2. Export a `hooks` object implementing `ReleaseHooks`
 *   3. Hooks are automatically loaded and executed during release
 *
 * Available hook points:
 *   - preRelease:    Before any release steps (validation, setup)
 *   - preCommit:     Before creating the release commit
 *   - postCommit:    After commit and tag are created
 *   - postPush:      After pushing to remote (skipped if --no-push)
 *   - postRelease:   After all release steps complete (cleanup, notifications)
 */

import { file, logger, projectPath } from "./utils";

// ============================================================================
// Types
// ============================================================================

/**
 * Context passed to all hooks
 */
export interface ReleaseHookContext {
  /** Current version before release */
  currentVersion: string;
  /** New version being released */
  newVersion: string;
  /** Whether --no-push flag was used */
  noPush: boolean;
  /** Whether --all flag was used (stage all changes) */
  stageAll: boolean;
}

/**
 * Hook function signature
 * Return false to abort the release (only for pre-hooks)
 */
export type ReleaseHookFn = (
  ctx: ReleaseHookContext,
  // biome-ignore lint/suspicious/noConfusingVoidType: Allow void for hooks that don't need to return anything
) => void | boolean | Promise<void | boolean>;
export type ReleaseHookFns = ReleaseHookFn | ReleaseHookFn[];

/**
 * All available hook points
 */
export interface ReleaseHooks {
  /** Before any release steps */
  preRelease?: ReleaseHookFns;
  /** Before creating commit */
  preCommit?: ReleaseHookFns;
  /** After commit and tag created */
  postCommit?: ReleaseHookFns;
  /** After pushing to remote */
  postPush?: ReleaseHookFns;
  /** After all release steps complete */
  postRelease?: ReleaseHookFns;
}

/**
 * Hook point names (for iteration and logging)
 */
export type ReleaseHookPoint = keyof ReleaseHooks;

// ============================================================================
// Hook Runner
// ============================================================================

/**
 * Default empty hooks (no-op)
 */
const DEFAULT_RELEASE_HOOKS: ReleaseHooks = {};

/**
 * Loaded hooks instance
 */
let loadedReleaseHooks: ReleaseHooks = DEFAULT_RELEASE_HOOKS;

/**
 * Load hooks from `scripts/release-hooks.ts` if it exists
 */
export async function loadReleaseHooks(): Promise<void> {
  const hookPaths = [projectPath("scripts/release-hooks.ts")];

  for (const hookPath of hookPaths) {
    if (file.exists(hookPath)) {
      try {
        const module = await import(hookPath);
        loadedReleaseHooks = module.hooks ?? module.default ?? {};
        logger.info(`Loaded release hooks from ${hookPath}`);
        return;
      } catch (error) {
        logger.warning(
          `Failed to load hooks from ${hookPath}: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }
  }

  // No hooks file found - that's fine, use defaults
  loadedReleaseHooks = DEFAULT_RELEASE_HOOKS;
}

/**
 * Run a specific hook
 * @param {ReleaseHookPoint} point - The hook point to run
 * @param {ReleaseHookContext} ctx - The hook context
 * @returns {Promise<boolean>} false if hook aborted the release, true otherwise
 */
export async function runReleaseHook(
  point: ReleaseHookPoint,
  ctx: ReleaseHookContext,
): Promise<boolean> {
  const hooks = loadedReleaseHooks[point];

  if (!hooks) {
    return true;
  }

  try {
    logger.info(`Running ${point} hook...`);
    if (Array.isArray(hooks)) {
      for (const hook of hooks) {
        const result = await hook(ctx);
        if (result === false && point.startsWith("pre")) {
          logger.warning(`Release aborted by ${point} hook`);
          return false;
        }
      }
    } else {
      const result = await hooks(ctx);
      if (result === false && point.startsWith("pre")) {
        logger.warning(`Release aborted by ${point} hook`);
        return false;
      }
    }
    return true;
  } catch (error) {
    logger.error(
      `Hook ${point} failed: ${error instanceof Error ? error.message : String(error)}`,
    );
    throw error;
  }
}

/**
 * Create a hook context
 * @param {string} currentVersion - The current version before release
 * @param {string} newVersion - The new version being released
 * @param {object} options - Additional options (noPush, stageAll)
 * @returns {ReleaseHookContext} The created hook context
 */
export function createReleaseHookContext(
  currentVersion: string,
  newVersion: string,
  options: { noPush?: boolean; stageAll?: boolean } = {},
): ReleaseHookContext {
  return {
    currentVersion,
    newVersion,
    noPush: options.noPush ?? false,
    stageAll: options.stageAll ?? false,
  };
}
