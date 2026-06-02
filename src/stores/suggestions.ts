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

function uniqueSuggestions(...pools: string[][]): string[] {
  const seen = new Set<string>();
  const suggestions: string[] = [];

  for (const pool of pools) {
    for (const suggestion of pool) {
      if (seen.has(suggestion)) continue;
      seen.add(suggestion);
      suggestions.push(suggestion);
    }
  }

  return suggestions;
}

function allSuggestions(languageConfig: LanguageSuggestions): string[] {
  return uniqueSuggestions(
    languageConfig.shortSuggestions,
    languageConfig.longSuggestions,
  );
}

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

function getSuggestionPool(
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
   * @param {SuggestionPool} pool Optional break-specific suggestion pool
   * @returns {string[]} Array of suggestions
   */
  function getSuggestionsSync(
    language: string,
    pool?: SuggestionPool,
  ): string[] {
    if (!config.value) return [];

    const languageConfig = config.value.byLanguage[language];
    if (languageConfig) {
      const normalizedLanguageConfig =
        normalizeLanguageSuggestions(languageConfig);
      return pool
        ? getSuggestionPool(normalizedLanguageConfig, pool)
        : allSuggestions(normalizedLanguageConfig);
    }

    console.warn(
      `No suggestions found for language: ${language}, falling back to ${LANGUAGE_FALLBACK}`,
    );

    const fallbackLanguageConfig = normalizeLanguageSuggestions(
      config.value.byLanguage[LANGUAGE_FALLBACK],
    );
    return pool
      ? getSuggestionPool(fallbackLanguageConfig, pool)
      : allSuggestions(fallbackLanguageConfig);
  }

  /**
   * Sample a random suggestion for a specific language
   * @param {string} language Language code
   * @param {SuggestionPool} pool Optional break-specific suggestion pool
   * @returns {string} A random suggestion
   */
  function sample(language: string, pool?: SuggestionPool): string {
    const suggestionPool = getSuggestionsSync(language, pool);
    if (!suggestionPool.length) return "";

    return suggestionPool[Math.floor(Math.random() * suggestionPool.length)];
  }

  /**
   * Sample multiple random suggestions for a specific language
   * @param {string} language Language code
   * @param {number} count Number of suggestions to sample
   * @param {SuggestionPool} pool Optional break-specific suggestion pool
   * @returns {string[]} Array of random suggestions
   */
  function sampleMany(
    language: string,
    count: number = 3,
    pool?: SuggestionPool,
  ): string[] {
    const suggestionPool = getSuggestionsSync(language, pool);
    if (!suggestionPool.length) return [];

    const safeCount = Number.isFinite(count)
      ? Math.max(0, Math.trunc(count))
      : 0;
    if (safeCount === 0) return [];

    // Fisher-Yates shuffle
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
    getSuggestionsForLanguage,
    getSuggestionsSync,
    hasLoaded,
    load,
    loading,
    sample,
    sampleMany,
    save,
  };
});
