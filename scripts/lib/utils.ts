/**
 * Common utilities for Focust scripts
 */

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";

// Color codes for terminal output
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
   */
  error: (msg: string) => {
    console.log(`${colors.red}❌ ${msg}${colors.reset}`);
  },
  /**
   * Print info message (cyan)
   */
  info: (msg: string) => {
    console.log(`${colors.cyan}${msg}${colors.reset}`);
  },

  /**
   * Print multi-line message with consistent indentation
   */
  multiline: (messages: string[], indent = 0) => {
    const prefix = " ".repeat(indent);
    for (const msg of messages) {
      console.log(`${prefix}${msg}`);
    }
  },

  /**
   * Print section header (yellow with 📝)
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
   */
  step: (step: number, msg: string) => {
    console.log(`${colors.blue}Step ${step}: ${msg}${colors.reset}`);
  },

  /**
   * Print success message (green with ✅)
   */
  success: (msg: string) => {
    console.log(`${colors.green}✅ ${msg}${colors.reset}`);
  },

  /**
   * Print warning message (yellow with ⚠️)
   */
  warning: (msg: string) => {
    console.log(`${colors.yellow}⚠️  ${msg}${colors.reset}`);
  },
} as const;

/**
 * Prompt user for input (synchronous)
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
 */
export function confirm(message: string): boolean {
  const response = prompt(`${message} (y/N): `);
  return response.toLowerCase() === "y";
}

/**
 * Execute shell command
 */
export function exec(command: string, args: string[]): boolean {
  const result = spawnSync(command, args, {
    shell: true,
    stdio: "inherit",
  });
  return result.status === 0;
}

/**
 * Execute shell command and throw on error
 */
export function execOrThrow(command: string, args: string[]): void {
  const result = spawnSync(command, args, {
    shell: true,
    stdio: "inherit",
  });

  if (result.status !== 0) {
    throw new Error(`Command failed: ${command} ${args.join(" ")}`);
  }
}

/**
 * File system utilities
 */
export const file = {
  /**
   * Copy file
   */
  copy: (src: string, dest: string): void => {
    fs.copyFileSync(src, dest);
  },
  /**
   * Check if file exists
   */
  exists: (path: string): boolean => fs.existsSync(path),

  /**
   * Create directory (recursive)
   */
  mkdir: (path: string): void => {
    fs.mkdirSync(path, { recursive: true });
  },

  /**
   * Read file as string
   */
  read: (path: string): string => fs.readFileSync(path, "utf-8"),

  /**
   * Write string to file
   */
  write: (path: string, content: string): void => {
    fs.writeFileSync(path, content, "utf-8");
  },
} as const;

/**
 * Exit with error message
 */
export function exitWithError(message: string, code = 1): never {
  logger.error(message);
  process.exit(code);
}

/**
 * Exit with success message
 */
export function exitWithSuccess(message: string): never {
  logger.success(message);
  process.exit(0);
}
