<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { getErrorMessage } from "@/utils/handleError";
import BreakApp from "@/views/BreakApp.vue";

const ready = ref(false);
const error = ref<string | null>(null);

// Break app is the only view in the main window now
const currentView = computed(() => BreakApp);

function reload() {
  window.location.reload();
}

defineExpose({
  currentView,
  reload,
});

onMounted(async () => {
  try {
    ready.value = true;
  } catch (err) {
    error.value = getErrorMessage(err);
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

