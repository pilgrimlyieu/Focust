/**
 * CI-specific constants for GitHub Actions workflows
 *
 * Platform mappings, artifact naming conventions, and build configuration.
 * Project-specific values (name, resources) are read from configuration files
 * at runtime — see {@link getProjectConfig}.
 */

import * as fs from "node:fs";
import * as path from "node:path";
import { PATHS } from "../lib/constants";
import {
  file,
  getProjectName,
  getTauriConfig,
  projectPath,
} from "../lib/utils";

// ============================================================================
// Platform Configuration
// ============================================================================

export type PlatformName = "linux" | "macos" | "windows";

/**
 * Tauri updater platform identifiers mapped from our platform names
 */
export const UPDATER_PLATFORMS: Record<PlatformName, string[]> = {
  linux: ["linux-x86_64"],
  macos: ["darwin-x86_64", "darwin-aarch64"],
  windows: ["windows-x86_64"],
};

/**
 * Valid platform names for argument validation
 */
export const VALID_PLATFORMS: PlatformName[] = ["windows", "linux", "macos"];

// ============================================================================
// Defaults
// ============================================================================

/**
 * Default output directory for prepared artifacts
 */
export const ARTIFACTS_DIR = "artifacts";

// ============================================================================
// Bundle Paths (relative to src-tauri/target/{target}/release/bundle/)
// ============================================================================

export const BUNDLE_PATHS = {
  linux: {
    appimage: "appimage",
    deb: "deb",
    rpm: "rpm",
  },
  macos: {
    dmg: "dmg",
    macos: "macos",
  },
  windows: {
    nsis: "nsis",
  },
} as const;

// ============================================================================
// Project Config (read from package.json + tauri.conf.json)
// ============================================================================

interface ProjectConfig {
  /** Project name from package.json (e.g. "focust") */
  name: string;
  /** Product name from tauri.conf.json (e.g. "Focust"), falls back to name */
  productName: string;
  /** Bundle resources from tauri.conf.json (e.g. ["assets/sounds/*"]) */
  resources: string[];
}

let cachedConfig: ProjectConfig | null = null;

/**
 * Read project configuration from package.json and tauri.conf.json.
 * Results are cached for subsequent calls.
 *
 * All project-specific naming derives from this — no hardcoded project names.
 */
export function getProjectConfig(): ProjectConfig {
  if (cachedConfig) return cachedConfig;

  const name = getProjectName();
  const tauri = getTauriConfig();
  const productName = (tauri.productName as string | undefined) ?? name;

  // Read bundle.resources from tauri config
  const bundle = tauri.bundle as
    | { resources?: string[] | Record<string, string> }
    | undefined;
  let resources: string[] = [];
  if (bundle?.resources) {
    if (Array.isArray(bundle.resources)) {
      resources = bundle.resources;
    } else {
      // Object format: { "target": "source" }
      resources = Object.values(bundle.resources);
    }
  }

  cachedConfig = { name, productName, resources };
  return cachedConfig;
}

/**
 * Build artifact filename: {name}_{version}_{platform}_{suffix}
 */
export function artifactName(
  version: string,
  platform: string,
  suffix: string,
): string {
  const { name } = getProjectConfig();
  return `${name}_${version}_${platform}_${suffix}`;
}

/**
 * Resolve resource glob patterns from tauri.conf.json to actual directories
 * that need to be included in portable builds.
 *
 * For patterns like "assets/sounds/*", returns the parent directory "assets/sounds".
 * Returns paths relative to src-tauri/.
 */
export function getPortableResourceDirs(): string[] {
  const { resources } = getProjectConfig();
  const dirs = new Set<string>();

  for (const pattern of resources) {
    // Strip glob parts to get the directory
    // "assets/sounds/*" → "assets/sounds"
    // "assets/**" → "assets"
    // "resources/i18n.json" → "resources" (the parent dir)
    const cleaned = pattern.replace(/[/*]+$/, "");
    const tauriPath = projectPath(PATHS.TAURI_DIR, cleaned);

    if (file.exists(tauriPath)) {
      // Check if it's a file — if so, use the parent directory
      const stats = fs.statSync(tauriPath);
      if (stats.isFile()) {
        const parent = path.dirname(cleaned);
        if (parent !== ".") {
          dirs.add(parent);
        }
      } else {
        dirs.add(cleaned);
      }
    } else {
      // Pattern might not resolve yet (glob), use parent dir
      const parentDir = path.dirname(cleaned);
      if (parentDir !== ".") {
        dirs.add(parentDir);
      }
    }
  }

  return [...dirs];
}
