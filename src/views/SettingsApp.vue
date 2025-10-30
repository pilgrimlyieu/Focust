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
import AdvancedOption from "@/components/icons/AdvancedOption.vue";
import AttentionBell from "@/components/icons/AttentionBell.vue";
import CleanCalendar from "@/components/icons/CleanCalendar.vue";
import SettingGear from "@/components/icons/SettingGear.vue";
import SuggestionBulb from "@/components/icons/SuggestionBulb.vue";
import ToastHost from "@/components/ui/ToastHost.vue";

const AdvancedPanel = defineAsyncComponent(
  () => import("@/components/settings/AdvancedPanel.vue"),
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

import type { ToastKind } from "@/composables/useToast";
import { useToast } from "@/composables/useToast";
import { useConfigStore } from "@/stores/config";
import { useSchedulerStore } from "@/stores/scheduler";
import {
  isAttention,
  isLongBreak,
  isMiniBreak,
  isNotificationKind,
} from "@/types/guards";

const { t } = useI18n();
const configStore = useConfigStore();
const schedulerStore = useSchedulerStore();

const tabs = [
  { key: "general", label: computed(() => t("nav.general")) },
  { key: "schedules", label: computed(() => t("nav.schedules")) },
  { key: "attentions", label: computed(() => t("nav.attentions")) },
  { key: "suggestions", label: computed(() => t("nav.suggestions")) },
  { key: "advanced", label: computed(() => t("nav.advanced")) },
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

onMounted(() => {
  intervalId = setInterval(() => {
    currentTime.value = Date.now();
  }, 1000);
});

onBeforeUnmount(() => {
  if (intervalId) {
    clearInterval(intervalId);
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
  let eventKind = event.kind;

  // If this is a Notification event, we need to calculate the actual break time
  // by adding the notification lead time (typically 5 seconds)
  // TODO: Maybe make it more reliable by fetching from schedule config?
  if (isNotificationKind(event.kind)) {
    const notifKind = event.kind.Notification;

    const schedule = configStore.draft?.schedules?.[0];
    const notifyBefore = schedule?.notificationBeforeS ?? 5;

    eventKind = notifKind;

    // The actual break happens AFTER the notification
    seconds += notifyBefore;
  }

  if (isMiniBreak(eventKind)) {
    kindStr = t("schedule.miniBreak");
  } else if (isLongBreak(eventKind)) {
    kindStr = t("schedule.longBreak");
  } else if (isAttention(eventKind)) {
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
 */
async function togglePause() {
  try {
    if (schedulerPaused.value) {
      await invoke("resume_scheduler");
      schedulerStore.setPaused(false);
    } else {
      await invoke("pause_scheduler");
      schedulerStore.setPaused(true);
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
    show("info", t("actions.postpone"));
  } catch (err) {
    console.error(err);
    show("error", "Failed to postpone");
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
            <!-- TODO: Use icon instead -->
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-white" fill="none" viewBox="0 0 24 24"
              stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <div>
            <h1 class="text-lg font-bold text-base-content sm:text-xl">{{ t("app.name") }}</h1>
            <p class="text-xs text-base-content/60 sm:text-sm">
              <span v-if="schedulerPaused" class="flex items-center gap-1">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24"
                  stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                {{ t("general.paused") }}
              </span>
              <span v-else-if="nextBreakInfo" class="flex items-center gap-1">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 text-success" fill="none" viewBox="0 0 24 24"
                  stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                {{ t("general.nextBreak", { kind: nextBreakInfo.kind, time: nextBreakInfo.timeRemaining }) }}
              </span>
              <span v-else class="flex items-center gap-1">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 text-success animate-pulse" fill="none"
                  viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                {{ t("general.running") }}
              </span>
            </p>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex flex-wrap items-center gap-2">
          <button class="btn btn-sm gap-2 btn-ghost hover:btn-primary" :class="{ 'btn-active': schedulerPaused }"
            @click="togglePause">
            <svg v-if="schedulerPaused" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none"
              viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
              stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span class="hidden sm:inline">{{ schedulerPaused ? t("actions.resume") : t("actions.pause") }}</span>
          </button>
          <button class="btn btn-sm gap-2 btn-ghost hover:btn-info" @click="handlePostpone">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
              stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span class="hidden sm:inline">{{ t("actions.postpone") }}</span>
          </button>
          <div class="divider divider-horizontal mx-0"></div>
          <button class="btn btn-sm gap-2 btn-ghost" :disabled="!isDirty" @click="handleReset">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
              stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <span class="hidden sm:inline">{{ t("actions.reset") }}</span>
          </button>
          <button class="btn btn-sm gap-2 btn-primary shadow-lg" :disabled="!isDirty || isSaving" @click="handleSave">
            <span v-if="isSaving" class="loading loading-spinner loading-xs" />
            <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
              stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
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
                <SettingGear v-if="index === 0" />
                <CleanCalendar v-else-if="index === 1" />
                <AttentionBell v-else-if="index === 2" />
                <SuggestionBulb v-else-if="index === 3" />
                <AdvancedOption v-else-if="index === 4" />
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
                  <GeneralSettingsPanel v-if="activeTab === 'general'" :key="'general'" :config="configStore.draft" />
                  <SchedulesPanel v-else-if="activeTab === 'schedules'" :key="'schedules'"
                    :config="configStore.draft" />
                  <AttentionsPanel v-else-if="activeTab === 'attentions'" :key="'attentions'"
                    :config="configStore.draft" />
                  <SuggestionsPanel v-else-if="activeTab === 'suggestions'" :key="'suggestions'" />
                  <AdvancedPanel v-else-if="activeTab === 'advanced'" :key="'advanced'" @notify="handleNotify" />
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
