#!/usr/bin/env bun

/** Preserve the exact release build before bundlers can strip or sign it. */
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";
import {
  getErrorMessage,
  logger,
  parseCliArgs,
  projectPath,
  requireArg,
} from "../lib/utils";
import { getProjectConfig, type PlatformName } from "./constants";

const STAGING_DIR = ".debug-build";
const OUTPUT_DIR = "debug-artifacts";
const MACOS_TARGETS = ["aarch64-apple-darwin", "x86_64-apple-darwin"];
const TARGETS: Record<PlatformName, string> = {
  linux: "x86_64-unknown-linux-gnu",
  macos: "universal-apple-darwin",
  windows: "x86_64-pc-windows-msvc",
};

interface BuildOptions {
  name: string;
  platform: PlatformName;
  target: string;
  version: string;
}

interface BuildMetadata {
  commit: string;
  runUrl: string;
  runAttempt: string;
  tools: Record<string, string>;
  profile: Record<string, string>;
}

function filesIn(directory: string): string[] {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const fullPath = path.join(directory, entry.name);
    return entry.isDirectory() ? filesIn(fullPath) : [fullPath];
  });
}

function requireFile(file: string): void {
  if (
    !fs.existsSync(file) ||
    !fs.statSync(file).isFile() ||
    fs.statSync(file).size === 0
  ) {
    throw new Error(`Missing or empty build output: ${file}`);
  }
}

function copy(source: string, destination: string): void {
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  // Cargo's top-level dSYM is a link into deps/. Archive the actual contents,
  // since the build cache it points to is not part of the downloadable archive.
  fs.cpSync(source, destination, { dereference: true, recursive: true });
}

/** Save maps alongside their matching JS, then exclude maps from embedded assets. */
export function stageFrontend(root: string): void {
  const dist = path.join(root, "dist");
  const staging = path.join(root, STAGING_DIR);
  if (fs.existsSync(staging)) {
    throw new Error(
      `Stale debug staging directory: ${staging}. Remove it before rebuilding.`,
    );
  }
  const maps = filesIn(dist).filter((file) => file.endsWith(".map"));
  if (maps.length === 0) {
    throw new Error(
      "No frontend source maps found. Build with FOCUST_DEBUG_ARTIFACTS=true.",
    );
  }
  copy(dist, path.join(staging, "frontend"));
  for (const map of maps) fs.unlinkSync(map);
  logger.success(
    `Archived ${maps.length} source maps; dist now contains distribution assets only.`,
  );
}

/** Keep binary/symbol basenames together, including both halves of macOS universal. */
export function stageNative(
  root: string,
  platform: PlatformName,
  target: string,
  name: string,
): void {
  const targets = platform === "macos" ? [target, ...MACOS_TARGETS] : [target];
  const outputs: string[] = [];
  for (const triple of targets) {
    const directory = path.join(root, "src-tauri", "target", triple, "release");
    const binary = path.join(
      directory,
      platform === "windows" ? `${name}.exe` : name,
    );
    requireFile(binary);
    outputs.push(binary);
    if (platform === "macos") {
      // Tauri creates the universal executable with lipo. Cargo emits a dSYM
      // for each architecture, not for the synthetic universal target.
      if (triple !== target) {
        const dsym = `${binary}.dSYM`;
        const dwarfDirectory = path.join(
          dsym,
          "Contents",
          "Resources",
          "DWARF",
        );
        // Cargo preserves the hashed linker output name inside the dSYM even
        // when it copies the outer bundle to <name>.dSYM.
        const dwarfFiles = fs.existsSync(dwarfDirectory)
          ? filesIn(dwarfDirectory)
          : [];
        if (dwarfFiles.length === 0)
          throw new Error(`Missing DWARF data: ${dsym}`);
        for (const dwarf of dwarfFiles) requireFile(dwarf);
        outputs.push(dsym);
      }
    } else {
      const symbols =
        platform === "windows"
          ? path.join(directory, `${name}.pdb`)
          : `${binary}.dwp`;
      requireFile(symbols);
      outputs.push(symbols);
    }
  }
  // Validate every required file before staging anything: missing symbols must
  // fail the build, rather than silently uploading an incomplete archive.
  for (const source of outputs) {
    copy(
      source,
      path.join(
        root,
        STAGING_DIR,
        "native",
        path.relative(path.join(root, "src-tauri", "target"), source),
      ),
    );
  }
  logger.success(
    "Archived release executables and their matching debug symbols.",
  );
}

/** Include final packages too, since a bundler may change the distributed binary. */
export function archiveDebugBuild(
  root: string,
  options: BuildOptions,
  metadata: BuildMetadata,
): string {
  const staging = path.join(root, STAGING_DIR);
  if (
    !fs.existsSync(path.join(staging, "native")) ||
    !fs.existsSync(path.join(staging, "frontend"))
  ) {
    throw new Error(
      "Both frontend and native debug outputs must be staged before archiving.",
    );
  }
  const packages = path.join(root, "artifacts");
  if (filesIn(packages).length === 0)
    throw new Error("No distribution packages found.");
  copy(packages, path.join(staging, "packages"));

  const files = filesIn(staging)
    .filter((file) => file !== path.join(staging, "build-info.json"))
    .sort()
    .map((file) => {
      const relative = path.relative(staging, file).split(path.sep).join("/");
      return {
        bytes: fs.statSync(file).size,
        path: relative,
        sha256: createHash("sha256")
          .update(fs.readFileSync(file))
          .digest("hex"),
      };
    });
  fs.writeFileSync(
    path.join(staging, "build-info.json"),
    `${JSON.stringify(
      {
        ...options,
        ...metadata,
        files,
      },
      null,
      2,
    )}\n`,
  );

  const outputDir = path.join(root, OUTPUT_DIR);
  fs.mkdirSync(outputDir, { recursive: true });
  const filename = `${options.name}_${options.version}_${options.target}_${metadata.commit.slice(0, 12)}_debug.tar.gz`;
  const archive = path.join(outputDir, filename);
  // tar preserves executable permissions and dSYM directory structure;
  // uploading loose files with upload-artifact would lose Unix permissions.
  // Git Bash's GNU tar treats a Windows drive letter as a remote host. Relative
  // paths also work with Windows/macOS bsdtar and Linux GNU tar.
  execFileSync(
    "tar",
    ["-czf", path.posix.join(OUTPUT_DIR, filename), "-C", STAGING_DIR, "."],
    {
      cwd: root,
      stdio: "inherit",
    },
  );
  logger.success(`Created ${filename}`);
  return archive;
}

function commandVersion(command: string, args: string[]): string {
  return execFileSync(command, args, { encoding: "utf8" }).trim();
}

function main(): void {
  const raw = parseCliArgs(process.argv, {
    helpText: `Usage: bun scripts/ci/prepare-debug-artifacts.ts --stage <frontend|native|archive>
  --platform <windows|linux|macos>  Required for native and archive
  --target <rust-triple>           Required for native and archive
  --version <X.Y.Z>               Required for archive

Run frontend after Vite, native after tauri build --no-bundle, and archive
after tauri bundle and prepare-artifacts.ts. Outputs: ${STAGING_DIR}/ and ${OUTPUT_DIR}/.`,
  });
  const root = projectPath();
  const stage = requireArg(raw, "stage");
  if (stage === "frontend") {
    stageFrontend(root);
    return;
  }
  const platform = requireArg(raw, "platform") as PlatformName;
  const target = requireArg(raw, "target");
  const { name } = getProjectConfig();
  if (TARGETS[platform] !== target)
    throw new Error(`Unsupported platform/target pair: ${platform}/${target}`);
  if (stage === "native") {
    stageNative(root, platform, target, name);
  } else if (stage === "archive") {
    const version = requireArg(raw, "version");
    const metadata: BuildMetadata = {
      commit:
        process.env.GITHUB_SHA || commandVersion("git", ["rev-parse", "HEAD"]),
      profile: {
        debug: process.env.CARGO_PROFILE_RELEASE_DEBUG ?? "",
        splitDebuginfo: process.env.CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO ?? "",
        strip: process.env.CARGO_PROFILE_RELEASE_STRIP ?? "",
      },
      runAttempt: process.env.GITHUB_RUN_ATTEMPT ?? "",
      runUrl: process.env.GITHUB_RUN_ID
        ? `${process.env.GITHUB_SERVER_URL}/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID}`
        : "",
      tools: {
        bun: commandVersion("bun", ["--version"]),
        cargo: commandVersion("cargo", ["--version"]),
        rustc: commandVersion("rustc", ["--version", "--verbose"]),
        tauri: commandVersion("bun", ["run", "tauri", "--version"]),
      },
    };
    const archive = archiveDebugBuild(
      root,
      { name, platform, target, version },
      metadata,
    );
    if (process.env.GITHUB_STEP_SUMMARY) {
      fs.appendFileSync(
        process.env.GITHUB_STEP_SUMMARY,
        `\n### Release debugging archive\n\n- File: \`${path.basename(archive)}\`\n- Commit: \`${metadata.commit}\`\n- Download the \`debug-\` artifact from this run's **Artifacts** section.\n- Retention: **90 days**. Save a copy before expiry for long-term debugging.\n- Includes original release binaries, symbols, frontend maps, final packages and SHA-256 checksums.\n`,
      );
    }
  } else {
    throw new Error(`Unknown stage: ${stage}`);
  }
}

if (import.meta.main) {
  try {
    main();
  } catch (error) {
    logger.error(getErrorMessage(error));
    process.exit(1);
  }
}
