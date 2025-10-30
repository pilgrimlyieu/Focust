import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  availableMonitors,
  currentMonitor,
  type Monitor,
} from "@tauri-apps/api/window";
import { defineStore } from "pinia";
import { ref } from "vue";
import { useConfigStore } from "@/stores/config";
import { useSuggestionsStore } from "@/stores/suggestions";
import type { AppConfig } from "@/types/generated/AppConfig";
import type { AudioSettings } from "@/types/generated/AudioSettings";
import type { BackgroundSource } from "@/types/generated/BackgroundSource";
import type { EventKind } from "@/types/generated/EventKind";
import type { SchedulerStatus } from "@/types/generated/SchedulerStatus";
import type { ThemeSettings } from "@/types/generated/ThemeSettings";
import {
  isAttention,
  isLongBreak,
  isMiniBreak,
  isNotificationKind,
} from "@/types/guards";

/** Break kind type */
export type BreakKind = "mini" | "long" | "attention";

/** Resolved background for break window */
export interface ResolvedBackground {
  type: "solid" | "image";
  value: string;
}

/** Scheduler break payload sent to break window */
export interface BreakPayload {
  id: number;
  kind: BreakKind;
  title: string;
  messageKey: string; // i18n key of break message
  message?: string; // Custom message for Attention
  duration: number;
  strictMode: boolean;
  theme: ThemeSettings;
  background: ResolvedBackground;
  suggestion?: string;
  audio?: AudioSettings;
  allScreens: boolean;
  scheduleName?: string;
  postponeShortcut: string;
}

/**
 * Resolve background source to actual background for break window
 * @param {BackgroundSource} source Background source from theme settings
 * @returns {Promise<ResolvedBackground>} Resolved background
 */
async function resolveBackground(
  source: BackgroundSource,
): Promise<ResolvedBackground> {
  if ("Solid" in source) {
    return { type: "solid", value: source.Solid };
  } else if ("ImagePath" in source) {
    return {
      type: "image",
      value: convertFileSrc(source.ImagePath),
    };
  } else if ("ImageFolder" in source) {
    try {
      const image = await invoke<string | null>("pick_background_image", {
        folder: source.ImageFolder,
      });
      if (image) {
        return { type: "image", value: convertFileSrc(image) };
      }
    } catch (err) {
      console.warn("Failed to pick background image", err);
    }
  }
  return { type: "solid", value: "#1f2937" }; // Default solid color
}

// Helper type for break extraction result
type BreakExtractionResult = {
  id: number;
  kind: BreakKind;
  duration: number;
  strictMode: boolean;
  theme: ThemeSettings;
  suggestion?: string; // Single suggestion text (changed from suggestions array)
  audio: AudioSettings | undefined;
  title: string;
  messageKey: string; // i18n key
  message?: string; // Custom message (for Attention)
  scheduleName: string | undefined;
};

/**
 * Extract break information from scheduler event payload
 * @param {EventKind} payload EventKind payload
 * @param {AppConfig} config Application configuration
 */
function extractBreakInfo(
  payload: EventKind,
  config: AppConfig,
  suggestionsStore: ReturnType<typeof useSuggestionsStore>,
): BreakExtractionResult | null {
  if (isMiniBreak(payload)) {
    const id = payload.MiniBreak;
    console.log("[Scheduler] Looking for mini break with id:", id);
    const schedule = config.schedules.find((s) => s.miniBreaks.id === id);
    console.log("[Scheduler] Found schedule:", schedule);
    if (!schedule) {
      console.error("[Scheduler] No schedule found for mini break id:", id);
      return null;
    }
    const mini = schedule.miniBreaks;
    // Only sample suggestion if enabled
    const suggestion = mini.suggestions.show
      ? suggestionsStore.sample(config.language)
      : undefined;
    console.log("[Scheduler] Mini break suggestion sampled:", suggestion);
    return {
      audio: mini.audio,
      duration: Math.round(mini.durationS),
      id: mini.id,
      kind: "mini",
      messageKey: "break.miniBreakMessage",
      scheduleName: schedule.name,
      strictMode: mini.strictMode,
      suggestion: suggestion, // Single suggestion, undefined if none
      theme: mini.theme,
      title: schedule.name,
    };
  } else if (isLongBreak(payload)) {
    const id = payload.LongBreak;
    console.log("[Scheduler] Looking for long break with id:", id);
    const schedule = config.schedules.find((s) => s.longBreaks.id === id);
    console.log("[Scheduler] Found schedule:", schedule);
    if (!schedule) {
      console.error("[Scheduler] No schedule found for long break id:", id);
      return null;
    }
    const long = schedule.longBreaks;
    // Only sample suggestion if enabled
    const suggestion = long.suggestions.show
      ? suggestionsStore.sample(config.language)
      : undefined;
    return {
      audio: long.audio,
      duration: Math.round(long.durationS),
      id: long.id,
      kind: "long",
      messageKey: "break.longBreakMessage",
      scheduleName: schedule.name,
      strictMode: long.strictMode,
      suggestion: suggestion,
      theme: long.theme,
      title: schedule.name,
    };
  } else if (isAttention(payload)) {
    const id = payload.Attention;
    const attention = config.attentions.find((a) => a.id === id);
    if (!attention) return null;
    return {
      audio: undefined,
      duration: Math.round(attention.durationS),
      id: attention.id,
      kind: "attention",
      message: attention.message || undefined, // Use custom message if provided
      messageKey: "break.attentionMessage", // Fallback i18n key
      scheduleName: undefined,
      strictMode: false,
      suggestion: undefined, // Attention doesn't use suggestions
      theme: attention.theme,
      title: attention.title,
    };
  }
  return null;
}

/**
 * Get monitor information with scaled dimensions
 * @param {Monitor | null} monitor Monitor object
 * @param {Screen} fallback Fallback screen object
 * @returns {object} Scaled monitor information
 */
function getMonitorInfo(
  monitor: Monitor | null,
  fallback: Screen,
): {
  height: number;
  width: number;
  x: number;
  y: number;
} {
  if (!monitor) {
    return {
      height: fallback.height,
      width: fallback.width,
      x: 0,
      y: 0,
    };
  }
  const scaleFactor = monitor.scaleFactor;
  return {
    height: monitor.size.height / scaleFactor,
    width: monitor.size.width / scaleFactor,
    x: monitor.position.x / scaleFactor,
    y: monitor.position.y / scaleFactor,
  };
}

/**
 * Calculate window options for a given monitor and window size
 * @param {Monitor | null} monitor Monitor object
 * @param {number} windowSize Window size ratio (0.0 to 1.0)
 * @returns {object} Calculated window options
 */
function getWindowOptionsForMonitor(
  monitor: Monitor | null,
  windowSize: number,
): {
  alwaysOnTop: boolean;
  decorations: boolean;
  focus: boolean;
  fullscreen: boolean;
  skipTaskbar: boolean;
  transparent: boolean;
  width?: number;
  height?: number;
  x: number;
  y: number;
} {
  const isFullscreen = windowSize >= 1.0;
  const monitorInfo = getMonitorInfo(monitor, globalThis.screen);
  const {
    width: monitorWidth,
    height: monitorHeight,
    x: monitorX,
    y: monitorY,
  } = monitorInfo;
  const windowWidth = isFullscreen
    ? monitorWidth
    : Math.floor(monitorWidth * windowSize);
  const windowHeight = isFullscreen
    ? monitorHeight
    : Math.floor(monitorHeight * windowSize);
  return {
    alwaysOnTop: true,
    decorations: false,
    focus: true,
    fullscreen: isFullscreen,
    skipTaskbar: true,
    transparent: true,
    ...(isFullscreen
      ? {
          x: monitorX,
          y: monitorY,
        }
      : {
          height: windowHeight,
          width: windowWidth,
          x: monitorX + Math.floor((monitorWidth - windowWidth) / 2),
          y: monitorY + Math.floor((monitorHeight - windowHeight) / 2),
        }),
  };
}

/** Scheduler store for managing break windows and events */
export const useSchedulerStore = defineStore("scheduler", () => {
  const initialized = ref(false); // Initialization flag
  const activeBreakLabel = ref<string | null>(null); // Active break window label
  const activeLabels = ref<string[]>([]); // All active break window labels
  const activePayload = ref<BreakPayload | null>(null); // Active break payload
  const schedulerPaused = ref(false); // Scheduler paused state
  const schedulerStatus = ref<SchedulerStatus | null>(null); // Scheduler status

  /**
   * Initialize scheduler store and set up event listeners
   */
  async function init() {
    if (initialized.value) {
      return;
    }
    initialized.value = true;

    await listen<EventKind>("scheduler-event", (event) => {
      handleSchedulerEvent(event.payload);
    });

    await listen("break-finished", () => {
      closeActiveBreak();
    });

    // Listen for scheduler status updates
    await listen<SchedulerStatus>("scheduler-status", (event) => {
      console.log("[Scheduler] Status update received:", event.payload);
      schedulerStatus.value = event.payload;
      schedulerPaused.value = event.payload.paused;
    });

    // Request initial status after listeners are set up
    try {
      await invoke("request_scheduler_status");
      console.log("[Scheduler] Requested initial status");
    } catch (err) {
      console.error("[Scheduler] Failed to request initial status:", err);
    }

    // Listen for break window ready signal
    await listen<{ label: string }>("break-window-ready", async (event) => {
      console.log(
        "[Scheduler] Break window ready signal received:",
        event.payload,
      );
      const label = event.payload.label;

      // Check if this is the active break window
      if (
        activePayload.value &&
        (label === activeBreakLabel.value || activeLabels.value.includes(label))
      ) {
        console.log(
          "[Scheduler] Sending payload to break window:",
          activePayload.value,
        );
        try {
          await emit("break-start", activePayload.value);
          console.log("[Scheduler] Payload sent successfully");
        } catch (err) {
          console.error("[Scheduler] Failed to send payload:", err);
        }
      } else {
        console.warn(
          "[Scheduler] Received ready signal from unknown window:",
          label,
        );
      }
    });
  }

  /**
   * Handle scheduler event and open break window if needed
   * @param {EventKind} payload Scheduler event payload
   */
  async function handleSchedulerEvent(payload: EventKind) {
    console.log("[Scheduler] Received scheduler event:", payload);

    if (isNotificationKind(payload)) {
      const id = Object.values(payload.Notification)[0];
      console.info("Break notification", id);
      return;
    }

    const configStore = useConfigStore();
    const suggestionsStore = useSuggestionsStore();
    const config = configStore.draft ?? configStore.original;
    if (!config) {
      console.warn("Config not loaded yet, ignoring scheduler event");
      return;
    }

    console.log("[Scheduler] Config loaded, processing event...");
    console.log("[Scheduler] Available schedules:", config.schedules);

    // Extract break information based on event type
    const breakInfo = extractBreakInfo(payload, config, suggestionsStore);
    if (!breakInfo) {
      return;
    }

    const background = await resolveBackground(breakInfo.theme.background);

    console.log("[Scheduler] Creating break payload...");

    const breakPayload: BreakPayload = {
      allScreens: config.allScreens,
      audio: breakInfo.audio,
      background,
      duration: breakInfo.duration,
      id: breakInfo.id,
      kind: breakInfo.kind,
      message: breakInfo.message,
      messageKey: breakInfo.messageKey,
      postponeShortcut: config.postponeShortcut || "P", // Default "P" if not set
      scheduleName: breakInfo.scheduleName,
      strictMode: breakInfo.strictMode,
      suggestion: breakInfo.suggestion,
      theme: breakInfo.theme,
      title: breakInfo.title,
    };

    console.log("[Scheduler] Break payload created:", breakPayload);
    await openBreakWindow(breakPayload);
  }

  /**
   * Open break window with given payload
   * @param {BreakPayload} payload Break payload to send to window
   */
  async function openBreakWindow(payload: BreakPayload) {
    console.log("[Scheduler] Opening break window with payload:", payload);
    await closeActiveBreak();

    const label = `break-${Date.now()}`;
    activeBreakLabel.value = label;
    activePayload.value = payload;
    activeLabels.value = [label];

    console.log("[Scheduler] Window label:", label);

    // Get config for window size
    const configStore = useConfigStore();
    const config = configStore.draft ?? configStore.original;

    // See `src-tauri/src/config/models.rs`
    const windowSize = config?.windowSize ?? 0.8;

    const monitors = payload.allScreens ? await availableMonitors() : [];

    console.log("[Scheduler] Monitors:", monitors.length);

    if (monitors.length <= 1) {
      const targetMonitor = await currentMonitor();
      const windowOptions = {
        url: `/index.html?view=break&label=${label}`,
        ...getWindowOptionsForMonitor(targetMonitor, windowSize),
      };
      console.log("[Scheduler] Creating window with options:", windowOptions);
      const breakWindow = new WebviewWindow(label, windowOptions);
      breakWindow.once("tauri://error", (e) => {
        console.error("[Scheduler] Window creation error:", e);
      });
      breakWindow.once("tauri://created", () => {
        console.log("[Scheduler] Window created successfully");
      });
    } else {
      monitors.forEach((monitor, index) => {
        const childLabel = `${label}-${index}`;
        const win = new WebviewWindow(childLabel, {
          url: `/index.html?view=break&label=${childLabel}`,
          ...getWindowOptionsForMonitor(monitor, windowSize),
        });
        win.once("tauri://error", (e) => {
          console.error("[Scheduler] Multi-monitor window creation error:", e);
        });
        win.once("tauri://created", () => {
          console.log(
            "[Scheduler] Multi-monitor window created successfully:",
            childLabel,
          );
        });
        activeLabels.value.push(childLabel);
      });
    }
  }

  /**
   * Close active break window(s)
   */
  async function closeActiveBreak() {
    if (!activeBreakLabel.value) {
      return;
    }
    for (const label of activeLabels.value) {
      const win = await WebviewWindow.getByLabel(label);
      await win?.close();
    }
    activeBreakLabel.value = null;
    activePayload.value = null;
    activeLabels.value = [];
  }

  /**
   * Set scheduler paused state
   * @param {boolean} paused Paused state
   */
  function setPaused(paused: boolean) {
    schedulerPaused.value = paused;
  }

  return {
    activePayload,
    closeActiveBreak,
    handleSchedulerEvent,
    init,
    openBreakWindow,
    schedulerPaused,
    schedulerStatus,
    setPaused,
  };
});
