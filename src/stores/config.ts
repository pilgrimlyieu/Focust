import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { type LocaleKey, setI18nLocale } from "@/i18n";
import type {
  AppConfig,
  AttentionSettings,
  LongBreakSettings,
  MiniBreakSettings,
  ScheduleSettings,
} from "@/types";
import {
  createAllDayTimeRange,
  createDefaultTheme,
  createNoAudio,
  createSuggestionsSettings,
} from "@/types";
import { getErrorMessage } from "@/utils/handleError";
import { safeClone } from "@/utils/safeClone";

// Re-export types
export type {
  AppConfig,
  AttentionSettings,
  LongBreakSettings,
  MiniBreakSettings,
  ScheduleSettings,
};

/** UI theme mode type */
export type ThemeMode = "light" | "dark" | "system";

/**
 * Get the next available ID given a list of existing IDs
 * @param {number[]} existing Array of existing IDs
 * @returns {number} Next available ID
 */
function nextId(existing: number[]): number {
  const max = existing.length ? Math.max(...existing) : 0;
  return max + 1;
}

/** Configuration store for managing application settings */
export const useConfigStore = defineStore("config", () => {
  const loading = ref(false); // Loading state
  const saving = ref(false); // Saving state
  const original = ref<AppConfig | null>(null); // Original loaded config
  const draft = ref<AppConfig | null>(null); // Editable draft config
  const error = ref<string | null>(null); // Error message

  /** Check if draft differs from original */
  const isDirty = computed(() => {
    if (!original.value || !draft.value) {
      return false;
    }
    return JSON.stringify(original.value) !== JSON.stringify(draft.value);
  });

  /**
   * Apply theme to DOM
   * @param {string} mode Theme mode to apply
   */
  function applyTheme(mode: string) {
    const html = document.documentElement;

    if (mode === "system") {
      // Check if matchMedia is available
      if (typeof window.matchMedia === "function") {
        const prefersDark = window.matchMedia(
          "(prefers-color-scheme: dark)",
        ).matches;
        html.setAttribute("data-theme", prefersDark ? "dark" : "light");
      } else {
        // Fallback to light theme
        html.setAttribute("data-theme", "light");
      }
    } else {
      html.setAttribute("data-theme", mode);
    }
  }

  /**
   * Set the theme mode
   * @param {ThemeMode} mode Theme mode to set
   */
  function setThemeMode(mode: ThemeMode) {
    if (!draft.value) return;

    draft.value.themeMode = mode;
    applyTheme(mode);

    // Setup system theme change listener when in system mode
    if (mode === "system" && typeof window.matchMedia === "function") {
      const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      const handler = () => applyTheme("system");
      mediaQuery.addEventListener("change", handler);
    }
  }

  /**
   * Load configuration from backend
   */
  async function load() {
    loading.value = true;
    error.value = null;
    try {
      const cfg = await invoke<AppConfig>("get_config");
      original.value = safeClone(cfg);
      draft.value = safeClone(cfg);
      setI18nLocale(cfg.language);
      applyTheme(cfg.themeMode);
    } catch (err) {
      console.error("Failed to load config", err);
      error.value = getErrorMessage(err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Apply configuration to store
   * @param {AppConfig} raw Raw configuration object
   */
  function applyConfig(raw: AppConfig) {
    original.value = safeClone(raw);
    draft.value = safeClone(raw);
    setI18nLocale(raw.language);
    applyTheme(raw.themeMode);
  }

  /**
   * Save the draft configuration to backend
   */
  async function save() {
    if (!draft.value) {
      return;
    }
    saving.value = true;
    error.value = null;
    try {
      await invoke("save_config", { config: draft.value });
      original.value = safeClone(draft.value);
    } catch (err) {
      console.error("Failed to save config", err);
      error.value = getErrorMessage(err);
      throw err;
    } finally {
      saving.value = false;
    }
  }

  /**
   * Reset draft to original configuration
   */
  function resetDraft() {
    if (original.value) {
      draft.value = safeClone(original.value);
      setI18nLocale(original.value.language);
      applyTheme(original.value.themeMode);
    }
  }

  /**
   * Ensure draft configuration is loaded
   * @returns {AppConfig} Draft configuration
   */
  function ensureDraft(): AppConfig {
    if (!draft.value) {
      throw new Error("Config not loaded yet");
    }
    return draft.value;
  }

  /**
   * Get raw draft configuration
   * @returns {AppConfig | null} Draft configuration or null if not loaded
   */
  function getRawDraft(): AppConfig | null {
    if (!draft.value) {
      return null;
    }
    return draft.value;
  }

  /**
   * Set application language
   * @param {LocaleKey} locale Locale string to set
   */
  function setLanguage(locale: LocaleKey) {
    const cfg = ensureDraft();
    cfg.language = locale;
    setI18nLocale(locale);
  }

  /**
   * Add a new schedule with default settings
   */
  function addSchedule() {
    // See `/src-tauri/src/core/schedule.rs` for defaults
    const cfg = ensureDraft();
    const miniId = nextId(cfg.schedules.map((s) => s.miniBreaks.id));
    const longId = nextId(cfg.schedules.map((s) => s.longBreaks.id));
    cfg.schedules.push({
      daysOfWeek: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
      enabled: true,
      longBreaks: {
        afterMiniBreaks: 4,
        audio: createNoAudio(),
        durationS: 300,
        enabled: true,
        id: longId,
        maxPostponeCount: 2,
        postponedS: 300,
        strictMode: false,
        suggestions: createSuggestionsSettings(),
        theme: createDefaultTheme(),
      },
      miniBreaks: {
        audio: createNoAudio(),
        durationS: 20,
        enabled: true,
        id: miniId,
        intervalS: 1200,
        maxPostponeCount: 2,
        postponedS: 300,
        strictMode: false,
        suggestions: createSuggestionsSettings(),
        theme: createDefaultTheme(),
      },
      name: `New Schedule (${miniId})`, // Use miniId to differentiate
      notificationBeforeS: 10,
      timeRange: createAllDayTimeRange(),
    });
  }

  /**
   * Remove a schedule by its mini break ID
   * @param {number} id Mini break ID of the schedule to remove
   */
  function removeSchedule(id: number) {
    const cfg = ensureDraft();
    cfg.schedules = cfg.schedules.filter((s) => s.miniBreaks.id !== id);
  }

  /**
   * Duplicate a schedule by its mini break ID
   * @param {number} scheduleId Mini break ID of the schedule to duplicate
   */
  function duplicateSchedule(scheduleId: number) {
    if (!draft.value) {
      return;
    }
    const target = draft.value.schedules.find(
      (s) => s.miniBreaks.id === scheduleId,
    );
    if (!target) {
      return;
    }
    const clone = safeClone(target);
    clone.name = `${clone.name} copy`;
    clone.miniBreaks.id = nextId(
      draft.value.schedules.map((s) => s.miniBreaks.id),
    );
    clone.longBreaks.id = nextId(
      draft.value.schedules.map((s) => s.longBreaks.id),
    );
    clone.enabled = false;
    draft.value.schedules.push(clone);
  }

  /**
   * Add a new attention with default settings
   */
  function addAttention() {
    // See `/src-tauri/src/core/schedule.rs` for defaults
    const cfg = ensureDraft();
    const id = nextId(cfg.attentions.map((a) => a.id));
    cfg.attentions.push({
      daysOfWeek: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
      durationS: 5,
      enabled: true,
      id,
      message: "This is an attention reminder.",
      name: "New attention",
      theme: createDefaultTheme(),
      times: [],
      title: "Attention Reminder",
    });
  }

  /**
   * Remove an attention by its ID
   * @param {number} id ID of the attention to remove
   */
  function removeAttention(id: number) {
    const cfg = ensureDraft();
    cfg.attentions = cfg.attentions.filter((a) => a.id !== id);
  }

  return {
    addAttention,
    addSchedule,
    applyConfig,
    draft,
    duplicateSchedule,
    error,
    getRawDraft,
    isDirty,
    load,
    loading,
    original,
    removeAttention,
    removeSchedule,
    resetDraft,
    save,
    saving,
    setLanguage,
    setThemeMode,
  };
});
