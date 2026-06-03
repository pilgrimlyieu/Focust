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

  describe("sampleMany", () => {
    it("should sample only from the requested break-specific pool", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(
            ["Short 1", "Short 2"],
            ["Long 1", "Long 2"],
          ),
        },
      };

      expect(store.sampleMany("en-US", "short", 2).sort()).toEqual([
        "Short 1",
        "Short 2",
      ]);
      expect(store.sampleMany("en-US", "long", 2).sort()).toEqual([
        "Long 1",
        "Long 2",
      ]);
    });

    it("should fallback to en-US for unknown language", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(
            ["English short"],
            ["English long"],
          ),
        },
      };

      expect(store.sampleMany("fr-FR", "long", 3)).toEqual(["English long"]);
      expect(console.warn).toHaveBeenCalledTimes(1);
    });

    it("should return all suggestions if count exceeds the pool size", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(["Short 1", "Short 2"], ["Long"]),
        },
      };

      expect(store.sampleMany("en-US", "short", 5).sort()).toEqual([
        "Short 1",
        "Short 2",
      ]);
    });

    it("should return empty array if config or the pool is empty", () => {
      const store = useSuggestionsStore();
      store.config = null;

      expect(store.sampleMany("en-US", "short", 3)).toEqual([]);
      expect(console.warn).not.toHaveBeenCalled();

      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions([], ["Long"]),
        },
      };

      expect(store.sampleMany("en-US", "short", 3)).toEqual([]);
    });

    it("should return empty array for invalid counts", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": createLanguageSuggestions(["Short"], ["Long"]),
        },
      };

      expect(store.sampleMany("en-US", "short", -1)).toEqual([]);
      expect(store.sampleMany("en-US", "short", Number.NaN)).toEqual([]);
    });
  });
});
