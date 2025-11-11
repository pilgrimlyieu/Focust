<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import {
  computed,
  defineAsyncComponent,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from "vue";
import { useI18n } from "vue-i18n";
import CheckCircleIcon from "@/components/icons/CheckCircleIcon.vue";
import CheckIcon from "@/components/icons/CheckIcon.vue";
import CleanCalendar from "@/components/icons/CleanCalendar.vue";
import ClockIcon from "@/components/icons/ClockIcon.vue";
import PauseIcon from "@/components/icons/PauseIcon.vue";
import PlayIcon from "@/components/icons/PlayIcon.vue";
import RefreshIcon from "@/components/icons/RefreshIcon.vue";
import SettingGear from "@/components/icons/SettingGear.vue";
import SuggestionBulb from "@/components/icons/SuggestionBulb.vue";
import ToastHost from "@/components/ui/ToastHost.vue";

const AdvancedPanel = defineAsyncComponent(
  () => import("@/components/settings/AdvancedPanel.vue"),
);
const AboutPanel = defineAsyncComponent(
  () => import("@/components/settings/AboutPanel.vue"),
);
const AppExclusionsPanel = defineAsyncComponent(
  () => import("@/components/settings/AppExclusionsPanel.vue"),
);
const AttentionsPanel = defineAsyncComponent(
  () => import("@/components/settings/AttentionsPanel.vue"),
);
const GeneralSettingsPanel = defineAsyncComponent(
  () => import("@/components/settings/GeneralSettingsPanel.vue"),
);
const SchedulesPanel = defineAsyncComponent(
  () => import("@/components/settings/SchedulesPanel.vue"),
);
const SuggestionsPanel = defineAsyncComponent(
  () => import("@/components/settings/SuggestionsPanel.vue"),
);

import { listen } from "@tauri-apps/api/event";
import AppExclusionIcon from "@/components/icons/AppExclusionIcon.vue";
import BellIcon from "@/components/icons/BellIcon.vue";
import InfoCircleIcon from "@/components/icons/InfoCircleIcon.vue";
import SlidersIcon from "@/components/icons/SlidersIcon.vue";
import type { ToastKind } from "@/composables/useToast";
import { useToast } from "@/composables/useToast";
import { useConfigStore } from "@/stores/config";
import { useSchedulerStore } from "@/stores/scheduler";
import {
  isSchedulerAttention,
  isSchedulerLongBreak,
  isSchedulerMiniBreak,
} from "@/types";

const { t } = useI18n();
const configStore = useConfigStore();
const schedulerStore = useSchedulerStore();

const tabs = [
  { key: "general", label: computed(() => t("nav.general")) },
  { key: "schedules", label: computed(() => t("nav.schedules")) },
  { key: "attentions", label: computed(() => t("nav.attentions")) },
  { key: "suggestions", label: computed(() => t("nav.suggestions")) },
  { key: "exclusions", label: computed(() => t("nav.exclusions")) },
  { key: "advanced", label: computed(() => t("nav.advanced")) },
  { key: "about", label: computed(() => t("nav.about")) },
] as const;

type TabKey = (typeof tabs)[number]["key"];
const activeTab = ref<TabKey>("general");

const { toasts, show, dismiss } = useToast();
const isDirty = computed(() => configStore.isDirty);
const isSaving = computed(() => configStore.saving);
const isLoading = computed(() => configStore.loading);

const schedulerPaused = computed(() => schedulerStore.schedulerPaused);
const schedulerStatus = computed(() => schedulerStore.schedulerStatus);

// Track the base time when status was received
const statusReceivedTime = ref<number>(Date.now());
const currentTime = ref<number>(Date.now());

// Update current time every second for live countdown
let intervalId: ReturnType<typeof setInterval> | null = null;

// Listen for postpone limit reached event
let unlistenPostponeLimit: (() => void) | null = null;

onMounted(async () => {
  intervalId = setInterval(() => {
    currentTime.value = Date.now();
  }, 1000);

  unlistenPostponeLimit = await listen("postpone-limit-reached", () => {
    console.log("[PromptApp] Postpone limit reached");
    show("info", t("break.noMorePostpone"), 3000);
  });
});

onBeforeUnmount(() => {
  if (intervalId) {
    clearInterval(intervalId);
  }
  if (unlistenPostponeLimit) {
    unlistenPostponeLimit();
  }
});

// Watch for status changes to update the base time
watch(schedulerStatus, () => {
  if (schedulerStatus.value?.nextEvent) {
    statusReceivedTime.value = Date.now();
  }
});

const nextBreakInfo = computed(() => {
  const status = schedulerStatus.value;

  if (!status || !status.nextEvent) {
    return null;
  }

  const event = status.nextEvent;

  // Calculate elapsed seconds since status was received
  const elapsedSeconds = Math.floor(
    (currentTime.value - statusReceivedTime.value) / 1000,
  );

  // Calculate remaining seconds, accounting for elapsed time
  let seconds = Math.max(0, Number(event.secondsUntil) - elapsedSeconds);
  let kindStr = "";

  // Determine the event kind and display string
  if (isSchedulerMiniBreak(event.kind)) {
    kindStr = t("schedule.miniBreak");
  } else if (isSchedulerLongBreak(event.kind)) {
    kindStr = t("schedule.longBreak");
  } else if (isSchedulerAttention(event.kind)) {
    kindStr = t("break.attention");
  }

  // Format time remaining in a human-readable way
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  let timeStr = "";
  if (hours > 0) {
    timeStr = t("general.hoursMinutes", { hours, minutes });
  } else if (minutes >= 2) {
    // Show minutes for 2+ minutes
    timeStr = t("general.minutesRemaining", { minutes });
  } else if (seconds > 0) {
    // Show seconds for less than 2 minutes
    timeStr = t("general.secondsRemaining", { seconds });
  } else {
    timeStr = t("general.imminent");
  }

  return {
    kind: kindStr,
    timeRemaining: timeStr,
  };
});

/**
 * Handle saving the configuration.
 */
async function handleSave() {
  try {
    await configStore.save();
    show("success", t("toast.saved"));
  } catch (err) {
    console.error(err);
    show("error", t("toast.saveFailed"));
  }
}

/**
 * Handle resetting the configuration draft.
 */
function handleReset() {
  configStore.resetDraft();
  show("info", t("general.postponeHint"));
}

/**
 * Toggle pausing/resuming the scheduler.
 *
 * Note: We don't manually update the state here. The backend will emit
 * scheduler-status events that update the frontend state automatically.
 * This ensures the frontend always reflects the actual backend state.
 */
async function togglePause() {
  try {
    if (schedulerPaused.value) {
      await invoke("resume_scheduler");
      // State will be updated via scheduler-status event
    } else {
      await invoke("pause_scheduler");
      // State will be updated via scheduler-status event
    }
  } catch (err) {
    console.error(err);
    show("error", "Failed to update scheduler state");
  }
}

/**
 * Postpone the upcoming break.
 */
async function handlePostpone() {
  try {
    await invoke("postpone_break");
  } catch (err) {
    console.error(err);
  }
}

/**
 * Handle notification from child components.
 * @param {ToastKind} kind The kind of toast notification.
 * @param {string} message The message to display.
 */
function handleNotify(kind: ToastKind, message: string) {
  show(kind, message);
}

// Initialize config and scheduler when settings window opens
onMounted(async () => {
  try {
    if (!configStore.draft) {
      await configStore.load();
    }
    await schedulerStore.init();

    if (configStore.draft) {
      show("success", t("toast.loaded"));
    }
  } catch (err) {
    console.error("Failed to initialize settings:", err);
    show("error", t("toast.loadFailed"));
  }
});

watch(
  () => configStore.error,
  (err) => {
    if (err) {
      show("error", err);
    }
  },
);

defineExpose({
  activeTab,
  dismiss,
  handleNotify,
  handlePostpone,
  handleReset,
  handleSave,
  isDirty,
  isLoading,
  isSaving,
  schedulerPaused,
  tabs,
  toasts,
  togglePause,
});
</script>

<template>
  <div class="flex min-h-screen flex-col bg-linear-to-br from-base-200 via-base-100 to-base-200">
    <header class="sticky top-0 z-50 border-b border-base-300/50 bg-base-100/95 backdrop-blur-md shadow-sm">
      <div class="mx-auto flex w-full max-w-7xl items-center justify-between gap-4 px-4 py-3 sm:px-6 lg:px-8">
        <!-- App info with icon -->
        <div class="flex items-center gap-3">
          <div
            class="flex h-10 w-10 items-center justify-center rounded-xl bg-linear-to-br from-primary to-secondary shadow-lg">
            <ClockIcon class-name="h-6 w-6 text-white" />
          </div>
          <div>
            <h1 class="text-lg font-bold text-base-content sm:text-xl">{{ t("app.name") }}</h1>
            <p class="text-xs text-base-content/60 sm:text-sm">
              <span v-if="schedulerPaused" class="flex items-center gap-1">
                <PauseIcon class-name="h-3 w-3" />
                {{ t("general.paused") }}
              </span>
              <span v-else-if="nextBreakInfo" class="flex items-center gap-1">
                <CheckCircleIcon class-name="h-3 w-3 text-success" />
                {{ t("general.nextBreak", { kind: nextBreakInfo.kind, time: nextBreakInfo.timeRemaining }) }}
              </span>
              <span v-else class="flex items-center gap-1">
                <PlayIcon class-name="h-3 w-3 text-success animate-pulse" />
                {{ t("general.running") }}
              </span>
            </p>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex flex-wrap items-center gap-2">
          <button class="btn btn-sm gap-2 btn-ghost hover:btn-primary" :class="{ 'btn-active': schedulerPaused }"
            @click="togglePause">
            <PlayIcon v-if="schedulerPaused" class-name="h-4 w-4" />
            <PauseIcon v-else class-name="h-4 w-4" />
            <span class="hidden sm:inline">{{ schedulerPaused ? t("actions.resume") : t("actions.pause") }}</span>
          </button>
          <button class="btn btn-sm gap-2 btn-ghost hover:btn-info" :disabled="schedulerPaused" @click="handlePostpone">
            <ClockIcon class-name="h-4 w-4" />
            <span class="hidden sm:inline">{{ t("actions.postpone") }}</span>
          </button>
          <div class="divider divider-horizontal mx-0"></div>
          <button class="btn btn-sm gap-2 btn-ghost" :disabled="!isDirty" @click="handleReset">
            <RefreshIcon class-name="h-4 w-4" />
            <span class="hidden sm:inline">{{ t("actions.reset") }}</span>
          </button>
          <button class="btn btn-sm gap-2 btn-primary shadow-lg" :disabled="!isDirty || isSaving" @click="handleSave">
            <span v-if="isSaving" class="loading loading-spinner loading-xs" />
            <CheckIcon v-else class-name="h-4 w-4" />
            <span>{{ t("actions.save") }}</span>
          </button>
        </div>
      </div>
    </header>

    <div class="flex flex-1">
      <!-- Sidebar navigation with icons -->
      <aside class="hidden w-64 border-r border-base-300/50 bg-base-100/40 backdrop-blur-sm lg:block">
        <nav class="sticky top-20 p-4">
          <ul class="menu gap-2">
            <li v-for="(tab, index) in tabs" :key="tab.key">
              <a class="group flex items-center gap-3 rounded-lg transition-all hover:scale-105" :class="activeTab === tab.key
                ? 'active bg-linear-to-r from-primary to-secondary text-primary-content font-semibold shadow-lg'
                : 'hover:bg-base-200'" @click="activeTab = tab.key">
                <!-- Icon -->
                <SettingGear :class-name="'h-5 w-5'" v-if="index === 0" />
                <CleanCalendar :class-name="'h-5 w-5'" v-else-if="index === 1" />
                <BellIcon :class-name="'h-5 w-5'" v-else-if="index === 2" />
                <SuggestionBulb :class-name="'h-5 w-5'" v-else-if="index === 3" />
                <AppExclusionIcon :class-name="'h-5 w-5'" v-else-if="index === 4" />
                <SlidersIcon :class-name="'h-5 w-5'" v-else-if="index === 5" />
                <InfoCircleIcon :class-name="'h-5 w-5'" v-else-if="index === 6" />
                <span>{{ tab.label }}</span>
              </a>
            </li>
          </ul>
        </nav>
      </aside>

      <!-- Mobile tabs -->
      <div class="lg:hidden w-full">
        <div class="tabs tabs-boxed sticky top-16 z-40 bg-base-100/95 backdrop-blur-md p-2 overflow-x-auto">
          <a v-for="tab in tabs" :key="tab.key" class="tab tab-sm whitespace-nowrap"
            :class="{ 'tab-active': activeTab === tab.key }" @click="activeTab = tab.key">
            {{ tab.label }}
          </a>
        </div>
      </div>

      <!-- Main content -->
      <main class="flex-1 overflow-y-auto">
        <div class="mx-auto w-full max-w-5xl px-4 py-6 sm:px-6 lg:px-8 lg:py-8">
          <div v-if="isLoading" class="flex flex-col items-center justify-center gap-4 py-32">
            <span class="loading loading-bars loading-lg text-primary" />
            <p class="text-sm text-base-content/60">Loading configuration…</p>
          </div>
          <template v-else-if="configStore.draft">
            <Suspense>
              <template #default>
                <Transition name="fade" mode="out-in">
                  <GeneralSettingsPanel v-if="activeTab === 'general'" :key="'general'" :config="configStore.draft"
                    @notify="handleNotify" />
                  <SchedulesPanel v-else-if="activeTab === 'schedules'" :key="'schedules'"
                    :config="configStore.draft" />
                  <AttentionsPanel v-else-if="activeTab === 'attentions'" :key="'attentions'"
                    :config="configStore.draft" />
                  <SuggestionsPanel v-else-if="activeTab === 'suggestions'" :key="'suggestions'" />
                  <AppExclusionsPanel v-else-if="activeTab === 'exclusions'" :key="'exclusions'" />
                  <AdvancedPanel v-else-if="activeTab === 'advanced'" :key="'advanced'" @notify="handleNotify" />
                  <AboutPanel v-else-if="activeTab === 'about'" :key="'about'" @notify="handleNotify" />
                </Transition>
              </template>
              <template #fallback>
                <div class="flex flex-col items-center justify-center gap-4 py-32">
                  <span class="loading loading-spinner loading-md text-primary" />
                  <p class="text-sm text-base-content/60">Loading panel…</p>
                </div>
              </template>
            </Suspense>
          </template>
        </div>
      </main>
    </div>
    <ToastHost :toasts="toasts" @dismiss="dismiss" />
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
