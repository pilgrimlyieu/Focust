<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useConfigStore } from "@/stores/config";
import { useSchedulerStore } from "@/stores/scheduler";
import { getErrorMessage } from "@/utils/handleError";
import BreakApp from "@/views/BreakApp.vue";
import SettingsApp from "@/views/SettingsApp.vue";

type AppView = "settings" | "break";

const params = new URLSearchParams(window.location.search);
const view = (params.get("view") as AppView) ?? "settings";

const ready = ref(false);
const error = ref<string | null>(null);

const currentView = computed(() => (view === "break" ? BreakApp : SettingsApp));

function reload() {
  window.location.reload();
}

defineExpose({
  currentView,
  reload,
});

onMounted(async () => {
  try {
    if (view === "settings") {
      const configStore = useConfigStore();
      await configStore.load();
      const schedulerStore = useSchedulerStore();
      await schedulerStore.init();
    }
  } catch (err) {
    error.value = getErrorMessage(err);
  } finally {
    ready.value = true;
  }
});
</script>

<template>
  <div class="min-h-screen bg-base-200 text-base-content">
    <div v-if="!ready" class="flex min-h-screen flex-col items-center justify-center gap-4">
      <span class="loading loading-spinner loading-lg text-primary" />
      <p class="text-sm opacity-70">Bootstrapping...</p>
    </div>
    <div v-else-if="error" class="flex min-h-screen flex-col items-center justify-center gap-3">
      <h1 class="text-2xl font-semibold">Something went wrong</h1>
      <p class="max-w-md text-center text-sm opacity-70">{{ error }}</p>
      <button class="btn" @click="reload">Reload</button>
    </div>
    <component v-else :is="currentView" />
  </div>
</template>

