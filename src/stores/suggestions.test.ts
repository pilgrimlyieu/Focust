import { invoke } from "@tauri-apps/api/core";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useSuggestionsStore } from "./suggestions";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

describe("useSuggestionsStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
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
    it("should load suggestions from backend", async () => {
      const mockConfig = {
        byLanguage: {
          "en-US": { suggestions: ["Stretch 1", "Stretch 2"] },
          "zh-CN": { suggestions: ["伸展 1", "伸展 2"] },
        },
      };

      mockInvoke.mockResolvedValue(mockConfig);

      const store = useSuggestionsStore();
      await store.load();

      expect(invoke).toHaveBeenCalledWith("get_suggestions");
      expect(store.config).toEqual(mockConfig);
      expect(store.hasLoaded).toBe(true);
      expect(store.loading).toBe(false);
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
    it("should save suggestions to backend", async () => {
      const newConfig = {
        byLanguage: {
          "en-US": { suggestions: ["New 1", "New 2"] },
        },
      };

      mockInvoke.mockResolvedValue(undefined);

      const store = useSuggestionsStore();
      await store.save(newConfig);

      expect(invoke).toHaveBeenCalledWith("save_suggestions", {
        config: newConfig,
      });
      expect(store.config).toEqual(newConfig);
    });
  });

  describe("getSuggestionsSync", () => {
    it("should return suggestions for given language", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": { suggestions: ["English 1", "English 2"] },
          "zh-CN": { suggestions: ["中文 1", "中文 2"] },
        },
      };

      const enSuggestions = store.getSuggestionsSync("en-US");
      expect(enSuggestions).toEqual(["English 1", "English 2"]);

      const zhSuggestions = store.getSuggestionsSync("zh-CN");
      expect(zhSuggestions).toEqual(["中文 1", "中文 2"]);
    });

    it("should fallback to en-US for unknown language", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": { suggestions: ["English 1"] },
        },
      };

      const suggestions = store.getSuggestionsSync("fr-FR");
      expect(suggestions).toEqual(["English 1"]);
    });

    it("should return empty array if config is null", () => {
      const store = useSuggestionsStore();
      store.config = null;

      const suggestions = store.getSuggestionsSync("en-US");
      expect(suggestions).toEqual([]);
    });
  });

  describe("sample", () => {
    it("should return random suggestion from given language", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": { suggestions: ["Test 1", "Test 2", "Test 3"] },
        },
      };

      const sample1 = store.sample("en-US");
      expect(["Test 1", "Test 2", "Test 3"]).toContain(sample1);
    });

    it("should return empty string for language with no suggestions", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": { suggestions: [] },
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
  });

  describe("sampleMany", () => {
    it("should return multiple random suggestions from given language", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": { suggestions: ["Test 1", "Test 2", "Test 3", "Test 4"] },
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
          "en-US": { suggestions: ["Test 1", "Test 2"] },
        },
      };

      const samples = store.sampleMany("en-US", 5);
      expect(samples.length).toBe(2);
    });

    it("should return empty array for language with no suggestions", () => {
      const store = useSuggestionsStore();
      store.config = {
        byLanguage: {
          "en-US": { suggestions: [] },
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
  });
});
