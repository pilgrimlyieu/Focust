import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { PauseReason, SchedulerStatus } from "@/types";

const USER_CLEARABLE_PAUSE_REASONS = new Set<PauseReason>([
  "manual",
  "timedManual",
]);

export function isUserClearablePauseReason(reason: PauseReason) {
  return USER_CLEARABLE_PAUSE_REASONS.has(reason);
}

/** Scheduler store to handle status updates */
export const useSchedulerStore = defineStore("scheduler", () => {
  const initialized = ref(false); // Initialization flag
  const schedulerPaused = ref(false); // Scheduler paused state
  const pauseReasons = ref<PauseReason[]>([]); // Active pause reasons
  const schedulerStatus = ref<SchedulerStatus | null>(null); // Scheduler status

  // Check if paused by a user action that can be cleared from the UI.
  const hasUserPause = computed(() =>
    pauseReasons.value.some(isUserClearablePauseReason),
  );

  /** Initialize scheduler store and set up event listeners */
  async function init() {
    if (initialized.value) {
      return;
    }
    initialized.value = true;

    // Listen for scheduler status updates
    await listen<SchedulerStatus>("scheduler-status", (event) => {
      console.log("[Scheduler] Status update received:", event.payload);
      schedulerStatus.value = event.payload;
      schedulerPaused.value = event.payload.paused;
      pauseReasons.value = event.payload.pauseReasons || [];
    });

    // Request initial status after listeners are set up
    try {
      await invoke("request_break_status");
      console.log("[Scheduler] Requested initial status");
    } catch (err) {
      console.error("[Scheduler] Failed to request initial status:", err);
    }
  }

  /**
   * Set scheduler paused state
   * @param {boolean} paused Paused state
   */
  function setPaused(paused: boolean) {
    schedulerPaused.value = paused;
  }

  return {
    hasUserPause,
    init,
    pauseReasons,
    schedulerPaused,
    schedulerStatus,
    setPaused,
  };
});
