<script setup lang="ts">
import type { ToastItem } from "@/composables/useToast";

const emit = defineEmits<(event: "dismiss", id: number) => void>();

defineProps<{
  toasts: ToastItem[];
}>();

/**
 * Dismiss a toast by its ID.
 * @param id - The ID of the toast to dismiss.
 */
const dismiss = (id: number) => emit("dismiss", id);

defineExpose({ dismiss });
</script>

<template>
  <div class="pointer-events-none fixed inset-x-0 bottom-4 z-2000 flex justify-center">
    <transition-group name="toast" tag="div" class="flex flex-col gap-2">
      <div v-for="toast in $props.toasts" :key="toast.id"
        class="pointer-events-auto flex items-center gap-3 rounded-lg px-4 py-3 shadow-lg" :class="[
          toast.kind === 'success' && 'bg-success text-success-content',
          toast.kind === 'error' && 'bg-error text-error-content',
          toast.kind === 'info' && 'bg-info text-info-content',
        ]">
        <span class="text-sm font-medium">{{ toast.message }}</span>
        <button class="btn btn-xs" @click="dismiss(toast.id)">
          ✕
        </button>
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(10px);
}

.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
</style>
