#!/usr/bin/env bun
/**
 * Generate Tauri updater manifest (latest.json)
 *
 * Scans artifacts directory for build outputs and generates a JSON manifest
 * compatible with Tauri's built-in updater plugin. Uses JSON.stringify for
 * safe serialization (no shell string concatenation).
 *
 * Usage:
 *   bun scripts/ci/generate-manifest.ts --version 0.3.4 --tag v0.3.4 --repo owner/repo
 *   bun scripts/ci/generate-manifest.ts --version 0.3.4 --tag v0.3.4 --repo owner/repo --artifacts-dir artifacts --output artifacts/latest.json
 *
 * Options:
 *   --version        <X.Y.Z>      Version string without 'v' prefix (required)
 *   --tag            <vX.Y.Z>     Git tag with 'v' prefix (required)
 *   --repo           <owner/repo> GitHub repository (required)
 *   --artifacts-dir  <dir>        Directory containing build artifacts (default: artifacts)
 *   --output         <path>       Output file path (default: artifacts/latest.json)
 */

import * as fs from "node:fs";
import * as path from "node:path";
import {
  file,
  logger,
  parseCliArgs,
  projectPath,
  requireArg,
} from "../lib/utils";
import {
  ARTIFACTS_DIR,
  type PlatformName,
  UPDATER_PLATFORMS,
} from "./constants";

// ============================================================================
// Types
// ============================================================================

interface Args {
  version: string;
  tag: string;
  repo: string;
  artifactsDir: string;
  output: string;
}

interface PlatformEntry {
  signature: string;
  url: string;
}

interface Manifest {
  version: string;
  notes: string;
  pub_date: string;
  platforms: Record<string, PlatformEntry>;
}

// ============================================================================
// Argument Parsing
// ============================================================================

const HELP_TEXT = `Usage: bun scripts/ci/generate-manifest.ts [options]

Options:
  --version        <X.Y.Z>      Version without 'v' prefix (required)
  --tag            <vX.Y.Z>     Git tag (required)
  --repo           <owner/repo> GitHub repository (required)
  --artifacts-dir  <dir>        Artifacts directory (default: ${ARTIFACTS_DIR})
  --output         <path>       Output path (default: <artifacts-dir>/latest.json)`;

function parseArgs(): Args {
  const raw = parseCliArgs(process.argv, { helpText: HELP_TEXT });

  const version = requireArg(raw, "version");
  const tag = requireArg(raw, "tag");
  const repo = requireArg(raw, "repo");
  const artifactsDir =
    typeof raw["artifacts-dir"] === "string"
      ? raw["artifacts-dir"]
      : ARTIFACTS_DIR;
  const output =
    typeof raw.output === "string"
      ? raw.output
      : path.join(artifactsDir, "latest.json");

  return { artifactsDir, output, repo, tag, version };
}

// ============================================================================
// Manifest Generation
// ============================================================================

/** Read signature from .sig file, return empty string if not found */
function readSignature(sigPath: string): string {
  if (file.exists(sigPath)) {
    return file.read(sigPath).trim();
  }
  logger.warning(`Signature file not found: ${path.basename(sigPath)}`);
  return "";
}

/** Build download URL for a release asset */
function downloadUrl(repo: string, tag: string, filename: string): string {
  return `https://github.com/${repo}/releases/download/${tag}/${filename}`;
}

/**
 * Register a detected platform into the manifest entries.
 * Maps our platform name to all Tauri updater platform identifiers.
 */
function registerPlatform(
  platforms: Record<string, PlatformEntry>,
  platformName: PlatformName,
  artifactFile: string,
  absDir: string,
  repo: string,
  tag: string,
): void {
  const sig = readSignature(path.join(absDir, `${artifactFile}.sig`));
  const url = downloadUrl(repo, tag, artifactFile);

  for (const id of UPDATER_PLATFORMS[platformName]) {
    platforms[id] = { signature: sig, url };
  }
  logger.success(`${platformName} platform detected: ${artifactFile}`);
}

/** Detect platform artifacts and build manifest entries */
function detectPlatforms(
  artifactsDir: string,
  repo: string,
  tag: string,
): Record<string, PlatformEntry> {
  const absDir = projectPath(artifactsDir);
  const files = fs.readdirSync(absDir);
  const platforms: Record<string, PlatformEntry> = {};

  // Windows: *_windows_installer.exe
  const win = files.find(
    (f) => f.includes("_windows_") && f.endsWith("_installer.exe"),
  );
  if (win) registerPlatform(platforms, "windows", win, absDir, repo, tag);

  // Linux: *_linux_*.AppImage
  const linux = files.find(
    (f) => f.includes("_linux_") && f.endsWith(".AppImage"),
  );
  if (linux) registerPlatform(platforms, "linux", linux, absDir, repo, tag);

  // macOS: *_macos_*.app.tar.gz (not .sig)
  const macos = files.find(
    (f) =>
      f.includes("_macos_") && f.endsWith(".app.tar.gz") && !f.endsWith(".sig"),
  );
  if (macos) registerPlatform(platforms, "macos", macos, absDir, repo, tag);

  return platforms;
}

// ============================================================================
// Main
// ============================================================================

function main(): void {
  const { version, tag, repo, artifactsDir, output } = parseArgs();

  logger.banner("Generate Update Manifest", `Version: v${version}`);

  const platforms = detectPlatforms(artifactsDir, repo, tag);

  if (Object.keys(platforms).length === 0) {
    logger.warning(
      "No platform artifacts detected. Generating empty manifest.",
    );
  }

  const manifest: Manifest = {
    notes: `https://github.com/${repo}/releases/tag/${tag}`,
    platforms,
    pub_date: new Date().toISOString(),
    version: `v${version}`,
  };

  const outputPath = projectPath(output);
  const outputDir = path.dirname(outputPath);
  if (!file.exists(outputDir)) {
    file.mkdir(outputDir);
  }

  const json = JSON.stringify(manifest, null, 2);
  file.write(outputPath, json);

  logger.spacer();
  logger.section("Generated latest.json:");
  logger.multiline(json.split("\n"));
  logger.spacer();
  logger.success(`Manifest written to ${output}`);
}

main();
