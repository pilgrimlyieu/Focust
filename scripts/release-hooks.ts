/**
 * Release pipeline stage configuration
 *
 * Customize the release workflow by composing stages.
 * Each stage receives the pipeline context and can modify it.
 * Return false from any stage to abort the release.
 *
 * Available default stages (import from "./lib/default-release-hooks"):
 *   defaults.updatePackageVersion  - Update package.json version
 *   defaults.updateTauriConfig     - Update tauri.conf.json version (skips if missing)
 *   defaults.updateChangelog       - Extract release notes, update CHANGELOG.md
 *   defaults.commit                - Stage files and create commit
 *   defaults.tag                   - Create version tag
 *   defaults.push                  - Push branch and tag to remote
 *   defaults.resetReleaseNote      - Reset RELEASE_NOTE.md to template
 *
 * Example customization:
 *   import { defaults } from "./lib/default-release-hooks";
 *   export const stages = [
 *     defaults.updatePackageVersion,
 *     // skip Tauri config and changelog
 *     defaults.commit,
 *     defaults.tag,
 *     defaults.push,
 *   ];
 */

import { DEFAULT_STAGES } from "./lib/default-release-hooks";
import type { ReleaseStage } from "./lib/release-hooks";

export const stages: ReleaseStage[] = DEFAULT_STAGES;
