<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import AdvancedOption from "@/components/icons/AdvancedOption.vue";
import BellIcon from "@/components/icons/BellIcon.vue";
import ClockIcon from "@/components/icons/ClockIcon.vue";
import CloseIcon from "@/components/icons/CloseIcon.vue";
import ExternalLinkIcon from "@/components/icons/ExternalLinkIcon.vue";
import FolderIcon from "@/components/icons/FolderIcon.vue";
import InfoIcon from "@/components/icons/InfoIcon.vue";
import MonitorIcon from "@/components/icons/MonitorIcon.vue";
import type { ToastKind } from "@/composables/useToast";
import { useConfigStore } from "@/stores/config";
import type { EventKind } from "@/types";

const emit =
  defineEmits<(event: "notify", kind: ToastKind, message: string) => void>();

const { t } = useI18n();
const configStore = useConfigStore();

// TODO: Hide debug section in production builds

/**
 * Open the configuration directory in the system's file explorer.
 */
async function openConfigDirectory() {
  try {
    await invoke("open_config_directory");
    emit("notify", "success", t("toast.directoryOpened"));
  } catch (err) {
    console.error(err);
    emit("notify", "error", t("toast.openFailed"));
  }
}

/**
 * Open the log directory in the system's file explorer.
 */
async function openLogDirectory() {
  try {
    await invoke("open_log_directory");
    emit("notify", "success", t("toast.directoryOpened"));
  } catch (err) {
    console.error(err);
    emit("notify", "error", t("toast.openFailed"));
  }
}

/**
 * Trigger a mini break for testing purposes.
 */
async function triggerMiniBreak() {
  try {
    const config = configStore.draft ?? configStore.original;
    if (!config || config.schedules.length === 0) {
      emit("notify", "error", "There's no available schedule configuration");
      return;
    }
    // Use the first schedule's mini break, whether it's enabled.
    const breakKind: EventKind = {
      miniBreak: config.schedules[0].miniBreaks.id,
    };
    console.log("Triggering mini break with:", breakKind);
    await invoke("trigger_break", { breakKind });
    emit("notify", "success", "Mini break triggered");
  } catch (err) {
    console.error("Failed to trigger mini break:", err);
    emit("notify", "error", `Mini break triggered failed: ${err}`);
  }
}

/**
 * Trigger a long break for testing purposes.
 */
async function triggerLongBreak() {
  try {
    const config = configStore.draft ?? configStore.original;
    if (!config || config.schedules.length === 0) {
      emit("notify", "error", "There's no available schedule configuration");
      return;
    }
    const breakKind: EventKind = {
      longBreak: config.schedules[0].longBreaks.id,
    };
    console.log("Triggering long break with:", breakKind);
    await invoke("trigger_break", { breakKind });
    emit("notify", "success", "Long break triggered");
  } catch (err) {
    console.error("Failed to trigger long break:", err);
    emit("notify", "error", `Long break triggered failed: ${err}`);
  }
}

/**
 * Trigger an attention reminder for testing purposes.
 */
async function triggerAttention() {
  try {
    const config = configStore.draft ?? configStore.original;
    if (!config || config.attentions.length === 0) {
      emit("notify", "error", "There's no available attention configuration");
      return;
    }
    const breakKind: EventKind = {
      attention: config.attentions[0].id,
    };
    console.log("Triggering attention with:", breakKind);
    await invoke("trigger_break", { breakKind });
    emit("notify", "success", "Attention triggered");
  } catch (err) {
    console.error("Failed to trigger attention:", err);
    emit("notify", "error", `Attention triggered failed: ${err}`);
  }
}

/**
 * Skip the current break for testing purposes.
 */
async function skipCurrentBreak() {
  try {
    await invoke("skip_break");
    emit("notify", "success", "The current break has been skipped");
  } catch (err) {
    console.error("Failed to skip break:", err);
    emit("notify", "error", `Skip current break failed: ${err}`);
  }
}
</script>

<template>
  <section class="space-y-6">
    <!-- Header Card -->
    <div
      class="rounded-2xl border border-warning/30 bg-linear-to-br from-warning/10 via-warning/5 to-transparent p-6 shadow-sm backdrop-blur-sm">
      <div class="flex flex-col sm:flex-row items-start gap-5">
        <div
          class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-linear-to-br from-warning to-warning/80 shadow-lg">
          <AdvancedOption class-name="h-7 w-7 text-white" />
        </div>
        <div class="flex-1 min-w-0">
          <h2 class="text-2xl font-bold text-base-content mb-2.5">
            {{ t("advanced.title") }}
          </h2>
          <p class="text-sm text-base-content/70 leading-relaxed mb-4">
            {{ t("advanced.description") }}
          </p>
          <div class="flex flex-wrap gap-2 items-center">
            <div class="badge badge-warning badge-outline gap-1.5 py-3 px-3">
              <InfoIcon class-name="h-3.5 w-3.5" />
              <span class="text-xs font-medium">{{ t("advanced.breakWindowHint") }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Configuration Directory -->
    <div class="rounded-2xl border border-base-300 bg-base-100/70 p-6 shadow-md">
      <h3 class="text-lg font-bold mb-4 flex items-center gap-2">
        <FolderIcon class-name="h-5 w-5 text-primary" />
        {{ t("advanced.openConfigDir") }}
      </h3>
      <p class="text-sm text-base-content/60 mb-4">
        {{ t("advanced.openConfigDirDescription") }}
      </p>
      <button class="btn btn-primary gap-2 shadow-md hover:shadow-lg transition-all" @click="openConfigDirectory">
        <ExternalLinkIcon class-name="h-5 w-5" />
        {{ t("advanced.openConfigDir") }}
      </button>
    </div>

    <!-- Log Directory -->
    <div class="rounded-2xl border border-base-300 bg-base-100/70 p-6 shadow-md">
      <h3 class="text-lg font-bold mb-4 flex items-center gap-2">
        <FolderIcon class-name="h-5 w-5 text-info" />
        {{ t("advanced.openLogDir") }}
      </h3>
      <p class="text-sm text-base-content/60 mb-4">
        {{ t("advanced.openLogDirDescription") }}
      </p>
      <button class="btn btn-info gap-2 shadow-md hover:shadow-lg transition-all" @click="openLogDirectory">
        <ExternalLinkIcon class-name="h-5 w-5" />
        {{ t("advanced.openLogDir") }}
      </button>
    </div>

    <!-- Debug/Test Section -->
    <div class="rounded-2xl border border-base-300 bg-base-100/70 p-6 shadow-md">
      <h3 class="text-lg font-bold mb-4 flex items-center gap-2">
        <MonitorIcon class-name="h-5 w-5 text-warning" />
        🧪 Debug
      </h3>
      <p class="text-sm text-base-content/60 mb-4">
        Manually trigger break windows for testing
      </p>
      <div class="flex gap-2 flex-wrap">
        <button class="btn btn-sm btn-secondary gap-2 shadow-sm hover:shadow-md transition-all"
          @click="triggerMiniBreak">
          <ClockIcon class-name="h-4 w-4" />
          Trigger Mini Break (20s)
        </button>
        <button class="btn btn-sm btn-accent gap-2 shadow-sm hover:shadow-md transition-all" @click="triggerLongBreak">
          <ClockIcon class-name="h-4 w-4" />
          Trigger Long Break (5min)
        </button>
        <button class="btn btn-sm btn-info gap-2 shadow-sm hover:shadow-md transition-all" @click="triggerAttention">
          <BellIcon class-name="h-4 w-4" />
          Trigger Attention
        </button>
        <button class="btn btn-sm btn-error gap-2 shadow-sm hover:shadow-md transition-all" @click="skipCurrentBreak">
          <CloseIcon class-name="h-4 w-4" />
          Skip Current Break
        </button>
      </div>
    </div>
  </section>
</template>
