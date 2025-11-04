<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import ClockIcon from "@/components/icons/ClockIcon.vue";
import InfoIcon from "@/components/icons/InfoIcon.vue";
import MonitorIcon from "@/components/icons/MonitorIcon.vue";
import SettingGear from "@/components/icons/SettingGear.vue";
import SlidersIcon from "@/components/icons/SlidersIcon.vue";
import KeyCapture from "@/components/ui/KeyCapture.vue";
import {
  useComputedProp,
  useComputedValidated,
  useDecimalToPercent,
} from "@/composables/useComputed";
import { supportedLocales } from "@/i18n";
import type { AppConfig, ThemeMode } from "@/stores/config";
import { useConfigStore } from "@/stores/config";

const props = defineProps<{ config: AppConfig }>();
const emit =
  defineEmits<
    (
      event: "notify",
      kind: "success" | "error" | "info",
      message: string,
    ) => void
  >();

const { t } = useI18n();
const configStore = useConfigStore();
const locales = supportedLocales;

// Autostart state
const autostartEnabled = ref(false);
const autostartLoading = ref(true);

// Use composables for computed properties
const inactivitySeconds = useComputedValidated(
  () => props.config.inactiveS,
  (value) => {
    props.config.inactiveS = value;
  },
  (value) => Math.max(30, Math.round(value)),
);

const postponeShortcut = useComputedProp(
  () => props.config,
  "postponeShortcut",
);

const windowSizePercent = useDecimalToPercent(
  () => props.config.windowSize,
  (value) => {
    props.config.windowSize = value;
  },
  10,
  100,
);

/**
 * Handle language change event
 * @param {Event} event The change event
 */
function onLanguageChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value;
  configStore.setLanguage(value);
}

/**
 * Load autostart status
 */
async function loadAutostartStatus() {
  try {
    // Use config value as source of truth
    autostartEnabled.value = props.config.autostart;
  } catch (err) {
    console.error("Failed to load autostart status:", err);
  } finally {
    autostartLoading.value = false;
  }
}

/**
 * Toggle autostart
 */
async function toggleAutostart() {
  const newValue = !autostartEnabled.value;
  try {
    await invoke("set_autostart_enabled", { enabled: newValue });
    autostartEnabled.value = newValue;
    // Update local config
    props.config.autostart = newValue;
    emit(
      "notify",
      "success",
      newValue ? t("general.autostartEnabled") : t("general.autostartDisabled"),
    );
  } catch (err) {
    console.error("Failed to toggle autostart:", err);
    // Show warning instead of error since preference is still saved
    emit(
      "notify",
      "info",
      `${newValue ? t("general.autostartEnabled") : t("general.autostartDisabled")} (${err})`,
    );
    // Still update the UI since preference was saved
    autostartEnabled.value = newValue;
    props.config.autostart = newValue;
  }
}

onMounted(() => {
  loadAutostartStatus();
});
</script>

<template>
  <section class="space-y-6">
    <!-- Header Card -->
    <div
      class="rounded-2xl border border-primary/30 bg-linear-to-br from-primary/10 via-primary/5 to-transparent p-6 shadow-sm backdrop-blur-sm">
      <div class="flex flex-col sm:flex-row items-start gap-5">
        <div
          class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-linear-to-br from-primary to-primary/80 shadow-lg">
          <SettingGear class-name="h-7 w-7 text-white" />
        </div>
        <div class="flex-1 min-w-0">
          <h2 class="text-2xl font-bold text-base-content mb-2.5">
            {{ t("general.title") }}
          </h2>
          <p class="text-sm text-base-content/70 leading-relaxed mb-4">
            {{ t("general.postponeHint") }}
          </p>
          <div class="flex flex-wrap gap-2 items-center">
            <div class="badge badge-primary badge-outline gap-1.5 py-3 px-3">
              <InfoIcon class-name="h-3.5 w-3.5" />
              <span class="text-xs font-medium">{{ t("general.uiSettings") }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Behavior Settings -->
    <div class="rounded-2xl border border-base-300 bg-base-100/70 p-6 shadow-md">
      <h3 class="text-lg font-bold mb-6 flex items-center gap-2">
        <SlidersIcon class-name="h-5 w-5 text-primary" />
        {{ t("general.behaviorSettings") }}
      </h3>
      <div class="space-y-2">
        <div class="flex items-center justify-between gap-4 p-4 rounded-lg hover:bg-base-200/50 transition-all">
          <div class="flex-1 min-w-0">
            <div class="font-medium text-sm">{{ t("general.autostart") }}</div>
            <p class="text-xs text-base-content/50 mt-1">
              {{ t("general.autostartHint") }}
            </p>
          </div>
          <input v-if="!autostartLoading" :checked="autostartEnabled" type="checkbox"
            class="toggle toggle-primary toggle-lg shrink-0 transition-all" @change="toggleAutostart" />
          <span v-else class="loading loading-spinner loading-md"></span>
        </div>

        <div class="divider my-0"></div>

        <div class="flex items-center justify-between gap-4 p-4 rounded-lg hover:bg-base-200/50 transition-all">
          <div class="flex-1 min-w-0">
            <div class="font-medium text-sm">{{ t("general.checkUpdates") }}</div>
            <p class="text-xs text-base-content/50 mt-1">
              {{ t("general.checkUpdatesHint") }}
            </p>
          </div>
          <input v-model="config.checkForUpdates" type="checkbox"
            class="toggle toggle-primary toggle-lg shrink-0 transition-all" />
        </div>

        <div class="divider my-0"></div>

        <div class="flex items-center justify-between gap-4 p-4 rounded-lg hover:bg-base-200/50 transition-all">
          <div class="flex-1 min-w-0">
            <div class="font-medium text-sm">{{ t("general.monitorDnd") }}</div>
            <p class="text-xs text-base-content/50 mt-1">
              {{ t("general.monitorDndHint") }}
            </p>
          </div>
          <input v-model="config.monitorDnd" type="checkbox"
            class="toggle toggle-primary toggle-lg shrink-0 transition-all" />
        </div>

        <div class="divider my-0"></div>

        <div class="flex items-center justify-between gap-4 p-4 rounded-lg hover:bg-base-200/50 transition-all">
          <div class="flex-1 min-w-0">
            <div class="font-medium text-sm">{{ t("general.allScreens") }}</div>
            <p class="text-xs text-base-content/50 mt-1">
              {{ t("general.allScreensHint") }}
            </p>
          </div>
          <input v-model="config.allScreens" type="checkbox"
            class="toggle toggle-primary toggle-lg shrink-0 transition-all" />
        </div>
      </div>
    </div>

    <!-- Timer Settings -->
    <div class="rounded-2xl border border-base-300 bg-base-100/70 p-6 shadow-md">
      <h3 class="text-lg font-bold mb-6 flex items-center gap-2">
        <ClockIcon class-name="h-5 w-5 text-primary" />
        {{ t("general.timerSettings") }}
      </h3>
      <div class="grid gap-6 md:grid-cols-2">
        <label class="form-control w-full">
          <div class="label pb-2">
            <span class="label-text font-medium text-sm">
              {{ t("general.inactivityLabel") }}
            </span>
          </div>
          <div class="join w-full">
            <input v-model.number="inactivitySeconds" type="number" min="30" step="10"
              class="input input-bordered join-item flex-1 focus:input-primary transition-all" />
            <span class="btn btn-ghost join-item pointer-events-none text-sm">{{
              t("general.inactivityUnit")
            }}</span>
          </div>
          <div class="label pt-1">
            <span class="label-text-alt text-base-content/50 text-xs">{{
              t("general.inactivityHint")
            }}</span>
          </div>
        </label>

        <KeyCapture v-model="postponeShortcut" :label="t('general.postponeShortcut')"
          :placeholder="t('general.postponeShortcutHint')" />
      </div>
    </div>

    <!-- UI Settings -->
    <div class="rounded-2xl border border-base-300 bg-base-100/70 p-6 shadow-md">
      <h3 class="text-lg font-bold mb-6 flex items-center gap-2">
        <MonitorIcon class-name="h-5 w-5 text-primary" />
        {{ t("general.uiSettings") }}
      </h3>
      <div class="grid gap-6 md:grid-cols-2">
        <label class="form-control w-full">
          <div class="label pb-2">
            <span class="label-text font-medium text-sm">{{ t("general.language") }}</span>
          </div>
          <select class="select select-bordered focus:select-primary w-full transition-all" :value="config.language"
            @change="onLanguageChange">
            <option v-for="locale in locales" :key="locale.key" :value="locale.key">
              {{ locale.label }}
            </option>
          </select>
        </label>

        <label class="form-control w-full">
          <div class="label pb-2">
            <span class="label-text font-medium text-sm">{{ t("general.themeMode") }}</span>
          </div>
          <select class="select select-bordered focus:select-primary w-full transition-all" :value="config.themeMode"
            @change="
              (e: Event) => configStore.setThemeMode((e.target as HTMLSelectElement).value as ThemeMode)
            ">
            <option value="light">{{ t("general.themeModeLight") }}</option>
            <option value="dark">{{ t("general.themeModeDark") }}</option>
            <option value="system">{{ t("general.themeModeSystem") }}</option>
          </select>
        </label>

        <div class="md:col-span-1 flex justify-left">
          <label class="form-control w-full max-w-lg">
            <div class="label pb-2">
              <span class="label-text font-medium text-sm">{{ t("general.windowSize") }}</span>
            </div>
            <input v-model.number="windowSizePercent" type="range" min="50" max="100" step="5"
              class="range range-primary w-full" />
            <div class="flex justify-between items-center mt-3 px-2">
              <span class="text-xs text-base-content/50">50%</span>
              <span class="badge badge-primary badge-lg font-semibold">
                {{ windowSizePercent >= 100 ? t("general.fullscreen") : `${windowSizePercent}%` }}
              </span>
              <span class="text-xs text-base-content/50">100%</span>
            </div>
          </label>
        </div>
      </div>
    </div>
  </section>
</template>
