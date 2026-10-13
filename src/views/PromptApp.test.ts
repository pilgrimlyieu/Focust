import { flushPromises, mount, type VueWrapper } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { i18n } from "@/i18n";
import { mockInvoke } from "@/test/setup";
import type { PromptPayload } from "@/types";
import {
  createBuiltinAudio,
  createDefaultTheme,
  createMiniBreakEvent,
  createResolvedSolidBackground,
} from "@/types";
import PromptApp from "./PromptApp.vue";

const nativeWindow = vi.hoisted(() => ({
  close: vi.fn().mockResolvedValue(undefined),
  label: "break-test-0",
  setFocus: vi.fn().mockResolvedValue(undefined),
  show: vi.fn().mockResolvedValue(undefined),
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => nativeWindow,
}));

let wrapper: VueWrapper<InstanceType<typeof PromptApp>> | undefined;
let payload: PromptPayload;
const respond = async (command: string) =>
  command === "get_prompt_payload" ? payload : undefined;

beforeEach(() => {
  vi.useFakeTimers();
  vi.clearAllMocks();
  vi.spyOn(console, "log").mockImplementation(() => {});
  vi.spyOn(console, "warn").mockImplementation(() => {});
  window.history.replaceState({}, "", "/?payloadId=test");
  payload = {
    allScreens: false,
    audio: createBuiltinAudio("gentle-bell"),
    background: createResolvedSolidBackground("#000000"),
    duration: 20,
    id: 1,
    kind: "mini",
    language: "en-US",
    maxPostponeCount: 2,
    message: null,
    messageKey: "break.miniBreakMessage",
    postponeCount: 0,
    postponeShortcut: "P",
    scheduleName: "Test",
    strictMode: false,
    suggestion: null,
    theme: createDefaultTheme(),
    title: "Break",
  };
  mockInvoke.mockImplementation(respond);
});

afterEach(async () => {
  wrapper?.unmount();
  wrapper = undefined;
  await flushPromises();
  vi.clearAllTimers();
  vi.useRealTimers();
  vi.restoreAllMocks();
  window.history.replaceState({}, "", "/");
});

async function renderPrompt() {
  wrapper = mount(PromptApp, { global: { plugins: [i18n] } });
  await flushPromises();
  return wrapper;
}

it.each(["success", "error", "timeout"])(
  "shows the prompt while handling audio %s",
  async (outcome) => {
    mockInvoke.mockImplementation((command: string) => {
      if (command === "play_builtin_audio") {
        if (outcome === "timeout") return new Promise(() => {});
        if (outcome === "error")
          return Promise.reject(new Error("Missing audio file"));
      }
      return respond(command);
    });
    await renderPrompt();
    expect(nativeWindow.show).toHaveBeenCalledOnce();
    expect(mockInvoke).toHaveBeenCalledWith("play_builtin_audio", {
      resourceName: "gentle-bell",
      volume: payload.audio?.volume,
    });
    await vi.advanceTimersByTimeAsync(5000);
    if (outcome === "success") expect(console.warn).not.toHaveBeenCalled();
    else
      expect(console.warn).toHaveBeenCalledExactlyOnceWith(
        "[PromptApp] Failed to play break audio:",
        new Error(
          outcome === "error" ? "Missing audio file" : "Audio playback timeout",
        ),
      );
  },
);

it("continues playback after the real stop timeout without delaying the window", async () => {
  mockInvoke.mockImplementation((command: string) =>
    command === "stop_audio" ? new Promise(() => {}) : respond(command),
  );
  await renderPrompt();
  expect(nativeWindow.show).toHaveBeenCalledOnce();
  expect(mockInvoke).not.toHaveBeenCalledWith(
    "play_builtin_audio",
    expect.anything(),
  );
  await vi.advanceTimersByTimeAsync(2000);
  expect(mockInvoke).toHaveBeenCalledWith(
    "play_builtin_audio",
    expect.anything(),
  );
  expect(console.warn).toHaveBeenCalledWith(
    "[PromptApp] Failed to stop audio via backend:",
    new Error("Stop audio timeout"),
  );
});

it("protects a later playback even when the first playback succeeded", async () => {
  let plays = 0;
  mockInvoke.mockImplementation((command: string) => {
    if (command === "play_builtin_audio" && ++plays === 2)
      return new Promise(() => {});
    return respond(command);
  });
  await renderPrompt();
  expect(plays).toBe(1);
  await vi.advanceTimersByTimeAsync(payload.duration * 1000);
  expect(plays).toBe(2);
  expect(console.warn).not.toHaveBeenCalled();
  await vi.advanceTimersByTimeAsync(5000);
  expect(console.warn).toHaveBeenCalledExactlyOnceWith(
    "[PromptApp] Failed to play break audio:",
    new Error("Audio playback timeout"),
  );
});

it("finishes a strict prompt exactly once when its countdown reaches zero", async () => {
  payload.duration = 1;
  payload.strictMode = true;
  payload.audio = null;
  const prompt = await renderPrompt();
  await vi.advanceTimersByTimeAsync(999);
  expect(mockInvoke).not.toHaveBeenCalledWith(
    "prompt_finished",
    expect.anything(),
  );
  await vi.advanceTimersByTimeAsync(1);
  expect(prompt.text()).toContain("00:00");
  expect(mockInvoke).toHaveBeenCalledWith("prompt_finished", {
    event: createMiniBreakEvent(1),
  });
  await vi.advanceTimersByTimeAsync(1000);
  expect(
    mockInvoke.mock.calls.filter(([command]) => command === "prompt_finished"),
  ).toHaveLength(1);
});
