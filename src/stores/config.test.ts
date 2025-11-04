import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";
import { mockInvoke } from "@/test/setup";
import type { AppConfig as RawAppConfig } from "@/types";
import {
  createNoAudio,
  createSolidBackground,
  createSuggestionsSettings,
  createTimeRange,
} from "@/types";
import { useConfigStore } from "./config";

const mockConfig: RawAppConfig = {
  allScreens: false,
  attentions: [
    {
      daysOfWeek: ["Mon", "Tue", "Wed", "Thu", "Fri"],
      durationS: 20,
      enabled: true,
      id: 3,
      message: "Time to refocus.",
      name: "Morning focus",
      theme: {
        background: createSolidBackground("#1e1b4b"),
        blurRadius: 12,
        fontFamily: "Inter",
        fontSize: 24,
        opacity: 0.94,
        textColor: "#f8fafc",
      },
      times: ["10:00"],
      title: "Attention",
    },
  ],
  autostart: false,
  checkForUpdates: true,
  inactiveS: 300,
  language: "en",
  monitorDnd: false,
  postponeShortcut: "Ctrl+X",
  schedules: [
    {
      daysOfWeek: ["Mon", "Tue", "Wed", "Thu", "Fri"],
      enabled: true,
      longBreaks: {
        afterMiniBreaks: 4,
        audio: createNoAudio(),
        durationS: 300,
        enabled: true,
        id: 2,
        postponedS: 600,
        strictMode: false,
        suggestions: createSuggestionsSettings(),
        theme: {
          background: createSolidBackground("#1f2937"),
          blurRadius: 12,
          fontFamily: "Inter",
          fontSize: 28,
          opacity: 0.96,
          textColor: "#f8fafc",
        },
      },
      miniBreaks: {
        audio: createNoAudio(),
        durationS: 20,
        enabled: true,
        id: 1,
        intervalS: 1200,
        postponedS: 300,
        strictMode: false,
        suggestions: createSuggestionsSettings(),
        theme: {
          background: createSolidBackground("#1f2937"),
          blurRadius: 8,
          fontFamily: "Inter",
          fontSize: 24,
          opacity: 0.92,
          textColor: "#f8fafc",
        },
      },
      name: "Work hours",
      notificationBeforeS: 30,
      timeRange: createTimeRange("09:00", "18:00"),
    },
  ],
  themeMode: "system",
  windowSize: 0.8,
};

describe("useConfigStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockInvoke.mockReset();
  });

  describe("initialization", () => {
    it("should start with null draft and original", () => {
      const store = useConfigStore();
      expect(store.draft).toBeNull();
      expect(store.original).toBeNull();
    });

    it("should have loading and saving flags as false", () => {
      const store = useConfigStore();
      expect(store.loading).toBe(false);
      expect(store.saving).toBe(false);
    });

    it("should not be dirty initially", () => {
      const store = useConfigStore();
      expect(store.isDirty).toBe(false);
    });
  });

  describe("load", () => {
    it("should load config from backend", async () => {
      mockInvoke.mockResolvedValue(mockConfig);
      const store = useConfigStore();

      await store.load();

      expect(mockInvoke).toHaveBeenCalledWith("get_config");
      expect(store.draft).toBeTruthy();
      expect(store.original).toBeTruthy();
      expect(store.draft?.language).toBe("en");
    });

    it("should set loading flag during load", async () => {
      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) => setTimeout(() => resolve(mockConfig), 50)),
      );
      const store = useConfigStore();

      const loadPromise = store.load();
      expect(store.loading).toBe(true);

      await loadPromise;
      expect(store.loading).toBe(false);
    });

    it("should handle load errors", async () => {
      const error = new Error("Failed to load config");
      mockInvoke.mockRejectedValue(error);
      const store = useConfigStore();

      await expect(store.load()).rejects.toThrow("Failed to load config");
      expect(store.error).toBe("Failed to load config");
      expect(store.loading).toBe(false);
    });
  });

  describe("save", () => {
    beforeEach(async () => {
      mockInvoke.mockResolvedValue(mockConfig);
      const store = useConfigStore();
      await store.load();
      mockInvoke.mockClear();
    });

    it("should save draft config to backend", async () => {
      mockInvoke.mockResolvedValue(undefined);
      const store = useConfigStore();

      await store.save();

      expect(mockInvoke).toHaveBeenCalledWith(
        "save_config",
        expect.objectContaining({
          config: expect.any(Object),
        }),
      );
    });

    it("should save config to backend with correct types", async () => {
      mockInvoke.mockResolvedValue(undefined);
      const store = useConfigStore();

      await store.save();

      const savedConfig = mockInvoke.mock.calls[0][1].config;
      expect(typeof savedConfig.inactiveS).toBe("number");
      expect(typeof savedConfig.schedules[0].miniBreaks.intervalS).toBe(
        "number",
      );
    });

    it("should update original after successful save", async () => {
      mockInvoke.mockResolvedValue(undefined);
      const store = useConfigStore();

      if (store.draft) {
        store.draft.language = "zh-CN";
      }

      await store.save();

      expect(store.original?.language).toBe("zh-CN");
      expect(store.isDirty).toBe(false);
    });

    it("should set saving flag during save", async () => {
      mockInvoke.mockImplementation(
        () => new Promise((resolve) => setTimeout(resolve, 50)),
      );
      const store = useConfigStore();

      const savePromise = store.save();
      expect(store.saving).toBe(true);

      await savePromise;
      expect(store.saving).toBe(false);
    });

    it("should handle save errors", async () => {
      const error = new Error("Failed to save config");
      mockInvoke.mockRejectedValue(error);
      const store = useConfigStore();

      await expect(store.save()).rejects.toThrow("Failed to save config");
      expect(store.error).toBe("Failed to save config");
      expect(store.saving).toBe(false);
    });
  });

  describe("applyConfig", () => {
    it("should apply raw config", () => {
      const store = useConfigStore();
      store.applyConfig(mockConfig);

      expect(store.draft).toBeTruthy();
      expect(store.original).toBeTruthy();
      expect(store.draft?.language).toBe("en");
    });

    it("should normalize applied config", () => {
      const store = useConfigStore();
      store.applyConfig(mockConfig);

      expect(store.draft?.inactiveS).toBe(300);
      expect(typeof store.draft?.inactiveS).toBe("number");
    });
  });

  describe("isDirty", () => {
    beforeEach(async () => {
      mockInvoke.mockResolvedValue(mockConfig);
      const store = useConfigStore();
      await store.load();
    });

    it("should be false when draft equals original", () => {
      const store = useConfigStore();
      expect(store.isDirty).toBe(false);
    });

    it("should be true when draft is modified", () => {
      const store = useConfigStore();
      if (store.draft) {
        store.draft.language = "zh-CN";
      }
      expect(store.isDirty).toBe(true);
    });

    it("should be false after reset", () => {
      const store = useConfigStore();
      if (store.draft) {
        store.draft.language = "zh-CN";
      }
      expect(store.isDirty).toBe(true);

      store.resetDraft();
      expect(store.isDirty).toBe(false);
    });
  });

  describe("resetDraft", () => {
    beforeEach(async () => {
      mockInvoke.mockResolvedValue(mockConfig);
      const store = useConfigStore();
      await store.load();
    });

    it("should revert draft to original", () => {
      const store = useConfigStore();
      if (store.draft) {
        store.draft.language = "zh-CN";
      }

      store.resetDraft();

      expect(store.draft?.language).toBe("en");
    });
  });

  describe("setLanguage", () => {
    beforeEach(async () => {
      mockInvoke.mockResolvedValue(mockConfig);
      const store = useConfigStore();
      await store.load();
    });

    it("should update language in draft", () => {
      const store = useConfigStore();
      store.setLanguage("zh-CN");

      expect(store.draft?.language).toBe("zh-CN");
      expect(store.isDirty).toBe(true);
    });
  });

  describe("schedule management", () => {
    beforeEach(async () => {
      mockInvoke.mockResolvedValue(mockConfig);
      const store = useConfigStore();
      await store.load();
    });

    it("should add a new schedule", () => {
      const store = useConfigStore();
      const initialCount = store.draft?.schedules.length ?? 0;

      store.addSchedule();

      expect(store.draft?.schedules.length).toBe(initialCount + 1);
      expect(store.draft?.schedules[initialCount].name).toBe(
        `New Schedule (${store.draft?.schedules[initialCount].miniBreaks.id})`,
      );
    });

    it("should remove a schedule by id", () => {
      const store = useConfigStore();
      const scheduleId = store.draft?.schedules[0].miniBreaks.id;

      expect(scheduleId).toBeDefined();
      if (scheduleId) {
        store.removeSchedule(scheduleId);
      }

      expect(store.draft?.schedules.length).toBe(0);
    });

    it("should duplicate a schedule", () => {
      const store = useConfigStore();
      const originalId = store.draft?.schedules[0].miniBreaks.id;
      const originalName = store.draft?.schedules[0].name;

      expect(originalId).toBeDefined();
      if (!originalId) return;

      store.duplicateSchedule(originalId);

      expect(store.draft?.schedules.length).toBe(2);
      expect(store.draft?.schedules[1].name).toBe(`${originalName} copy`);
      expect(store.draft?.schedules[1].miniBreaks.id).not.toBe(originalId);
    });

    it("should generate unique IDs for new schedules", () => {
      const store = useConfigStore();

      store.addSchedule();
      store.addSchedule();

      const ids = store.draft?.schedules.map((s) => s.miniBreaks.id) ?? [];
      const uniqueIds = new Set(ids);
      expect(uniqueIds.size).toBe(ids.length);
    });
  });

  describe("attention management", () => {
    beforeEach(async () => {
      mockInvoke.mockResolvedValue(mockConfig);
      const store = useConfigStore();
      await store.load();
    });

    it("should add a new attention", () => {
      const store = useConfigStore();
      const initialCount = store.draft?.attentions.length ?? 0;

      store.addAttention();

      expect(store.draft?.attentions.length).toBe(initialCount + 1);
      expect(store.draft?.attentions[initialCount].name).toBe("New attention");
    });

    it("should remove an attention by id", () => {
      const store = useConfigStore();
      const attentionId = store.draft?.attentions[0].id;

      expect(attentionId).toBeDefined();
      if (attentionId) {
        store.removeAttention(attentionId);
      }

      expect(store.draft?.attentions.length).toBe(0);
    });

    it("should generate unique IDs for new attentions", () => {
      const store = useConfigStore();

      store.addAttention();
      store.addAttention();

      const ids = store.draft?.attentions.map((a) => a.id) ?? [];
      const uniqueIds = new Set(ids);
      expect(uniqueIds.size).toBe(ids.length);
    });
  });

  describe("getRawDraft", () => {
    it("should return null when draft is null", () => {
      const store = useConfigStore();
      expect(store.getRawDraft()).toBeNull();
    });

    it("should return draft config as-is (no denormalization needed)", async () => {
      mockInvoke.mockResolvedValue(mockConfig);
      const store = useConfigStore();
      await store.load();

      const raw = store.getRawDraft();

      expect(raw).toBeTruthy();
      expect(typeof raw?.inactiveS).toBe("number");
      expect(typeof raw?.schedules[0].miniBreaks.intervalS).toBe("number");
    });
  });

  describe("error handling", () => {
    it("should clear error on successful load", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("First error"));
      const store = useConfigStore();

      await expect(store.load()).rejects.toThrow();
      expect(store.error).toBeTruthy();

      mockInvoke.mockResolvedValue(mockConfig);
      await store.load();
      expect(store.error).toBeNull();
    });

    it("should throw when accessing draft before load", () => {
      const store = useConfigStore();
      expect(() => store.setLanguage("zh-CN")).toThrow("Config not loaded yet");
    });
  });
});
