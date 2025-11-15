import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { ref } from "vue";
import type { SchedulerStatus } from "@/types";

/** Scheduler store to handle status updates */
export const useSchedulerStore = defineStore("scheduler", () => {
  const initialized = ref(false); // Initialization flag
  const schedulerPaused = ref(false); // Scheduler paused state
  const schedulerStatus = ref<SchedulerStatus | null>(null); // Scheduler status

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
    init,
    schedulerPaused,
    schedulerStatus,
    setPaused,
  };
});
