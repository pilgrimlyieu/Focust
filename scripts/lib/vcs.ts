/**
 * VCS (Version Control System) abstraction layer
 *
 * Provides a unified interface for Git and Jujutsu (JJ) operations,
 * enabling the release script to work with either VCS backend.
 *
 * VCS resolution priority (highest to lowest):
 *   1. CLI --vcs argument
 *   2. VCS.DEFAULT_VCS in constants.ts (if not "auto")
 *   3. Auto-detect via .jj directory presence
 */

import { existsSync } from "node:fs";
import { VCS } from "./constants";
import { execOrThrow, logger, projectPath } from "./utils";

// ============================================================================
// Types
// ============================================================================

/** Supported VCS backend types */
export type VcsType = "git" | "jj";

/**
 *  VCS driver interface
 *
 *  All version control operations required by the release script.
 */
export interface VcsDriver {
  /** VCS backend identifier */
  readonly type: VcsType;

  /** Show diff for the given files */
  showDiff(files: string[]): void;

  /**
   * Commit changes
   * @param {string} message Commit message
   * @param {string[]} files   Files to include (used when stageAll is false)
   * @param {boolean} stageAll Whether to include all working copy changes
   */
  commit(message: string, files: string[], stageAll: boolean): void;

  /** Create a version tag */
  createTag(tag: string): void;

  /** Push the branch/bookmark to remote */
  pushBranch(branch: string): void;

  /** Push the tag to remote */
  pushTag(tag: string): void;
}

// ============================================================================
// Git Driver
// ============================================================================

function git(...args: string[]): void {
  execOrThrow("git", args);
}

/** Git driver implementation */
export const gitDriver: VcsDriver = {
  commit(message, files, stageAll) {
    if (stageAll) {
      logger.info("🔍 Staging all changes");
      git("add", "-A");
    } else {
      logger.info("🔍 Staging specific changes");
      git("add", ...files);
    }
    git("commit", "-m", message);
  },

  createTag(tag) {
    git("tag", tag);
  },

  pushBranch(branch) {
    git("push", "origin", branch);
  },

  pushTag(tag) {
    git("push", "origin", "tag", tag);
  },

  showDiff(files) {
    git("diff", "-U0", ...files);
  },
  type: "git",
};

// ============================================================================
// JJ Driver (Jujutsu, colocated mode)
// ============================================================================

function jj(...args: string[]): void {
  execOrThrow("jj", args);
}

/** Jujutsu driver implementation (colocated mode) */
export const jjDriver: VcsDriver = {
  commit(message, files, stageAll) {
    // JJ has no staging area. The working copy is always a commit (@).
    // `jj commit` finalizes @ with the given message and creates a new empty @.
    // With path args, only those files enter the commit; the rest stay in the new @.
    if (stageAll) {
      logger.info("🔍 Committing all changes");
      jj("commit", "-m", message);
    } else {
      logger.info("🔍 Committing specific changes");
      jj("commit", "-m", message, ...files);
    }
  },

  createTag(tag) {
    // After `jj commit`, the release content is at @- (the new @ is an empty working copy)
    jj("tag", "set", tag, "-r", "@-");
  },

  pushBranch(branch) {
    // Move the bookmark to the release commit, then push
    jj("bookmark", "move", branch, "--to", "@-");
    jj("git", "push", "--bookmark", branch);
  },

  pushTag(tag) {
    jj("git", "push", "--tag", tag);
  },

  showDiff(files) {
    jj("diff", ...files);
  },
  type: "jj",
};

// ============================================================================
// VCS Resolution
// ============================================================================

/**
 * Auto-detect VCS type from the project directory.
 *
 * Detection: presence of .jj directory → JJ, otherwise → Git
 */
export function detectVcsType(): VcsType {
  return existsSync(projectPath(".jj")) ? "jj" : "git";
}

/**
 * Resolve the effective VCS type using the priority chain:
 *   CLI arg > VCS.DEFAULT_VCS (if not "auto") > auto-detect
 *
 * @param {VcsType?} forced Explicit type from CLI --vcs argument
 */
export function resolveVcsType(forced?: VcsType): VcsType {
  if (forced) return forced;
  if (VCS.DEFAULT_VCS !== "auto") return VCS.DEFAULT_VCS;
  return detectVcsType();
}

/**
 * Create a VCS driver instance for the resolved backend.
 *
 * @param {VcsType?} forced Explicit type from CLI --vcs argument
 */
export function createVcsDriver(forced?: VcsType): VcsDriver {
  const type = resolveVcsType(forced);
  const driver = type === "jj" ? jjDriver : gitDriver;

  const source = forced
    ? "cli"
    : VCS.DEFAULT_VCS !== "auto"
      ? "config"
      : "auto-detect";
  logger.info(
    `VCS: ${type === "jj" ? "Jujutsu (colocated)" : "Git"} (${source})`,
  );

  return driver;
}

/**
 * Generate a manual push hint when the user skips the push step.
 */
export function getManualPushHint(type: VcsType, tag: string): string {
  if (type === "jj") {
    return [
      `  jj bookmark move ${VCS.DEFAULT_BRANCH} --to @-`,
      `  jj git push --bookmark ${VCS.DEFAULT_BRANCH}`,
      `  git push origin tag ${tag}`,
    ].join("\n");
  }
  return `  git push origin ${VCS.DEFAULT_BRANCH} && git push origin tag ${tag}`;
}
