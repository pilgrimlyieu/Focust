<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import KeyCapture from "@/components/ui/KeyCapture.vue";
import { supportedLocales } from "@/i18n";
import type { AppConfig, ThemeMode } from "@/stores/config";
import { useConfigStore } from "@/stores/config";

const props = defineProps<{ config: AppConfig }>();
const { t } = useI18n();
const configStore = useConfigStore();
const locales = supportedLocales;

const inactivitySeconds = computed({
  get: () => props.config.inactiveS,
  set: (value: number) => {
    props.config.inactiveS = Math.max(30, Math.round(value));
  },
});

const postponeShortcut = computed({
  get: () => props.config.postponeShortcut,
  set: (value: string) => {
    props.config.postponeShortcut = value;
  },
});

const windowSizePercent = computed({
  get: () => Math.round(props.config.windowSize * 100),
  set: (value: number) => {
    props.config.windowSize = Math.max(10, Math.min(100, value)) / 100; // min 10%
  },
});

/**
 * Handle language change event
 * @param {Event} event The change event
 */
function onLanguageChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value;
  configStore.setLanguage(value);
}

defineExpose({
  inactivitySeconds,
  locales,
  onLanguageChange,
  postponeShortcut,
  t,
});
</script>

<template>
  <section class="space-y-6">
    <!-- Header -->
    <div class="mb-8">
      <h2 class="text-2xl font-bold text-base-content mb-2">{{ t("general.title") }}</h2>
      <p class="text-sm text-base-content/60">{{ t("general.postponeHint") }}</p>
    </div>

    <!-- Behavior settings card -->
    <div class="card bg-base-100 shadow-xl border border-base-300">
      <div class="card-body">
        <h3 class="card-title text-lg mb-6 flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-primary" fill="none" viewBox="0 0 24 24"
            stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
          </svg>
          {{ t("general.behaviorSettings") }}
        </h3>
        <div class="space-y-5">
          <div class="flex items-center justify-between gap-4 p-4 rounded-lg hover:bg-base-200/50 transition-colors">
            <div class="flex-1 min-w-0">
              <div class="font-medium text-base-content">{{ t("general.checkUpdates") }}</div>
              <p class="text-xs text-base-content/50 mt-1">{{ t("general.checkUpdatesHint") }}</p>
            </div>
            <input v-model="config.checkForUpdates" type="checkbox" class="toggle toggle-primary toggle-lg shrink-0" />
          </div>

          <div class="divider my-0"></div>

          <div class="flex items-center justify-between gap-4 p-4 rounded-lg hover:bg-base-200/50 transition-colors">
            <div class="flex-1 min-w-0">
              <div class="font-medium text-base-content">{{ t("general.monitorDnd") }}</div>
              <p class="text-xs text-base-content/50 mt-1">{{ t("general.monitorDndHint") }}</p>
            </div>
            <input v-model="config.monitorDnd" type="checkbox" class="toggle toggle-primary toggle-lg shrink-0" />
          </div>

          <div class="divider my-0"></div>

          <div class="flex items-center justify-between gap-4 p-4 rounded-lg hover:bg-base-200/50 transition-colors">
            <div class="flex-1 min-w-0">
              <div class="font-medium text-base-content">{{ t("general.allScreens") }}</div>
              <p class="text-xs text-base-content/50 mt-1">{{ t("general.allScreensHint") }}</p>
            </div>
            <input v-model="config.allScreens" type="checkbox" class="toggle toggle-primary toggle-lg shrink-0" />
          </div>
        </div>
      </div>
    </div>

    <!-- Timer settings card -->
    <div class="card bg-base-100 shadow-xl border border-base-300">
      <div class="card-body">
        <h3 class="card-title text-lg mb-6 flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-primary" fill="none" viewBox="0 0 24 24"
            stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          {{ t("general.timerSettings") }}
        </h3>
        <div class="grid gap-6 md:grid-cols-2">
          <label class="form-control w-full">
            <div class="label pb-2">
              <span class="label-text font-medium flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-info" fill="none" viewBox="0 0 24 24"
                  stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                {{ t("general.inactivityLabel") }}
              </span>
            </div>
            <div class="join w-full">
              <input v-model.number="inactivitySeconds" type="number" min="30" step="10"
                class="input input-bordered join-item flex-1 focus:input-primary" />
              <span class="btn btn-ghost join-item pointer-events-none">{{ t("general.inactivityUnit") }}</span>
            </div>
            <div class="label pt-1">
              <span class="label-text-alt text-base-content/50">{{ t("general.inactivityHint") }}</span>
            </div>
          </label>

          <KeyCapture v-model="postponeShortcut" :label="t('general.postponeShortcut')"
            :placeholder="t('general.postponeShortcutHint')" />
        </div>
      </div>
    </div>

    <!-- UI settings card -->
    <div class="card bg-base-100 shadow-xl border border-base-300">
      <div class="card-body">
        <h3 class="card-title text-lg mb-6 flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-primary" fill="none" viewBox="0 0 24 24"
            stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg>
          {{ t("general.uiSettings") }}
        </h3>
        <div class="grid gap-6 md:grid-cols-2">
          <label class="form-control w-full">
            <div class="label pb-2">
              <span class="label-text font-medium flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-info" fill="none" viewBox="0 0 24 24"
                  stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129" />
                </svg>
                {{ t("general.language") }}
              </span>
            </div>
            <select class="select select-bordered focus:select-primary w-full" :value="config.language"
              @change="onLanguageChange">
              <option v-for="locale in locales" :key="locale.key" :value="locale.key">
                {{ locale.label }}
              </option>
            </select>
          </label>

          <label class="form-control w-full">
            <div class="label pb-2">
              <span class="label-text font-medium flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-info" fill="none" viewBox="0 0 24 24"
                  stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
                </svg>
                {{ t("general.themeMode") }}
              </span>
            </div>
            <select class="select select-bordered focus:select-primary w-full" :value="config.themeMode"
              @change="(e) => configStore.setThemeMode((e.target as HTMLSelectElement).value as ThemeMode)">
              <option value="light">{{ t("general.themeModeLight") }}</option>
              <option value="dark">{{ t("general.themeModeDark") }}</option>
              <option value="system">{{ t("general.themeModeSystem") }}</option>
            </select>
          </label>

          <label class="form-control w-full">
            <div class="label pb-2">
              <span class="label-text font-medium flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-info" fill="none" viewBox="0 0 24 24"
                  stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
                </svg>
                {{ t("general.windowSize") }}
              </span>
            </div>
            <input v-model.number="windowSizePercent" type="range" min="50" max="100" step="5"
              class="range range-primary" />
            <div class="flex justify-between items-center mt-3 px-1">
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
