import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { LANGUAGE_FALLBACK } from "@/i18n";
import type { SuggestionsConfig } from "@/types/generated/SuggestionsConfig";

/** Suggestions store for managing suggestion configurations */
export const useSuggestionsStore = defineStore("suggestions", () => {
  const config = ref<SuggestionsConfig | null>(null); // Suggestions configuration
  const loading = ref(false); // Loading state

  const hasLoaded = computed(() => config.value !== null); // Check if config is loaded

  /**
   * Load suggestions configuration from backend
   */
  async function load() {
    loading.value = true;
    try {
      const result = await invoke<SuggestionsConfig>("get_suggestions");
      config.value = result;
      console.log("Suggestions loaded successfully");
    } catch (err) {
      console.error("Failed to load suggestions:", err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Save suggestions configuration to backend
   * @param {SuggestionsConfig} newConfig New suggestions configuration
   */
  async function save(newConfig: SuggestionsConfig) {
    loading.value = true;
    try {
      await invoke("save_suggestions", { config: newConfig });
      config.value = newConfig;
    } catch (err) {
      console.error("Failed to save suggestions:", err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Get suggestions for a specific language from backend
   * @param {string} language Language code
   * @returns {Promise<string[]>} Promise resolving to array of suggestions
   */
  async function getSuggestionsForLanguage(
    language: string,
  ): Promise<string[]> {
    try {
      return await invoke<string[]>("get_suggestions_for_language", {
        language,
      });
    } catch (err) {
      console.error("Failed to get suggestions for language:", err);
      return [];
    }
  }

  /**
   * Get suggestions synchronously for a specific language
   * @param {string} language Language code
   * @returns {string[]} Array of suggestions
   */
  function getSuggestionsSync(language: string): string[] {
    if (!config.value || !config.value.byLanguage[language]) {
      console.warn(
        `No suggestions found for language: ${language}, falling back to ${LANGUAGE_FALLBACK}`,
      );
      if (config.value?.byLanguage[LANGUAGE_FALLBACK]) {
        return config.value.byLanguage[LANGUAGE_FALLBACK].suggestions;
      }
      return [];
    }
    return config.value.byLanguage[language].suggestions;
  }

  /**
   * Sample a random suggestion for a specific language
   * @param {string} language Language code
   * @returns {string} A random suggestion
   */
  function sample(language: string): string {
    const pool = getSuggestionsSync(language);
    if (!pool.length) return "";

    return pool[Math.floor(Math.random() * pool.length)];
  }

  /**
   * Sample multiple random suggestions for a specific language
   * @param {string} language Language code
   * @param {number} count Number of suggestions to sample
   * @returns {string[]} Array of random suggestions
   */
  function sampleMany(language: string, count: number = 3): string[] {
    const pool = getSuggestionsSync(language);
    if (!pool.length) return [];

    // Fisher-Yates shuffle
    const indices = Array.from({ length: pool.length }, (_, i) => i);
    for (let i = indices.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [indices[i], indices[j]] = [indices[j], indices[i]];
    }

    return indices.slice(0, Math.min(count, pool.length)).map((i) => pool[i]);
  }

  return {
    config,
    loading,
    hasLoaded,
    load,
    save,
    getSuggestionsForLanguage,
    getSuggestionsSync,
    sample,
    sampleMany,
  };
});
