<script setup lang="ts">
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from "vue";
import { useI18n } from "vue-i18n";
import { setI18nLocale } from "@/i18n";
import type { AudioSettings, PromptPayload, SchedulerEvent } from "@/types";
import {
  createAttentionEvent,
  createLongBreakEvent,
  createMiniBreakEvent,
  isBuiltinAudio,
  isFilePathAudio,
  isNoAudio,
  isResolvedImageBackground,
  isResolvedSolidBackground,
} from "@/types";

const { t } = useI18n();

const payload = ref<PromptPayload | null>(null);
const remaining = ref(0);
const intervalId = ref<number | null>(null);
const isClosing = ref(false);
const isRendered = ref(false);

/**
 * Start the countdown timer for the break.
 * @param {number} duration The duration of the break in seconds.
 */
const startTimer = (duration: number) => {
  remaining.value = Math.max(0, duration);
  if (intervalId.value) {
    clearInterval(intervalId.value);
  }
  intervalId.value = window.setInterval(() => {
    if (remaining.value > 0) {
      remaining.value -= 1;
    } else {
      clearInterval(intervalId.value ?? undefined);
      intervalId.value = null;
      // Auto-finish when timer reaches 0
      void finishPrompt(true);
    }
  }, 1000);
};

/**
 * Format time in seconds to MM:SS string.
 * @param {number} value Time in seconds.
 */
const formatTime = (value: number) => {
  const minutes = Math.floor(value / 60);
  const seconds = value % 60;
  return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
};

const backgroundStyle = computed(() => {
  const current = payload.value;
  if (!current) return {};
  if (isResolvedSolidBackground(current.background)) {
    return {
      backgroundColor: current.background.value,
      backgroundImage: "none",
    };
  }
  const imageUrl = convertFileSrc(current.background.value);
  return {
    backgroundImage: `url("${imageUrl}")`,
    backgroundPosition: "center",
    backgroundSize: "cover",
  };
});

const overlayStyle = computed(() => {
  const current = payload.value;
  if (!current) return {};
  return {
    backdropFilter: `blur(${current.theme.blurRadius}px)`,
    backgroundColor: `rgba(15, 23, 42, ${1 - current.theme.opacity})`,
    color: current.theme.textColor,
    fontFamily: current.theme.fontFamily,
    fontSize: `${current.theme.fontSize}px`,
  };
});

const progress = computed(() => {
  if (!payload.value || payload.value.duration === 0) {
    return 0;
  }
  return Math.min(
    100,
    Math.max(0, (remaining.value / payload.value.duration) * 100),
  );
});

const elapsed = computed(() => 100 - progress.value);

const controlsDisabled = computed(() => payload.value?.strictMode ?? false);

const isAttention = computed(() => payload.value?.kind === "attention");

const remainingPostpones = computed(() => {
  if (!payload.value?.maxPostponeCount) return null;
  return payload.value.maxPostponeCount - payload.value.postponeCount;
});

const canPostpone = computed(() => {
  if (controlsDisabled.value || isAttention.value) return false;
  if (!payload.value?.maxPostponeCount) return true; // No limit
  return payload.value.postponeCount < payload.value.maxPostponeCount;
});

/** Stop the active audio playback */
const stopAudio = async () => {
  await invoke("stop_audio").catch((err) => {
    console.warn("Failed to stop audio via backend", err);
  });
};

/**
 * Play the break audio based on the provided settings.
 * @param {AudioSettings?} settings The audio settings.
 */
const playAudio = async (settings?: AudioSettings | null) => {
  await stopAudio();
  if (!settings || isNoAudio(settings)) return;

  try {
    if (isBuiltinAudio(settings)) {
      const name = settings.builtinName;
      await invoke("play_builtin_audio", {
        resourceName: name,
        volume: settings.volume,
      });
    } else if (isFilePathAudio(settings)) {
      const path = settings.filePath;
      await invoke("play_audio", {
        path,
        volume: settings.volume,
      });
    }
  } catch (err) {
    console.warn("Failed to play break audio", err);
  }
};

const preloadBackground = async (data: PromptPayload): Promise<void> => {
  if (isResolvedImageBackground(data.background)) {
    return new Promise((resolve) => {
      const img = new Image();
      img.onload = () => resolve();
      img.onerror = () => resolve();
      setTimeout(() => resolve(), 1000);
      img.src = convertFileSrc(data.background.value);
    });
  }
};

/**
 * Handle incoming prompt payload and start the break.
 * @param {PromptPayload} data The prompt payload data.
 */
const handlePayload = async (data: PromptPayload) => {
  console.log("[PromptApp] Handling payload:", data);
  console.log("[PromptApp] Suggestion field:", data.suggestion);
  isClosing.value = false;
  payload.value = data;

  await preloadBackground(data);

  // Wait for next tick to ensure DOM is updated
  await nextTick();

  // Start timer after rendering
  startTimer(data.duration);

  // Only play audio in the primary window (first monitor, index 0)
  const currentWindow = getCurrentWindow();
  const windowLabel = currentWindow.label;
  const isPrimaryWindow = windowLabel.endsWith("-0");

  if (isPrimaryWindow) {
    console.log("[PromptApp] Playing break audio (primary window)");
    playAudio(data.audio).catch((err) => {
      console.warn("[PromptApp] Audio playback error:", err);
    });
  } else {
    console.log("[PromptApp] Skipping audio playback (secondary window)");
  }

  // Show window after everything is ready
  console.log("[PromptApp] Content rendered, showing window...");

  try {
    await currentWindow.show();
    // await currentWindow.setFocus();
    isRendered.value = true;
    console.log("[PromptApp] Window shown successfully");
  } catch (err) {
    console.error("[PromptApp] Failed to show window:", err);
  }
};

const finishPrompt = async (isAutoFinish = false) => {
  console.log(
    `[PromptApp] finishPrompt called (isAutoFinish: ${isAutoFinish})`,
  );

  // Strict mode: cannot finish break early manually, but allow auto-finish
  if (!isAutoFinish && controlsDisabled.value) {
    console.log("[PromptApp] finishPrompt blocked by strict mode");
    return;
  }

  if (isClosing.value) {
    console.warn("[PromptApp] finishPrompt blocked - already closing");
    return;
  }

  console.log("[PromptApp] Starting finishPrompt sequence");
  isClosing.value = true;

  if (intervalId.value) {
    clearInterval(intervalId.value);
    intervalId.value = null;
    console.log("[PromptApp] Timer cleared");
  }

  stopAudio();
  console.log("[PromptApp] Audio stopped");

  // Notify backend that break has finished (so it can update timers)
  if (payload.value) {
    try {
      const event = constructSchedulerEvent(payload.value);
      console.log("[PromptApp] Notifying backend: prompt_finished", event);
      await invoke("prompt_finished", { event });
      console.log("[PromptApp] Backend notified successfully");
    } catch (err) {
      console.error(
        "[PromptApp] Failed to notify backend about break finish:",
        err,
      );
    }
  }

  // Close all prompt windows
  try {
    const params = new URLSearchParams(window.location.search);
    const payloadId = params.get("payloadId");
    if (payloadId) {
      console.log(
        "[PromptApp] Calling close_all_prompt_windows for payload:",
        payloadId,
      );
      await invoke("close_all_prompt_windows", { payloadId });
      console.log("[PromptApp] close_all_prompt_windows completed");
    } else {
      // Fallback: close only current window
      console.warn(
        "[PromptApp] No payloadId found, closing current window only",
      );
      const window = getCurrentWindow();
      await window.close();
    }
  } catch (err) {
    console.error("[PromptApp] Failed to close windows:", err);
  }
};

/**
 * Construct a SchedulerEvent from the current prompt payload using factory functions
 * @param {PromptPayload} payload The prompt payload
 * @returns {SchedulerEvent} SchedulerEvent object
 */
const constructSchedulerEvent = (payload: PromptPayload): SchedulerEvent => {
  switch (payload.kind) {
    case "mini":
      return createMiniBreakEvent(payload.id);
    case "long":
      return createLongBreakEvent(payload.id);
    case "attention":
      return createAttentionEvent(payload.id);
    default:
      throw new Error(`Unknown break kind: ${payload.kind}`);
  }
};

const postponeBreak = async () => {
  // Check if postpone is allowed (button should already be disabled, but double-check)
  if (!payload.value || !canPostpone.value) return;
  await invoke("postpone_break");
  await finishPrompt();
};

const handleKeydown = (event: KeyboardEvent) => {
  const isCtrlOrCmd = event.ctrlKey || event.metaKey;
  const key = event.key.toLowerCase();

  const blockedKeys = ["f5", "f11"];
  const blockedCtrlKeys = ["a", "c", "v", "x", "r", "w", "t", "n"];

  if (blockedKeys.includes(key)) {
    event.preventDefault();
    return;
  }

  if (isCtrlOrCmd && blockedCtrlKeys.includes(key)) {
    event.preventDefault();
    return;
  }

  if (controlsDisabled.value) {
    if (key === "enter" || key === "escape") {
      event.preventDefault();
    }
    return;
  }

  const postponeKey =
    payload.value?.postponeShortcut?.split("+").pop()?.toLowerCase() || "p";

  if (key === postponeKey) {
    event.preventDefault();
    void postponeBreak();
    return;
  }

  if (key === "enter" || key === " " || key === "spacebar") {
    event.preventDefault();
    void finishPrompt();
  }
};

watch(payload, (next) => {
  document.title = next ? `${next.title} — Focust` : "Focust";
});

onMounted(async () => {
  console.log("[PromptApp] Component mounted");
  window.addEventListener("keydown", handleKeydown);

  // Get payloadId from URL
  const params = new URLSearchParams(window.location.search);
  const payloadId = params.get("payloadId");
  console.log("[PromptApp] Payload ID from URL:", payloadId);

  if (!payloadId) {
    console.error("[PromptApp] No payloadId found in URL");
    return;
  }

  // Fetch payload from backend immediately
  try {
    console.log("[PromptApp] Fetching payload from backend...");
    const fetchedPayload = await invoke<PromptPayload>("get_prompt_payload", {
      payloadId,
    });
    console.log("[PromptApp] Payload fetched successfully:", fetchedPayload);

    // Set the language from payload before handling payload
    if (fetchedPayload.language) {
      console.log("[PromptApp] Setting language to:", fetchedPayload.language);
      setI18nLocale(fetchedPayload.language);
    }

    await handlePayload(fetchedPayload);
  } catch (err) {
    console.error("[PromptApp] Failed to fetch payload from backend:", err);
  }
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeydown);
  if (intervalId.value) {
    clearInterval(intervalId.value);
  }
  stopAudio();
  document.title = "Focust";
});

defineExpose({
  backgroundStyle,
  controlsDisabled,
  elapsed,
  finishBreak: finishPrompt,
  formatTime,
  isAttention,
  overlayStyle,
  postponeBreak,
  progress,
  t,
});
</script>

<template>
  <div class="break-app flex min-h-screen flex-col overflow-hidden"
    :class="{ 'is-strict': controlsDisabled, 'is-rendered': isRendered }" :style="backgroundStyle" @contextmenu.prevent>
    <div class="flex flex-1 items-center justify-center bg-slate-950/35 p-6">
      <div
        class="w-full max-w-3xl rounded-3xl border border-white/10 bg-white/10 p-10 shadow-2xl backdrop-blur-xl transition-all"
        :style="overlayStyle">
        <div v-if="!payload" class="flex flex-col items-center gap-3 text-center">
          <span class="loading loading-ring loading-lg" />
          <p class="text-sm opacity-70">{{ t("break.preparing") }}</p>
        </div>

        <div v-else class="space-y-8 text-center">
          <div class="space-y-2">
            <p class="text-xs uppercase tracking-[0.35em] opacity-60">
              {{ payload.scheduleName ?? (payload.kind === "attention" ? t("break.attention") : payload.kind === "long"
                ? t("schedule.longBreak") : t("schedule.miniBreak")) }}
            </p>
            <h1 class="text-4xl font-semibold">{{ payload.title }}</h1>
            <p class="text-base opacity-80">
              {{ payload.message || t(payload.messageKey) }}
            </p>
          </div>

          <div class="flex flex-col items-center gap-4">
            <div class="radial-progress text-5xl font-bold" role="progressbar" :aria-valuenow="elapsed"
              aria-valuemin="0" aria-valuemax="100" :style="`--value:${elapsed}; --size:12rem; --thickness:12px`">
              {{ formatTime(remaining) }}
            </div>
            <p class="text-xs uppercase tracking-wide"
              :class="controlsDisabled ? 'text-orange-400 font-semibold' : 'opacity-60'">
              {{ controlsDisabled ? t("break.strict") : t("break.timerLabel") }}
            </p>
          </div>

          <!-- Single suggestion display: larger font, centered -->
          <div v-if="payload.suggestion" class="space-y-3">
            <p class="text-sm uppercase tracking-wide opacity-60">{{ t("break.suggestion") }}</p>
            <p class="text-2xl text-center opacity-90 font-medium">{{ payload.suggestion }}</p>
          </div>

          <div class="flex flex-wrap justify-center gap-3">
            <button class="btn btn-success btn-wide sm:btn-lg" :disabled="controlsDisabled"
              @click="() => finishPrompt()">
              {{ isAttention ? t("break.gotIt") : t("break.resume") }}
            </button>
            <button v-if="!isAttention" class="btn btn-outline btn-wide sm:btn-lg" :disabled="!canPostpone"
              @click="postponeBreak">
              {{ t("break.postpone") }}
              <span v-if="remainingPostpones !== null" class="text-xs opacity-70">
                ({{ t("break.remainingCount", { count: remainingPostpones }) }})
              </span>
            </button>
          </div>

          <!-- Only show shortcut hint in non-strict mode -->
          <p v-if="!isAttention && !controlsDisabled" class="text-xs opacity-50">
            {{ t("break.shortcutHint", { postpone: payload.postponeShortcut }) }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>