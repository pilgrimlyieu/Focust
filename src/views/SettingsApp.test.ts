import { flushPromises, mount, type VueWrapper } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";
import SuggestionsPanel from "@/components/settings/SuggestionsPanel.vue";
import { useToast } from "@/composables/useToast";
import { i18n } from "@/i18n";
import { useConfigStore } from "@/stores/config";
import { useSchedulerStore } from "@/stores/scheduler";
import {
  createLanguageSuggestions,
  useSuggestionsStore,
} from "@/stores/suggestions";
import { mockInvoke, mockListen } from "@/test/setup";
import type { AppConfig } from "@/types";
import SettingsApp from "./SettingsApp.vue";

const config: AppConfig = {
  allScreens: false,
  appExclusions: [],
  attentions: [],
  autostart: false,
  inactiveS: 300,
  language: "en-US",
  monitorDnd: false,
  postponeShortcut: "Ctrl+X",
  schedules: [],
  showTrayIcon: true,
  themeMode: "light",
  windowSize: 0.8,
};

describe("settings reset and undo", () => {
  let wrapper: VueWrapper<InstanceType<typeof SettingsApp>>;

  beforeEach(async () => {
    const pinia = createPinia();
    setActivePinia(pinia);
    useConfigStore().applyConfig(config);
    useSuggestionsStore().config = {
      byLanguage: {
        "en-US": createLanguageSuggestions(["Saved mini"], ["Saved long"]),
      },
    };
    vi.spyOn(useSchedulerStore(), "init").mockResolvedValue();
    mockListen.mockResolvedValue(() => {});
    mockInvoke.mockResolvedValue(undefined);
    wrapper = mount(SettingsApp, {
      global: {
        plugins: [pinia, i18n],
        stubs: {
          GeneralSettingsPanel: true,
          transition: true,
          "transition-group": true,
        },
      },
    });
    await flushPromises();
  });

  afterEach(() => {
    wrapper.unmount();
    useToast().toasts.value = [];
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  async function navigate(tab: "general" | "suggestions") {
    const link = wrapper
      .findAll("aside a")
      .find((item) => item.text() === i18n.global.t(`nav.${tab}`));
    if (!link) throw new Error(`Missing settings tab: ${tab}`);
    await link.trigger("click");
    await flushPromises();
  }

  function buttonWithText(panel: Pick<VueWrapper, "findAll">, label: string) {
    const button = panel
      .findAll("button")
      .find((item) => item.text() === label);
    if (!button) throw new Error(`Missing editor button: ${label}`);
    return button;
  }

  it("keeps Undo reset in the header until it is used or another edit begins", async () => {
    const store = useConfigStore();
    (store.draft as AppConfig).windowSize = 0.5;
    await nextTick();
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    await wrapper.get('header button[aria-label="Reset"]').trigger("click");
    await vi.advanceTimersByTimeAsync(60_000);
    expect(store.isDirty).toBe(false);
    await wrapper
      .get('header button[aria-label="Undo reset"]')
      .trigger("click");
    expect(store.draft?.windowSize).toBe(0.5);
    expect(store.original?.windowSize).toBe(0.8);
    expect(
      wrapper.get<HTMLButtonElement>("header button.btn-primary").element
        .disabled,
    ).toBe(false);
    expect(mockInvoke).not.toHaveBeenCalled();

    await wrapper.get('header button[aria-label="Reset"]').trigger("click");
    (store.draft as AppConfig).windowSize = 0.7;
    await nextTick();
    expect(
      wrapper.find('header button[aria-label="Undo reset"]').exists(),
    ).toBe(false);
    expect(
      wrapper.get<HTMLButtonElement>('header button[aria-label="Reset"]')
        .element.disabled,
    ).toBe(false);
    expect(store.draft?.windowSize).toBe(0.7);
  });

  it("restores both suggestion pools and unfinished input after navigating away", async () => {
    await navigate("suggestions");
    let panel = wrapper.getComponent(SuggestionsPanel);
    await panel.get("input.input-sm").setValue("Edited mini");
    await panel.findAll("button[aria-pressed]")[1].trigger("click");
    await panel.get("input.input-sm").setValue("Edited long");
    await panel.get("input[placeholder]").setValue("Unadded idea");
    await panel.get('button[aria-label="Reset"]').trigger("click");
    expect(panel.get<HTMLInputElement>("input.input-sm").element.value).toBe(
      "Saved long",
    );
    expect(
      panel.get<HTMLInputElement>("input[placeholder]").element.value,
    ).toBe("Unadded idea");
    await navigate("general");
    await navigate("suggestions");
    panel = wrapper.getComponent(SuggestionsPanel);
    await panel.get('button[aria-label="Undo reset"]').trigger("click");
    expect(panel.get<HTMLInputElement>("input.input-sm").element.value).toBe(
      "Edited long",
    );
    expect(
      panel.get<HTMLInputElement>("input[placeholder]").element.value,
    ).toBe("Unadded idea");
    expect(
      useSuggestionsStore().config?.byLanguage["en-US"].longSuggestions,
    ).toEqual(["Saved long"]);
    expect(mockInvoke).not.toHaveBeenCalled();
    await panel.findAll("button[aria-pressed]")[0].trigger("click");
    expect(panel.get<HTMLInputElement>("input.input-sm").element.value).toBe(
      "Edited mini",
    );
    await panel.get("button.btn-sm.btn-primary").trigger("click");
    await flushPromises();
    expect(mockInvoke).toHaveBeenCalledWith("save_suggestions", {
      config: {
        byLanguage: {
          "en-US": createLanguageSuggestions(["Edited mini"], ["Edited long"]),
        },
      },
    });
    expect(
      panel.get<HTMLButtonElement>('button[aria-label="Reset"]').element
        .disabled,
    ).toBe(true);
  });

  it.each(["input.input-sm", "input[placeholder]"])(
    "invalidates suggestion undo on input in %s, even before a saved value changes",
    async (selector) => {
      await navigate("suggestions");
      const panel = wrapper.getComponent(SuggestionsPanel);
      await panel.get("input.input-sm").setValue("Discarded draft");
      await panel.get('button[aria-label="Reset"]').trigger("click");
      await panel
        .get(selector)
        .setValue(selector === "input.input-sm" ? "Saved mini " : "New idea");
      expect(panel.find('button[aria-label="Undo reset"]').exists()).toBe(
        false,
      );
      expect(
        panel.get<HTMLButtonElement>('button[aria-label="Reset"]').element
          .disabled,
      ).toBe(true);
    },
  );

  it("restores suggestions while keeping the current mode and pool", async () => {
    useSuggestionsStore().config = { byLanguage: {} };
    await navigate("suggestions");
    const panel = wrapper.getComponent(SuggestionsPanel);
    await buttonWithText(panel, "Bulk Import").trigger("click");
    await panel.get("textarea").setValue("Unsaved mini");
    await panel.get('button[aria-label="Reset"]').trigger("click");
    await buttonWithText(panel, "List Mode").trigger("click");
    await panel.findAll("button[aria-pressed]")[1].trigger("click");
    await panel.get('button[aria-label="Undo reset"]').trigger("click");
    expect(panel.find("textarea").exists()).toBe(false);
    expect(
      panel.findAll("button[aria-pressed]")[1].attributes("aria-pressed"),
    ).toBe("true");
    await panel.findAll("button[aria-pressed]")[0].trigger("click");
    expect(panel.get<HTMLInputElement>("input.input-sm").element.value).toBe(
      "Unsaved mini",
    );
  });

  it("expires suggestion undo when the saved configuration is reloaded", async () => {
    await navigate("suggestions");
    const panel = wrapper.getComponent(SuggestionsPanel);
    await panel.get("input.input-sm").setValue("Discarded draft");
    await panel.get('button[aria-label="Reset"]').trigger("click");
    mockInvoke.mockResolvedValue(useSuggestionsStore().config);
    await useSuggestionsStore().load();
    await nextTick();
    expect(panel.find('button[aria-label="Undo reset"]').exists()).toBe(false);
    expect(panel.get<HTMLInputElement>("input.input-sm").element.value).toBe(
      "Saved mini",
    );
  });

  it("expires undo when a saved suggestion changes through blur normalization", async () => {
    useSuggestionsStore().config = {
      byLanguage: {
        "en-US": createLanguageSuggestions([" Saved mini "], []),
      },
    };
    await navigate("suggestions");
    const panel = wrapper.getComponent(SuggestionsPanel);
    await panel.get("input.input-sm").setValue("Discarded draft");
    await panel.get('button[aria-label="Reset"]').trigger("click");
    await panel.get("input.input-sm").trigger("blur");
    expect(panel.find('button[aria-label="Undo reset"]').exists()).toBe(false);
    expect(panel.get<HTMLInputElement>("input.input-sm").element.value).toBe(
      "Saved mini",
    );
    expect(
      panel.get<HTMLButtonElement>('button[aria-label="Reset"]').element
        .disabled,
    ).toBe(false);
  });
});
