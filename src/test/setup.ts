import { config } from "@vue/test-utils";
import { beforeEach, vi } from "vitest";

/**
 * Note: Currently using manual Tauri API mocks instead of @tauri-apps/api-mocks
 *
 * The manual mocks below provide sufficient test coverage for our needs.
 * Using official Tauri mock packages would require additional dependencies
 * and doesn't provide significant benefits for our current test scenarios.
 *
 * If more sophisticated Tauri API testing is needed in the future, consider
 * migrating to official mocking solutions.
 */

// Polyfill structuredClone with JSON fallback for tests
// This is needed because test mocks may contain non-cloneable properties
const nativeStructuredClone = globalThis.structuredClone;
globalThis.structuredClone = <T>(obj: T): T => {
  try {
    return nativeStructuredClone(obj);
  } catch {
    // Fallback to JSON clone for test objects with functions
    return JSON.parse(JSON.stringify(obj));
  }
};

// Mock browser APIs
if (typeof globalThis.screen === "undefined") {
  globalThis.screen = {
    availHeight: 1040,
    availWidth: 1920,
    height: 1080,
    width: 1920,
  } as Screen;
}

// Mock matchMedia for theme detection
if (typeof window.matchMedia === "undefined") {
  window.matchMedia = vi.fn().mockImplementation((query: string) => ({
    addEventListener: vi.fn(),
    addListener: vi.fn(),
    dispatchEvent: vi.fn(),
    matches: !query.includes("dark"),
    media: query,
    onchange: null,
    removeEventListener: vi.fn(),
    removeListener: vi.fn(),
  }));
}

// Mock Tauri API
const mockInvoke = vi.fn();
const mockListen = vi.fn();
const mockEmit = vi.fn();
const mockOnce = vi.fn();
const mockGetCurrentWindow = vi.fn(() => ({
  close: vi.fn(),
  hide: vi.fn(),
  label: "test-window",
  setAlwaysOnTop: vi.fn(),
  setFullscreen: vi.fn(),
  show: vi.fn(),
}));

// Mock WebviewWindow
class MockWebviewWindow {
  label: string;
  // biome-ignore lint/suspicious/noExplicitAny: This is a mock class for testing purposes
  constructor(label: string, _options?: any) {
    this.label = label;
  }
  static async getByLabel(_label: string) {
    return {
      close: vi.fn().mockResolvedValue(undefined),
    };
  }
  // biome-ignore lint/complexity/noBannedTypes: This is a mock class for testing purposes
  once(_event: string, _handler: Function) {
    mockOnce(_event, _handler);
    return Promise.resolve();
  }
  // biome-ignore lint/suspicious/noExplicitAny: This is a mock class for testing purposes
  emit(_event: string, _payload?: any) {
    mockEmit(_event, _payload);
    return Promise.resolve();
  }
  close() {
    return Promise.resolve();
  }
}

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  emit: mockEmit,
  listen: mockListen,
  once: vi.fn(),
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  availableMonitors: vi.fn(() => Promise.resolve([])),
  WebviewWindow: MockWebviewWindow,
}));

vi.mock("@tauri-apps/api/window", () => ({
  availableMonitors: vi.fn(() => Promise.resolve([])),
  currentMonitor: vi.fn(() => Promise.resolve(null)),
  getCurrentWindow: mockGetCurrentWindow,
  WebviewWindow: MockWebviewWindow,
}));

// Configure Vue Test Utils
config.global.stubs = {
  teleport: true,
};

// Mock console.error and console.warn to suppress expected logs in tests
const originalConsoleError = console.error;
const consoleErrorMock = vi.fn((...args: unknown[]) => {
  const message = args[0]?.toString() || "";
  // Suppress expected test errors
  if (
    message.includes("Failed to load config") ||
    message.includes("Failed to save config") ||
    message.includes("Failed to load suggestions")
  ) {
    return; // Silently ignore these expected test errors
  }
  // Log other errors normally
  originalConsoleError(...args);
});

const originalConsoleWarn = console.warn;
const consoleWarnMock = vi.fn((...args: unknown[]) => {
  // Log warnings normally (no suppressions needed)
  originalConsoleWarn(...args);
});

// Reset mocks before each test
beforeEach(() => {
  mockInvoke.mockReset();
  mockListen.mockReset();
  mockEmit.mockReset();
  console.error = consoleErrorMock;
  console.warn = consoleWarnMock;
});

// Export mocks for use in tests
export { mockInvoke, mockListen, mockEmit };
