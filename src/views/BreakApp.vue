<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
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
import type { AudioSettings } from "@/types/generated/AudioSettings";
import type { BreakPayload } from "@/types/generated/BreakPayload";
import { isBuiltinAudio, isFilePathAudio } from "@/types/guards";

const { t } = useI18n();

const payload = ref<BreakPayload | null>(null);
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
      void finishBreak(true);
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
  if (current.background.type === "solid") {
    return {
      backgroundColor: current.background.value,
      backgroundImage: "none",
    };
  }
  return {
    backgroundImage: `url(${current.background.value})`,
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
  if (!settings || settings.source === "None") return;

  try {
    if (isBuiltinAudio(settings)) {
      const name = settings.name;
      await invoke("play_builtin_audio", {
        resourceName: name,
        volume: settings.volume,
      });
    } else if (isFilePathAudio(settings)) {
      const path = settings.path;
      await invoke("play_audio", {
        path,
        volume: settings.volume,
      });
    }
  } catch (err) {
    console.warn("Failed to play break audio", err);
  }
};

/**
 * Handle incoming break payload and start the break.
 * @param {BreakPayload} data The break payload data.
 */
const handlePayload = async (data: BreakPayload) => {
  console.log("[BreakApp] Handling payload:", data);
  console.log("[BreakApp] Suggestion field:", data.suggestion);
  isClosing.value = false;
  payload.value = data;

  // Wait for next tick to ensure DOM is updated
  await nextTick();

  // Start timer after rendering
  startTimer(data.duration);

  // Play audio (non-blocking)
  console.log("[BreakApp] Playing break audio");
  playAudio(data.audio).catch((err) => {
    console.warn("[BreakApp] Audio playback error:", err);
  });

  // Show window after everything is ready
  console.log("[BreakApp] Content rendered, showing window...");

  try {
    const currentWindow = getCurrentWindow();
    await currentWindow.show();
    await currentWindow.setFocus();
    isRendered.value = true;
    console.log("[BreakApp] Window shown successfully");
  } catch (err) {
    console.error("[BreakApp] Failed to show window:", err);
  }
};

const finishBreak = async (isAutoFinish = false) => {
  // Strict mode: cannot finish break early manually, but allow auto-finish
  if (!isAutoFinish && controlsDisabled.value) return;

  if (isClosing.value) return;
  isClosing.value = true;
  if (intervalId.value) {
    clearInterval(intervalId.value);
    intervalId.value = null;
  }
  stopAudio();
  await emit("break-finished", null);
};

const postponeBreak = async () => {
  // Attention reminders cannot be postponed - they are just notifications
  if (!payload.value || controlsDisabled.value || isAttention.value) return;
  await invoke("postpone_break");
  await finishBreak();
};

const handleKeydown = (event: KeyboardEvent) => {
  // TODO: More robust way
  // Block common browser shortcuts during break
  if (
    event.ctrlKey &&
    (event.key === "a" ||
      event.key === "c" ||
      event.key === "v" ||
      event.key === "x" ||
      event.key === "r" ||
      event.key === "w" ||
      event.key === "t" ||
      event.key === "n")
  ) {
    event.preventDefault();
    return;
  }
  if (event.key === "F5" || event.key === "F11") {
    event.preventDefault();
    return;
  }

  // Handle break controls
  if (controlsDisabled.value) {
    if (event.key === "Enter" || event.key === "Escape") {
      event.preventDefault();
    }
    return;
  }

  // Postpone shortcut (from config or fallback to "P")
  const postponeKey =
    payload.value?.postponeShortcut?.split("+").pop()?.toLowerCase() || "p";
  if (event.key.toLowerCase() === postponeKey) {
    event.preventDefault();
    void postponeBreak();
    return;
  }

  // Resume/finish with Enter or Space
  if (event.key === "Enter" || event.key === " " || event.key === "Spacebar") {
    event.preventDefault();
    void finishBreak();
  }
};

/**
 * Prevent context menu (right-click) in break window
 * @param {MouseEvent} event The mouse event.
 */
const handleContextMenu = (event: MouseEvent) => {
  event.preventDefault();
};

watch(payload, (next) => {
  document.title = next ? `${next.title} — Focust` : "Focust";
});

onMounted(async () => {
  console.log("[BreakApp] Component mounted");
  window.addEventListener("keydown", handleKeydown);
  window.addEventListener("contextmenu", handleContextMenu);

  // Get payloadId from URL
  const params = new URLSearchParams(window.location.search);
  const payloadId = params.get("payloadId");
  console.log("[BreakApp] Payload ID from URL:", payloadId);

  if (!payloadId) {
    console.error("[BreakApp] No payloadId found in URL");
    return;
  }

  // Fetch payload from backend immediately
  try {
    console.log("[BreakApp] Fetching payload from backend...");
    const fetchedPayload = await invoke<BreakPayload>("get_break_payload", {
      payloadId,
    });
    console.log("[BreakApp] Payload fetched successfully:", fetchedPayload);
    await handlePayload(fetchedPayload);
  } catch (err) {
    console.error("[BreakApp] Failed to fetch payload from backend:", err);
  }
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("contextmenu", handleContextMenu);
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
  finishBreak,
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
    :class="{ 'is-strict': controlsDisabled, 'is-rendered': isRendered }" :style="backgroundStyle">
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
              {{
              payload.scheduleName ??
              (payload.kind === "attention"
              ? t("break.attention")
              : payload.kind === "long"
              ? t("schedule.longBreak")
              : t("schedule.miniBreak"))
              }}
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
              @click="() => finishBreak()">
              {{ isAttention ? t("break.gotIt") : t("break.resume") }}
            </button>
            <button v-if="!isAttention" class="btn btn-outline btn-wide sm:btn-lg" :disabled="controlsDisabled"
              @click="postponeBreak">
              {{ t("break.postpone") }}
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