import { flushPromises, mount, type VueWrapper } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { i18n } from "@/i18n";
import { mockInvoke } from "@/test/setup";
import { createBuiltinAudio } from "@/types";
import AudioPicker from "./AudioPicker.vue";

let picker: VueWrapper<InstanceType<typeof AudioPicker>>;
beforeEach(() => {
  vi.useFakeTimers();
  vi.spyOn(console, "error").mockImplementation(() => {});
  mockInvoke.mockResolvedValue(undefined);
  picker = mount(AudioPicker, {
    global: { plugins: [i18n] },
    props: { audio: createBuiltinAudio("gentle-bell"), label: "Audio" },
  });
});
afterEach(() => {
  picker.unmount();
  vi.clearAllTimers();
  vi.useRealTimers();
  vi.restoreAllMocks();
});

async function click(action: "play" | "stop") {
  await picker
    .get(`button.${action === "play" ? "btn-primary" : "btn-ghost"}`)
    .trigger("click");
  await flushPromises();
}
const canStop = () =>
  picker.get("button.btn-ghost").attributes("disabled") === undefined;

it.each(["completed", "late success", "late error"])(
  "keeps a new preview controllable after an older %s playback",
  async (outcome) => {
    const first = Promise.withResolvers<void>();
    const second = Promise.withResolvers<void>();
    const replies = [first.promise, second.promise];
    mockInvoke.mockImplementation((command: string) =>
      command === "play_builtin_audio" ? replies.shift() : Promise.resolve(),
    );
    if (outcome === "completed") first.resolve();
    await click("play");
    await vi.advanceTimersByTimeAsync(1000);
    await click("stop");
    await click("play");
    if (outcome === "late error")
      first.reject(new Error("Old playback failed"));
    else first.resolve();
    await vi.advanceTimersByTimeAsync(3000);
    expect(canStop()).toBe(true);

    second.resolve();
    await flushPromises();
    await vi.advanceTimersByTimeAsync(2999);
    expect(canStop()).toBe(true);
    await vi.advanceTimersByTimeAsync(1);
    expect(canStop()).toBe(false);
    expect(
      mockInvoke.mock.calls.filter(([command]) => command === "stop_audio"),
    ).toHaveLength(3);
  },
);

it("does not start an old preview after its stop response arrives late", async () => {
  const firstStop = Promise.withResolvers<void>();
  mockInvoke.mockImplementationOnce(() => firstStop.promise);
  await click("play");
  await click("play");
  firstStop.resolve();
  await flushPromises();
  expect(
    mockInvoke.mock.calls.filter(
      ([command]) => command === "play_builtin_audio",
    ),
  ).toHaveLength(1);
  expect(canStop()).toBe(true);
});
