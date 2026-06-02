import { invoke } from "@tauri-apps/api/core";
import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  createLanguageSuggestions,
  normalizeLanguageSuggestions,
  useSuggestionsStore,
} from "./suggestions";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;
let consoleWarnSpy: ReturnType<typeof vi.spyOn>;

function legacySuggestions(suggestions: string[]) {
  return { suggestions };
}

describe("useSuggestionsStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    consoleWarnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
  });

  afterEach(() => {
    consoleWarnSpy.mockRestore();
  });

  describe("initial state", () => {
    it("should start with empty config and not loaded", () => {
      const store = useSuggestionsStore();
      expect(store.config).toBeNull();
      expect(store.hasLoaded).toBe(false);
      expect(store.loading).toBe(false);
    });
  });

  describe("load", () => {
    it("should normalize legacy suggestions loaded from backend", async () => {
      const mockConfig = {
        byLanguage: {
          "en-US": legacySuggestions(["Stretch 1", "Stretch 2"]),
          "zh-CN": legacySuggestions(["伸展 1", "伸展 2"]),
        },
      };

      mockInvoke.mockResolvedValue(mockConfig);

      const store = useSuggestionsStore();
      await store.load();

      expect(invoke).toHaveBeenCalledWith("get_suggestions");
      expect(store.config).toEqual({
        byLanguage: {
          "en-US": createLanguageSuggestions(
            ["Stretch 1", "Stretch 2"],
            ["Stretch 1", "Stretch 2"],
          ),
          "zh-CN": createLanguageSuggestions(
            ["伸展 1", "伸展 2"],
            ["伸展 1", "伸展 2"],
          ),
        },
      });
      expect(store.hasLoaded).toBe(true);
      expect(store.loading).toBe(false);
    });

    it("should keep split suggestions loaded from backend", async () => {
      const mockConfig = {
        byLanguage: {
          "en-US": createLanguageSuggestions(["Look away"], ["Walk"]),
        },
      };

      mockInvoke.mockResolvedValue(mockConfig);

      const store = useSuggestionsStore();
      await store.load();

      expect(store.config).toEqual(mockConfig);
    });

    it("should handle load errors gracefully", async () => {
      mockInvoke.mockRejectedValue(new Error("Backend error"));

      const store = useSuggestionsStore();
      await expect(store.load()).rejects.toThrow("Backend error");

      expect(store.loading).toBe(false);
      expect(store.hasLoaded).toBe(false);
    });
  });

  describe("save", () => {
    it("should save normalized suggestions to backend", async () => {
      const newConfig = {
        byLanguage: {
          "en-US": legacySuggestions(["New 1", "New 2"]),
        },
      };
      const normalizedConfig = {
        byLanguage: {
          "en-US": createLanguageSuggestions(
            ["New 1", "New 2"],
            ["New 1", "New 2"],
          ),
        },
      };

      mockInvoke.mockResolvedValue(undefined);

      const store = useSuggestionsStore();
      await store.save(newConfig);

      expect(invoke).toHaveBeenCalledWith("save_suggestions", {
        config: normalizedConfig,
      });
      expect(store.config).toEqual(normalizedConfig);
    });
  });

  describe("normalizeLanguageSuggestions", () => {
    it("should not share arrays between legacy fallback pools", () => {
      const normalized = normalizeLanguageSuggestions(
        legacySuggestions(["Legacy"]),
      );

      normalized.shortSuggestions.push("Short only");

      expect(normalized.shortSuggestions).toEqual(["Legacy", "Short only"]);
      expect(normalized.longSuggestions).toEqual(["Legacy"]);
    });
  });

  describe("getSuggestionsSync", () => {
    it("should return suggestions for given language", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(["English 1"], ["English 2"]),
          "zh-CN": createLanguageSuggestions(["中文 1"], ["中文 2"]),
        },
      };

      const enSuggestions = store.getSuggestionsSync("en-US");
      expect(enSuggestions).toEqual(["English 1", "English 2"]);

      const zhSuggestions = store.getSuggestionsSync("zh-CN");
      expect(zhSuggestions).toEqual(["中文 1", "中文 2"]);
    });

    it("should return break-specific suggestion pools", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(["Short 1"], ["Long 1"]),
        },
      };

      expect(store.getSuggestionsSync("en-US", "short")).toEqual(["Short 1"]);
      expect(store.getSuggestionsSync("en-US", "long")).toEqual(["Long 1"]);
    });

    it("should fallback to en-US for unknown language", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(["English 1"], ["English 2"]),
        },
      };

      const suggestions = store.getSuggestionsSync("fr-FR");
      expect(suggestions).toEqual(["English 1", "English 2"]);
    });

    it("should return empty array if config is null", () => {
      const store = useSuggestionsStore();
      store.config = null;

      const suggestions = store.getSuggestionsSync("en-US");
      expect(suggestions).toEqual([]);
    });

    it("should not warn when config is null", () => {
      const store = useSuggestionsStore();
      store.config = null;

      const suggestions = store.getSuggestionsSync("en-US");
      expect(suggestions).toEqual([]);
      expect(console.warn).not.toHaveBeenCalled();
    });

    it("should warn when language is missing and fallback is used", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(["English 1"], ["English 2"]),
        },
      };

      const suggestions = store.getSuggestionsSync("fr-FR");
      expect(suggestions).toEqual(["English 1", "English 2"]);
      expect(console.warn).toHaveBeenCalledTimes(1);
    });
  });

  describe("sample", () => {
    it("should return random suggestion from given language", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(["Test 1", "Test 2"], ["Test 3"]),
        },
      };

      const sample1 = store.sample("en-US");
      expect(["Test 1", "Test 2", "Test 3"]).toContain(sample1);
    });

    it("should return empty string for language with no suggestions", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions([], []),
        },
      };

      const sample = store.sample("en-US");
      expect(sample).toBe("");
    });

    it("should return empty string if config is null", () => {
      const store = useSuggestionsStore();
      store.config = null;

      const sample = store.sample("en-US");
      expect(sample).toBe("");
    });

    it("should sample only from the requested break-specific pool", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(["Short only"], ["Long only"]),
        },
      };

      expect(store.sample("en-US", "short")).toBe("Short only");
      expect(store.sample("en-US", "long")).toBe("Long only");
    });
  });

  describe("sampleMany", () => {
    it("should return multiple random suggestions from given language", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(
            ["Test 1", "Test 2"],
            ["Test 3", "Test 4"],
          ),
        },
      };

      const samples = store.sampleMany("en-US", 3);
      expect(samples.length).toBe(3);
      samples.forEach((sample) => {
        expect(["Test 1", "Test 2", "Test 3", "Test 4"]).toContain(sample);
      });

      // All should be unique
      expect(new Set(samples).size).toBe(3);
    });

    it("should return all suggestions if count exceeds pool size", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(["Test 1"], ["Test 2"]),
        },
      };

      const samples = store.sampleMany("en-US", 5);
      expect(samples.length).toBe(2);
    });

    it("should return empty array for language with no suggestions", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions([], []),
        },
      };

      const samples = store.sampleMany("en-US", 3);
      expect(samples).toEqual([]);
    });

    it("should return empty array if config is null", () => {
      const store = useSuggestionsStore();
      store.config = null;

      const samples = store.sampleMany("en-US", 3);
      expect(samples).toEqual([]);
    });

    it("should return empty array when count is negative", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(["Test 1", "Test 2"], ["Test 3"]),
        },
      };

      const samples = store.sampleMany("en-US", -1);
      expect(samples).toEqual([]);
    });

    it("should return empty array when count is not finite", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(["Test 1", "Test 2"], ["Test 3"]),
        },
      };

      const samples = store.sampleMany("en-US", Number.NaN);
      expect(samples).toEqual([]);
    });

    it("should sample many only from the requested break-specific pool", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(
            ["Short 1", "Short 2"],
            ["Long 1", "Long 2"],
          ),
        },
      };

      expect(store.sampleMany("en-US", 2, "short").sort()).toEqual([
        "Short 1",
        "Short 2",
      ]);
      expect(store.sampleMany("en-US", 2, "long").sort()).toEqual([
        "Long 1",
        "Long 2",
      ]);
    });
  });
});
