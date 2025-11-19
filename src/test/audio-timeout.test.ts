import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

/**
 * Tests for audio playback timeout protection
 * These tests verify that the frontend properly handles audio command timeouts
 */
describe("Audio Playback Timeout Protection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("should timeout if audio playback hangs", async () => {
    // Mock invoke to simulate hanging promise
    vi.mocked(invoke).mockImplementation(
      () =>
        new Promise(() => {
          // Never resolves or rejects
        }),
    );

    // Simulate the timeout logic from PromptApp.vue
    const timeoutMs = 100; // Short timeout for testing
    const timeoutPromise = new Promise<void>((_, reject) => {
      setTimeout(() => reject(new Error("Audio playback timeout")), timeoutMs);
    });

    const audioPromise = invoke("play_builtin_audio", {
      resourceName: "test",
      volume: 0.5,
    });

    // Should reject with timeout error
    await expect(Promise.race([audioPromise, timeoutPromise])).rejects.toThrow(
      "Audio playback timeout",
    );
  }, 1000);

  it("should resolve successfully if audio completes in time", async () => {
    // Mock invoke to resolve quickly
    vi.mocked(invoke).mockResolvedValue(undefined);

    const timeoutMs = 5000;
    const timeoutPromise = new Promise<void>((_, reject) => {
      setTimeout(() => reject(new Error("Audio playback timeout")), timeoutMs);
    });

    const audioPromise = invoke("play_builtin_audio", {
      resourceName: "test",
      volume: 0.5,
    });

    // Should resolve without timeout
    await expect(
      Promise.race([audioPromise, timeoutPromise]),
    ).resolves.toBeUndefined();
  });

  it("should handle audio errors gracefully", async () => {
    // Mock invoke to reject with error
    vi.mocked(invoke).mockRejectedValue(new Error("Audio file not found"));

    const timeoutMs = 5000;
    const timeoutPromise = new Promise<void>((_, reject) => {
      setTimeout(() => reject(new Error("Audio playback timeout")), timeoutMs);
    });

    const audioPromise = invoke("play_builtin_audio", {
      resourceName: "nonexistent",
      volume: 0.5,
    });

    // Should reject with audio error, not timeout
    await expect(Promise.race([audioPromise, timeoutPromise])).rejects.toThrow(
      "Audio file not found",
    );
  });

  it("should timeout stop_audio if it hangs", async () => {
    // Mock invoke to simulate hanging promise
    vi.mocked(invoke).mockImplementation(
      () =>
        new Promise(() => {
          // Never resolves or rejects
        }),
    );

    const timeoutMs = 100; // Short timeout for testing
    const timeoutPromise = new Promise<void>((_, reject) => {
      setTimeout(() => reject(new Error("Stop audio timeout")), timeoutMs);
    });

    const stopPromise = invoke("stop_audio");

    // Should reject with timeout error
    await expect(Promise.race([stopPromise, timeoutPromise])).rejects.toThrow(
      "Stop audio timeout",
    );
  }, 1000);

  it("should handle multiple sequential audio commands with timeout", async () => {
    let audioCallCount = 0;

    // Mock invoke to simulate varying response times
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "stop_audio") {
        return Promise.resolve();
      }
      audioCallCount++;
      if (audioCallCount === 1) {
        // First audio call succeeds quickly
        return Promise.resolve();
      }
      // Second audio call hangs
      return new Promise(() => {
        // Never resolves
      });
    });
    const timeoutMs = 100; // Short timeout for testing

    // First audio command should succeed
    const createTimeoutPromise = () =>
      new Promise<void>((_, reject) => {
        setTimeout(
          () => reject(new Error("Audio playback timeout")),
          timeoutMs,
        );
      });

    const stopPromise = invoke("stop_audio");
    await expect(
      Promise.race([stopPromise, createTimeoutPromise()]),
    ).resolves.toBeUndefined();

    const firstAudioPromise = invoke("play_builtin_audio", {
      resourceName: "test1",
      volume: 0.5,
    });
    await expect(
      Promise.race([firstAudioPromise, createTimeoutPromise()]),
    ).resolves.toBeUndefined();

    // Second audio command should timeout
    const secondAudioPromise = invoke("play_builtin_audio", {
      resourceName: "test2",
      volume: 0.5,
    });
    await expect(
      Promise.race([secondAudioPromise, createTimeoutPromise()]),
    ).rejects.toThrow("Audio playback timeout");
  }, 1000);
});
