<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import type { ToastKind } from "@/composables/useToast";
import { useConfigStore } from "@/stores/config";
import type { EventKind } from "@/types/generated/EventKind";

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
      MiniBreak: config.schedules[0].miniBreaks.id,
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
      LongBreak: config.schedules[0].longBreaks.id,
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
      Attention: config.attentions[0].id,
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

defineExpose({ openConfigDirectory });
</script>

<template>
  <section class="space-y-6">
    <header>
      <h2 class="text-xl font-semibold">{{ t("advanced.title") }}</h2>
      <p class="text-sm opacity-70">{{ t("advanced.description") }}</p>
    </header>

    <div class="grid gap-4">
      <button class="btn btn-primary" @click="openConfigDirectory">
        {{ t("advanced.openConfigDir") }}
      </button>
    </div>

    <section class="space-y-3">
      <h3 class="text-lg font-semibold">{{ t("advanced.window") }}</h3>
      <p class="text-sm opacity-70">{{ t("advanced.breakWindowHint") }}</p>
    </section>

    <!-- Debug/Test Section -->
    <section class="space-y-3">
      <h3 class="text-lg font-semibold">🧪 Debug</h3>
      <p class="text-sm opacity-70">Manually trigger break windows for testing</p>
      <div class="flex gap-2 flex-wrap">
        <button class="btn btn-secondary btn-sm" @click="triggerMiniBreak">
          Trigger Mini Break (20s)
        </button>
        <button class="btn btn-accent btn-sm" @click="triggerLongBreak">
          Trigger Long Break (5min)
        </button>
        <button class="btn btn-info btn-sm" @click="triggerAttention">
          Trigger Attention
        </button>
        <button class="btn btn-error btn-sm" @click="skipCurrentBreak">
          🚫 Skip Current Break
        </button>
      </div>
    </section>
  </section>
</template>
