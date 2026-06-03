import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { LANGUAGE_FALLBACK } from "@/i18n";
import type { LanguageSuggestions, SuggestionsConfig } from "@/types";

export const SUGGESTION_POOLS = ["short", "long"] as const;
export type SuggestionPool = (typeof SUGGESTION_POOLS)[number];
export type LanguageSuggestionsInput = Partial<LanguageSuggestions>;
export type SuggestionsConfigInput = {
  byLanguage: Record<string, LanguageSuggestionsInput>;
};

const SUGGESTION_POOL_KEYS = {
  long: "longSuggestions",
  short: "shortSuggestions",
} as const satisfies Record<SuggestionPool, keyof LanguageSuggestions>;

export function createLanguageSuggestions(
  shortSuggestions: string[],
  longSuggestions: string[],
): LanguageSuggestions {
  return {
    longSuggestions: [...longSuggestions],
    shortSuggestions: [...shortSuggestions],
    suggestions: [],
  };
}

export function normalizeLanguageSuggestions(
  languageConfig: Partial<LanguageSuggestions> | null | undefined,
): LanguageSuggestions {
  const legacySuggestions = languageConfig?.suggestions ?? [];
  const shortSuggestions =
    languageConfig?.shortSuggestions ?? legacySuggestions;
  const longSuggestions = languageConfig?.longSuggestions ?? legacySuggestions;

  return createLanguageSuggestions(shortSuggestions, longSuggestions);
}

function normalizeSuggestionsConfig(
  config: SuggestionsConfigInput,
): SuggestionsConfig {
  const byLanguage: Record<string, LanguageSuggestions> = {};
  for (const [language, languageConfig] of Object.entries(config.byLanguage)) {
    byLanguage[language] = normalizeLanguageSuggestions(languageConfig);
  }

  return {
    byLanguage,
  };
}

function getSuggestionPoolForLanguageConfig(
  languageConfig: LanguageSuggestions,
  pool: SuggestionPool,
): string[] {
  return languageConfig[SUGGESTION_POOL_KEYS[pool]];
}

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
      const result = await invoke<SuggestionsConfigInput>("get_suggestions");
      config.value = normalizeSuggestionsConfig(result);
    } catch (err) {
      console.error("Failed to load suggestions:", err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Save suggestions configuration to backend
   * @param {SuggestionsConfigInput} newConfig New suggestions configuration
   */
  async function save(newConfig: SuggestionsConfigInput) {
    loading.value = true;
    try {
      const normalizedConfig = normalizeSuggestionsConfig(newConfig);
      await invoke("save_suggestions", { config: normalizedConfig });
      config.value = normalizedConfig;
    } catch (err) {
      console.error("Failed to save suggestions:", err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Get a break-specific suggestion pool for a language.
   * @param {string} language Language code
   * @param {SuggestionPool} pool Break-specific suggestion pool
   * @returns {string[]} Array of suggestions
   */
  function getSuggestionPoolForLanguage(
    language: string,
    pool: SuggestionPool,
  ): string[] {
    if (!config.value) return [];

    const languageConfig = config.value.byLanguage[language];
    if (languageConfig) {
      return getSuggestionPoolForLanguageConfig(languageConfig, pool);
    }

    console.warn(
      `No suggestions found for language: ${language}, falling back to ${LANGUAGE_FALLBACK}`,
    );

    const fallbackLanguageConfig = config.value.byLanguage[LANGUAGE_FALLBACK];
    if (!fallbackLanguageConfig) {
      return [];
    }

    return getSuggestionPoolForLanguageConfig(fallbackLanguageConfig, pool);
  }

  /**
   * Sample multiple random suggestions for a specific language and break kind.
   * @param {string} language Language code
   * @param {SuggestionPool} pool Break-specific suggestion pool
   * @param {number} count Number of suggestions to sample
   * @returns {string[]} Array of random suggestions
   */
  function sampleMany(
    language: string,
    pool: SuggestionPool,
    count: number = 3,
  ): string[] {
    const suggestionPool = getSuggestionPoolForLanguage(language, pool);
    if (!suggestionPool.length) return [];

    const safeCount = Number.isFinite(count)
      ? Math.max(0, Math.trunc(count))
      : 0;
    if (safeCount === 0) return [];

    const indices = Array.from({ length: suggestionPool.length }, (_, i) => i);
    for (let i = indices.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [indices[i], indices[j]] = [indices[j], indices[i]];
    }

    return indices
      .slice(0, Math.min(safeCount, suggestionPool.length))
      .map((i) => suggestionPool[i]);
  }

  return {
    config,
    hasLoaded,
    load,
    loading,
    sampleMany,
    save,
  };
});
