/**
 * Release hooks configuration
 *
 * Define custom hooks to run at various points during the release process.
 * See `scripts/lib/hooks.ts` for available hook points and the HookContext type.
 *
 * Hook points (in order of execution):
 *   preRelease    → Before any release steps (can abort by returning false)
 *   preCommit     → Before creating commit (can abort by returning false)
 *   postCommit    → After commit and tag created
 *   postPush      → After pushing to remote (skipped if --no-push)
 *   postRelease   → After all release steps complete
 */

import type { ReleaseHookContext, ReleaseHooks } from "./lib/release-hooks";
import { file, logger, projectPath } from "./lib/utils";

// ============================================================================
// Hook Implementations
// ============================================================================

/**
 * Reset `RELEASE_NOTE.md` to template after release
 *
 * Copies the template from `.github/RELEASE_NOTE_TEMPLATE.md` to `RELEASE_NOTE.md`
 * so it's ready for the next release cycle.
 */
function resetReleaseNoteTemplate(_ctx: ReleaseHookContext): void {
  const templatePath = projectPath(".github/RELEASE_NOTE_TEMPLATE.md");
  const targetPath = projectPath("RELEASE_NOTE.md");
  if (!file.exists(templatePath)) {
    logger.warning(
      "Template not found: .github/RELEASE_NOTE_TEMPLATE.md - skipping reset",
    );
    return;
  }
  file.copy(templatePath, targetPath);
  logger.success("Reset RELEASE_NOTE.md to template");
}

// ============================================================================
// Export Hooks
// ============================================================================

export const hooks: ReleaseHooks = {
  // preRelease: (ctx) => {
  //   // Run validations, check prerequisites
  //   // Return false to abort the release
  // },

  // preCommit: (ctx) => {
  //   // Final checks before commit
  //   // Return false to abort
  // },

  // postCommit: (ctx) => {
  //   // Actions after commit but before push
  // },

  // postPush: (ctx) => {
  //   // Trigger CI/CD, notify team, etc.
  // },

  postRelease: resetReleaseNoteTemplate,
};
