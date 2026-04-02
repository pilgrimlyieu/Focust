/**
 * Common utilities for Tauri project scripts
 *
 * Project-agnostic utilities for release automation and setup scripts.
 * All project-specific values are read from configuration files.
 */

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { PATHS } from "./constants";

// ============================================================================
// Types
// ============================================================================

interface PackageJson {
  name: string;
  version: string;
  [key: string]: unknown;
}

interface TauriConfig {
  productName?: string;
  identifier?: string;
  version?: string;
  [key: string]: unknown;
}

// ============================================================================
// Logging
// ============================================================================

const colors = {
  blue: "\x1b[34m",
  cyan: "\x1b[36m",
  green: "\x1b[32m",
  magenta: "\x1b[35m",
  red: "\x1b[31m",
  reset: "\x1b[0m",
  yellow: "\x1b[33m",
} as const;

/**
 * Logger with consistent formatting
 */
export const logger = {
  /**
   * Print banner with title
   * @param {string} title - The title to display in the banner
   * @param {string} [subtitle] - Optional subtitle to display below the title
   */
  banner: (title: string, subtitle?: string) => {
    const border = "=".repeat(title.length + 4);
    console.log(`${colors.cyan}${border}${colors.reset}`);
    console.log(`${colors.cyan}  ${title}${colors.reset}`);
    if (subtitle) {
      console.log(`${colors.cyan}  ${subtitle}${colors.reset}`);
    }
    console.log(`${colors.cyan}${border}${colors.reset}`);
  },

  /**
   * Print error message (red with ❌)
   * @param {string} msg - The error message to display
   */
  error: (msg: string) => {
    console.log(`${colors.red}❌ ${msg}${colors.reset}`);
  },
  /**
   * Print info message (cyan)
   * @param {string} msg - The info message to display
   */
  info: (msg: string) => {
    console.log(`${colors.cyan}${msg}${colors.reset}`);
  },

  /**
   * Print message
   * @param {string} msg - The message to display
   */
  log: (msg: string) => {
    console.log(msg);
  },

  /**
   * Print multi-line message with consistent indentation
   * @param {string[]} messages - Array of message lines to display
   * @param {number} [indent=0] - Number of spaces to indent each line
   */
  multiline: (messages: string[], indent: number = 0) => {
    const prefix = " ".repeat(indent);
    for (const msg of messages) {
      console.log(`${prefix}${msg}`);
    }
  },

  /**
   * Print section header (yellow with 📝)
   * @param {string} msg - The section title to display
   */
  section: (msg: string) => {
    console.log(`${colors.yellow}📝 ${msg}${colors.reset}`);
  },

  /**
   * Print a blank line
   */
  spacer: () => console.log(),

  /**
   * Print step header (blue)
   * @param {number} step - The step number to display
   * @param {string} msg - The step message to display
   */
  step: (step: number, msg: string) => {
    console.log(`${colors.blue}Step ${step}: ${msg}${colors.reset}`);
  },

  /**
   * Print success message (green with ✅)
   * @param {string} msg - The success message to display
   */
  success: (msg: string) => {
    console.log(`${colors.green}✅ ${msg}${colors.reset}`);
  },

  /**
   * Print warning message (yellow with ⚠️)
   * @param {string} msg - The warning message to display
   */
  warning: (msg: string) => {
    console.log(`${colors.yellow}⚠️  ${msg}${colors.reset}`);
  },
} as const;

// ============================================================================
// Input
// ============================================================================

/**
 * Prompt user for input (synchronous)
 * @param {string} message - The prompt message to display
 * @returns {string} The user's input
 */
export function prompt(message: string): string {
  process.stdout.write(`${colors.cyan}${message}${colors.reset}`);
  const buffer = Buffer.alloc(1024);
  const bytesRead = fs.readSync(
    process.stdin.fd,
    buffer,
    0,
    buffer.length,
    null,
  );
  return buffer.toString("utf-8", 0, bytesRead).trim();
}

/**
 * Confirm with user (y/N)
 * @param {string} message - The confirmation message to display
 * @returns {boolean} True if user confirmed with 'y', false otherwise
 */
export function confirm(message: string): boolean {
  const response = prompt(`${message} (y/N): `);
  return response.toLowerCase() === "y";
}

// ============================================================================
// Execution
// ============================================================================

/**
 * Execute command
 * @param {string} command - The command to execute
 * @param {string[]} args - Array of arguments to pass to the command
 * @returns {boolean} True if command succeeded (exit code 0), false otherwise
 */
export function exec(command: string, args: string[]): boolean {
  const result = spawnSync(command, args, {
    stdio: "inherit",
  });
  return result.status === 0;
}

/**
 * Execute command and throw on error
 * @param {string} command - The command to execute
 * @param {string[]} args - Array of arguments to pass to the command
 * @throws {Error} If the command exits with a non-zero status
 */
export function execOrThrow(command: string, args: string[]): void {
  const result = spawnSync(command, args, {
    stdio: "inherit",
  });

  if (result.status !== 0) {
    throw new Error(`Command failed: ${command} ${args.join(" ")}`);
  }
}

/**
 * Exit with error message
 * @param {string} message - The error message to display before exiting
 * @param {number} [code=1] - The exit code to use (default: 1)
 */
export function exitWithError(message: string, code: number = 1): never {
  logger.error(message);
  process.exit(code);
}

/**
 * Exit with success message
 * @param {string} message - The success message to display before exiting
 */
export function exitWithSuccess(message: string): never {
  logger.success(message);
  process.exit(0);
}

/**
 * Execute command and capture stdout
 * @param {string} command - The command to execute
 * @param {string[]} args - Array of arguments to pass to the command
 * @param {{ warnOnError?: boolean }} [options] - Optional configuration
 * @returns {string} Trimmed stdout output
 */
export function execCapture(
  command: string,
  args: string[],
  options?: { warnOnError?: boolean },
): string {
  const result = spawnSync(command, args, {
    encoding: "utf-8",
    stdio: ["pipe", "pipe", "pipe"],
  });
  if (
    options?.warnOnError &&
    (result.error != null || (result.status !== null && result.status !== 0))
  ) {
    const stderr = (result.stderr ?? "").trim();
    logger.warning(
      `Command '${command} ${args.join(" ")}' exited with code ${
        result.status ?? "unknown"
      }${stderr ? `: ${stderr}` : ""}`,
    );
  }
  return (result.stdout ?? "").trim();
}

/**
 * Parse CLI arguments into a key-value map.
 *
 * Supports `--key value` pairs and `--flag` booleans.
 * @param {string[]} argv - Raw process.argv (will be sliced from index 2)
 * @param {{ helpText?: string }} [options] - Optional help text printed on --help
 * @returns {Record<string, string | true>} Parsed arguments map
 */
export function parseCliArgs(
  argv: string[],
  options?: { helpText?: string },
): Record<string, string | true> {
  const args = argv.slice(2);
  const result: Record<string, string | true> = {};

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--help" || arg === "-h") {
      if (options?.helpText) {
        logger.log(options.helpText);
      }
      process.exit(0);
    }
    if (arg.startsWith("--")) {
      const key = arg.slice(2);
      const next = args[i + 1];
      if (next && !next.startsWith("--")) {
        result[key] = next;
        i++;
      } else {
        result[key] = true;
      }
    }
  }

  return result;
}

/**
 * Require a string argument from parsed CLI args, exit with error if missing.
 * @param {Record<string, string | true>} args - Parsed args map
 * @param {string} key - Argument key (without --)
 * @param {string} [label] - Human-readable label for error messages
 * @returns {string} The argument value
 */
export function requireArg(
  args: Record<string, string | true>,
  key: string,
  label?: string,
): string {
  const val = args[key];
  if (!val || val === true) {
    exitWithError(`Missing required argument: --${label ?? key}`);
  }
  return val;
}

// ============================================================================
// Files
// ============================================================================

/**
 * File system utilities
 */
export const file = {
  /**
   * Copy file
   * @param {string} src - The source file path
   * @param {string} dest - The destination file path
   */
  copy: (src: string, dest: string): void => {
    fs.copyFileSync(src, dest);
  },
  /**
   * Check if file exists
   * @param {string} path - The file path to check
   * @returns {boolean} True if file exists, false otherwise
   */
  exists: (path: string): boolean => fs.existsSync(path),

  /**
   * Create directory (recursive)
   * @param {string} path - The directory path to create
   */
  mkdir: (path: string): void => {
    fs.mkdirSync(path, { recursive: true });
  },

  /**
   * Read file as string
   * @param {string} path - The file path to read
   * @returns {string} The file contents as a string
   */
  read: (path: string): string => fs.readFileSync(path, "utf-8"),

  /**
   * Write string to file
   * @param {string} path - The file path to write to
   * @param {string} content - The string content to write to the file
   */
  write: (path: string, content: string): void => {
    fs.writeFileSync(path, content, "utf-8");
  },
} as const;

// ============================================================================
// Project
// ============================================================================

type ProjectInfo = { name: string; version: string };

/**
 * Cached project info to avoid repeated file reads
 */
let cachedProjectInfo: ProjectInfo | null = null;

/**
 * Get project info from `package.json`
 * Results are cached for subsequent calls
 * @returns {ProjectInfo} An object containing project name and version
 */
export function getProjectInfo(): ProjectInfo {
  if (cachedProjectInfo) {
    return cachedProjectInfo;
  }

  const packageJsonPath = projectPath(PATHS.PACKAGE_JSON);
  if (!file.exists(packageJsonPath)) {
    exitWithError(`${PATHS.PACKAGE_JSON} not found in current directory`);
  }

  const packageJson = JSON.parse(file.read(packageJsonPath)) as PackageJson;
  cachedProjectInfo = {
    name: packageJson.name,
    version: packageJson.version,
  };

  return cachedProjectInfo;
}

/**
 * Get Tauri config
 * @returns {TauriConfig} The contents of `src-tauri/tauri.conf.json` as an object
 */
export function getTauriConfig(): TauriConfig {
  const tauriConfigPath = projectPath(PATHS.TAURI_CONFIG);
  if (!file.exists(tauriConfigPath)) {
    exitWithError(`${PATHS.TAURI_CONFIG} not found`);
  }
  return JSON.parse(file.read(tauriConfigPath)) as TauriConfig;
}

/**
 * Get project name (from package.json name field)
 * @returns {string} The project name
 */
export function getProjectName(): string {
  return getProjectInfo().name;
}

/**
 * Get current version (from package.json)
 * @returns {string} The current project version
 */
export function getCurrentVersion(): string {
  return getProjectInfo().version;
}

// ============================================================================
// Version
// ============================================================================

/**
 * Version bump types
 */
export type BumpType = "major" | "minor" | "patch";

/**
 * Calculate new version based on bump type
 * @param {string} current - The current version (format: X.Y.Z)
 * @param {BumpType} bumpType - The type of version bump (major, minor, patch)
 * @returns {string} The new version after applying the bump
 * @throws {Error} If the current version format is invalid
 */
export function calculateNewVersion(
  current: string,
  bumpType: BumpType,
): string {
  const parts = current.split(".").map(Number);
  const [major, minor, patch] = parts;

  if (parts.length !== 3 || parts.some(Number.isNaN)) {
    throw new Error(`Invalid version format: ${current}`);
  }

  switch (bumpType) {
    case "major":
      return `${major + 1}.0.0`;
    case "minor":
      return `${major}.${minor + 1}.0`;
    case "patch":
      return `${major}.${minor}.${patch + 1}`;
  }
}

/**
 * Validate semver version format (X.Y.Z)
 * @param {string} version - The version string to validate
 * @returns {boolean} True if version is valid, false otherwise
 */
export function isValidVersion(version: string): boolean {
  return /^\d+\.\d+\.\d+$/.test(version);
}

/**
 * Update version field in a JSON file
 * @param {string} filePath - The path to the JSON file to update
 * @param {string} newVersion - The new version string to set (format: X.Y.Z)
 */
export function updateJsonVersion(filePath: string, newVersion: string): void {
  const fullPath = projectPath(filePath);
  const content = file.read(fullPath);
  const updated = content.replace(
    /("version"\s*:\s*")([^"]+)(")/,
    `$1${newVersion}$3`,
  );
  file.write(fullPath, updated);
  logger.success(`Updated ${filePath}`);
}

// ============================================================================
// Date
// ============================================================================

/**
 * Get current date in YYYY.M.D format
 * @returns {string} The current date as a string in the format YYYY.M.D
 */
export function getDateString(): string {
  const now = new Date();
  return `${now.getFullYear()}.${now.getMonth() + 1}.${now.getDate()}`;
}

// ============================================================================
// Anchor Operations
// ============================================================================

/**
 * Anchor manipulation utilities for text content.
 *
 * Anchor types:
 * - Insert: Add content before/after anchor (anchor preserved)
 * - Replace: Replace anchor with content (anchor removed)
 * - Marker: Extract content after anchor or between two anchors
 */
export const anchor = {
  /**
   * Check if anchor exists in content
   * @param {string} content - The content to search
   * @param {string} anchorStr - The anchor string to find
   * @returns {boolean} True if anchor exists
   */
  exists(content: string, anchorStr: string): boolean {
    return content.includes(anchorStr);
  },

  /**
   * Get content after anchor (to end of content or optional end anchor)
   * @param {string} content - The original content
   * @param {string} startAnchor - The start anchor string
   * @param {string} [endAnchor] - Optional end anchor string
   * @returns {string} Content between start anchor and end (trimmed)
   * @throws {Error} If start anchor is not found
   */
  getAfter(content: string, startAnchor: string, endAnchor?: string): string {
    const startIndex = content.indexOf(startAnchor);
    if (startIndex === -1) {
      throw new Error(`Start anchor not found: ${startAnchor}`);
    }

    const afterStart = content.slice(startIndex + startAnchor.length);

    if (endAnchor) {
      const endIndex = afterStart.indexOf(endAnchor);
      if (endIndex === -1) {
        return afterStart.trim();
      }
      return afterStart.slice(0, endIndex).trim();
    }

    return afterStart.trim();
  },

  /**
   * Get content between two anchors (exclusive)
   * @param {string} content - The original content
   * @param {string} startAnchor - The start anchor string
   * @param {string} endAnchor - The end anchor string
   * @returns {string} Content between anchors (trimmed)
   * @throws {Error} If either anchor is not found
   */
  getBetween(content: string, startAnchor: string, endAnchor: string): string {
    const startIndex = content.indexOf(startAnchor);
    if (startIndex === -1) {
      throw new Error(`Start anchor not found: ${startAnchor}`);
    }

    const afterStart = content.slice(startIndex + startAnchor.length);
    const endIndex = afterStart.indexOf(endAnchor);
    if (endIndex === -1) {
      throw new Error(`End anchor not found: ${endAnchor}`);
    }

    return afterStart.slice(0, endIndex).trim();
  },

  /**
   * Insert content after anchor (anchor preserved)
   * @param {string} content - The original content
   * @param {string} anchorStr - The anchor string to find
   * @param {string} insertion - The content to insert after anchor
   * @returns {string} Content with insertion added after anchor
   * @throws {Error} If anchor is not found
   */
  insertAfter(content: string, anchorStr: string, insertion: string): string {
    const index = content.indexOf(anchorStr);
    if (index === -1) {
      throw new Error(`Anchor not found: ${anchorStr}`);
    }
    const insertPos = index + anchorStr.length;
    return content.slice(0, insertPos) + insertion + content.slice(insertPos);
  },

  /**
   * Insert content before anchor (anchor preserved)
   * @param {string} content - The original content
   * @param {string} anchorStr - The anchor string to find
   * @param {string} insertion - The content to insert before anchor
   * @returns {string} Content with insertion added before anchor
   * @throws {Error} If anchor is not found
   */
  insertBefore(content: string, anchorStr: string, insertion: string): string {
    const index = content.indexOf(anchorStr);
    if (index === -1) {
      throw new Error(`Anchor not found: ${anchorStr}`);
    }
    return content.slice(0, index) + insertion + content.slice(index);
  },

  /**
   * Replace anchor with content (anchor removed)
   * @param {string} content - The original content
   * @param {string} anchorStr - The anchor string to replace
   * @param {string} replacement - The content to replace anchor with
   * @returns {string} Content with anchor replaced
   * @throws {Error} If anchor is not found
   */
  replace(content: string, anchorStr: string, replacement: string): string {
    const index = content.indexOf(anchorStr);
    if (index === -1) {
      throw new Error(`Anchor not found: ${anchorStr}`);
    }
    return (
      content.slice(0, index) +
      replacement +
      content.slice(index + anchorStr.length)
    );
  },
} as const;

// ============================================================================
// Chore
// ============================================================================

/**
 * Render template string with placeholders (e.g. "Hello, %s!" with "World" => "Hello, World!")
 * @param {string} template - The template string containing "%s" as placeholder(s)
 * @param {string} value - The value to replace the placeholder(s) with
 * @returns {string} The rendered string with placeholders replaced by the value
 */
export function renderTemplate(template: string, value: string): string {
  return template.replaceAll("%s", value);
}

/**
 * Join path segments relative to project root
 * @param {...string} segments - The path segments to join
 * @returns {string} The joined path relative to the project root
 */
export function projectPath(...segments: string[]): string {
  return path.join(process.cwd(), ...segments);
}

/**
 * Get error message from unknown error object
 * @param {unknown} err - The error object to extract the message from
 * @returns {string} The error message as a string
 */
export function getErrorMessage(err: unknown): string {
  if (err instanceof Error) {
    return err.message;
  }
  return String(err);
}