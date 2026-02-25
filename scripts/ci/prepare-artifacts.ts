#!/usr/bin/env bun
/**
 * Prepare build artifacts for release
 *
 * Collects Tauri build outputs (installers, portable archives, signatures)
 * into a standardized artifacts directory with consistent naming.
 * Project name and portable resources are read from configuration files.
 *
 * Usage:
 *   bun scripts/ci/prepare-artifacts.ts --platform windows --version 0.3.4 --target x86_64-pc-windows-msvc
 *   bun scripts/ci/prepare-artifacts.ts --platform linux   --version 0.3.4 --target x86_64-unknown-linux-gnu
 *   bun scripts/ci/prepare-artifacts.ts --platform macos   --version 0.3.4 --target universal-apple-darwin
 *
 * Options:
 *   --platform <windows|linux|macos>  Target platform (required)
 *   --version  <X.Y.Z>               Version string without 'v' prefix (required)
 *   --target   <rust-triple>          Rust target triple (required)
 *   --output   <dir>                  Output directory (default: artifacts)
 */

import * as fs from "node:fs";
import * as path from "node:path";
import { PATHS } from "../lib/constants";
import {
  execOrThrow,
  exitWithError,
  file,
  logger,
  parseCliArgs,
  projectPath,
  requireArg,
} from "../lib/utils";
import {
  ARTIFACTS_DIR,
  artifactName,
  BUNDLE_PATHS,
  getPortableResourceDirs,
  getProjectConfig,
  type PlatformName,
  VALID_PLATFORMS,
} from "./constants";

// ============================================================================
// Argument Parsing
// ============================================================================

const HELP_TEXT = `Usage: bun scripts/ci/prepare-artifacts.ts [options]

Options:
  --platform <windows|linux|macos>  Target platform (required)
  --version  <X.Y.Z>                Version string (required)
  --target   <rust-triple>          Rust target triple (required)
  --output   <dir>                  Output directory (default: ${ARTIFACTS_DIR})`;

interface Args {
  platform: PlatformName;
  version: string;
  target: string;
  outputDir: string;
}

function parseArgs(): Args {
  const raw = parseCliArgs(process.argv, { helpText: HELP_TEXT });

  const platform = requireArg(raw, "platform");
  const version = requireArg(raw, "version");
  const target = requireArg(raw, "target");
  const outputDir = typeof raw.output === "string" ? raw.output : ARTIFACTS_DIR;

  if (!VALID_PLATFORMS.includes(platform as PlatformName)) {
    exitWithError(
      `Invalid platform: ${platform}. Must be one of: ${VALID_PLATFORMS.join(", ")}`,
    );
  }

  return {
    outputDir,
    platform: platform as PlatformName,
    target,
    version,
  };
}

// ============================================================================
// Helpers
// ============================================================================

/** Build the base path to Tauri bundle output */
function bundleBase(target: string): string {
  return projectPath(PATHS.TAURI_DIR, "target", target, "release", "bundle");
}

/** Find files matching an extension in a directory (one level deep) */
function findFiles(dir: string, ext: string): string[] {
  if (!file.exists(dir)) return [];

  const results: string[] = [];
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isFile() && entry.name.endsWith(ext)) {
      results.push(fullPath);
    }
    // Search into immediate subdirectories for nested bundlesSearch one level deep for nested bundles
    if (entry.isDirectory()) {
      const nested = fs.readdirSync(fullPath, { withFileTypes: true });
      for (const nEntry of nested) {
        if (nEntry.isFile() && nEntry.name.endsWith(ext)) {
          results.push(path.join(fullPath, nEntry.name));
        }
      }
    }
  }
  return results;
}

/** Copy a file and its .sig companion if it exists */
function copyWithSignature(src: string, destPath: string): void {
  file.copy(src, destPath);
  logger.success(`Copied: ${path.basename(destPath)}`);

  const sigSrc = `${src}.sig`;
  if (file.exists(sigSrc)) {
    file.copy(sigSrc, `${destPath}.sig`);
    logger.success(`Copied: ${path.basename(destPath)}.sig`);
  }
}

/** Recursively copy a directory */
function copyDirRecursive(src: string, dest: string): void {
  file.mkdir(dest);
  const entries = fs.readdirSync(src, { withFileTypes: true });
  for (const entry of entries) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyDirRecursive(srcPath, destPath);
    } else {
      file.copy(srcPath, destPath);
    }
  }
}

/** Create ZIP archive */
function createZip(sourceDir: string, outputPath: string): void {
  if (process.platform === "win32") {
    execOrThrow("powershell", [
      "-NoProfile",
      "-Command",
      `Compress-Archive -Path '${sourceDir}/*' -DestinationPath '${outputPath}' -Force`,
    ]);
  } else {
    exitWithError(
      "Portable ZIP creation is only implemented for Windows platform.",
    );
  }
}

// ============================================================================
// Platform-specific Artifact Preparation
// ============================================================================

function prepareWindows(version: string, target: string, outDir: string): void {
  const base = bundleBase(target);
  const config = getProjectConfig();

  // --- NSIS installer ---
  const nsisDir = path.join(base, BUNDLE_PATHS.windows.nsis);
  const installers = findFiles(nsisDir, ".exe").filter(
    (f) => !f.endsWith(".exe.sig"),
  );

  if (installers.length > 0) {
    const destName = artifactName(version, "windows", "installer.exe");
    copyWithSignature(installers[0], path.join(outDir, destName));
  } else {
    logger.warning("No NSIS installer found");
  }

  // --- Portable ZIP ---
  const exePath = projectPath(
    PATHS.TAURI_DIR,
    "target",
    target,
    "release",
    `${config.name}.exe`,
  );

  if (!file.exists(exePath)) {
    logger.warning(`Executable not found at ${exePath}, skipping portable ZIP`);
    return;
  }

  const portableDir = projectPath("portable");
  file.mkdir(portableDir);

  // Copy executable
  file.copy(exePath, path.join(portableDir, `${config.name}.exe`));
  logger.success("Copied executable to portable directory");

  // Copy resource directories defined in tauri.conf.json
  const resourceDirs = getPortableResourceDirs();
  for (const relDir of resourceDirs) {
    const srcDir = projectPath(PATHS.TAURI_DIR, relDir);
    if (file.exists(srcDir)) {
      copyDirRecursive(srcDir, path.join(portableDir, relDir));
      logger.success(`Copied resource: ${relDir}`);
    } else {
      logger.warning(`Resource directory not found: ${relDir}`);
    }
  }

  // Create ZIP
  const zipName = artifactName(version, "windows", "portable.zip");
  const zipPath = path.join(outDir, zipName);
  createZip(portableDir, zipPath);
  logger.success(`Created portable ZIP: ${zipName}`);

  // Clean up
  fs.rmSync(portableDir, { force: true, recursive: true });
}

function prepareLinux(version: string, target: string, outDir: string): void {
  const base = bundleBase(target);

  // AppImage
  const appImageDir = path.join(base, BUNDLE_PATHS.linux.appimage);
  const appImages = findFiles(appImageDir, ".AppImage").filter(
    (f) => !f.endsWith(".sig"),
  );
  if (appImages.length > 0) {
    const destName = artifactName(version, "linux", "amd64.AppImage");
    copyWithSignature(appImages[0], path.join(outDir, destName));
  }

  // Deb
  const debDir = path.join(base, BUNDLE_PATHS.linux.deb);
  const debs = findFiles(debDir, ".deb");
  if (debs.length > 0) {
    const destName = artifactName(version, "linux", "amd64.deb");
    file.copy(debs[0], path.join(outDir, destName));
    logger.success(`Copied: ${destName}`);
  }

  // RPM
  const rpmDir = path.join(base, BUNDLE_PATHS.linux.rpm);
  const rpms = findFiles(rpmDir, ".rpm");
  if (rpms.length > 0) {
    const destName = artifactName(version, "linux", "x86_64.rpm");
    file.copy(rpms[0], path.join(outDir, destName));
    logger.success(`Copied: ${destName}`);
  }
}

function prepareMacos(version: string, target: string, outDir: string): void {
  const base = bundleBase(target);

  // DMG
  const dmgDir = path.join(base, BUNDLE_PATHS.macos.dmg);
  const dmgs = findFiles(dmgDir, ".dmg");
  if (dmgs.length > 0) {
    const destName = artifactName(version, "macos", "universal.dmg");
    file.copy(dmgs[0], path.join(outDir, destName));
    logger.success(`Copied: ${destName}`);
  }

  // App tarball (used by updater)
  const macosDir = path.join(base, BUNDLE_PATHS.macos.macos);
  const tarballs = findFiles(macosDir, ".app.tar.gz").filter(
    (f) => !f.endsWith(".sig"),
  );
  if (tarballs.length > 0) {
    const destName = artifactName(version, "macos", "universal.app.tar.gz");
    copyWithSignature(tarballs[0], path.join(outDir, destName));
  }
}

// ============================================================================
// Main
// ============================================================================

function main(): void {
  const { platform, version, target, outputDir } = parseArgs();
  const outDir = projectPath(outputDir);

  logger.banner(
    `Prepare Artifacts: ${platform}`,
    `Version: ${version} | Target: ${target}`,
  );

  file.mkdir(outDir);

  switch (platform) {
    case "windows":
      prepareWindows(version, target, outDir);
      break;
    case "linux":
      prepareLinux(version, target, outDir);
      break;
    case "macos":
      prepareMacos(version, target, outDir);
      break;
  }

  // List output
  logger.spacer();
  logger.section("Artifacts prepared:");
  const files = fs.readdirSync(outDir);
  for (const f of files) {
    const stat = fs.statSync(path.join(outDir, f));
    const sizeMB = (stat.size / (1024 * 1024)).toFixed(2);
    logger.info(`  ${f} (${sizeMB} MB)`);
  }

  logger.spacer();
  logger.success("Artifact preparation complete!");
}

main();
