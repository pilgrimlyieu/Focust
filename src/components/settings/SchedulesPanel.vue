<script setup lang="ts">
import { computed, ref, TransitionGroup } from "vue";
import { useI18n } from "vue-i18n";
import CleanCalendar from "@/components/icons/CleanCalendar.vue";
import InfoIcon from "@/components/icons/InfoIcon.vue";
import PlusIcon from "@/components/icons/PlusIcon.vue";
import ScheduleCard from "@/components/settings/ScheduleCard.vue";
import { type AppConfig, useConfigStore } from "@/stores/config";

const props = defineProps<{ config: AppConfig }>();

const { t } = useI18n();
const configStore = useConfigStore();

const schedules = computed(() => props.config.schedules);
const draggedIndex = ref<number | null>(null);

/**
 * Handle drag start from the drag handle
 * @param {DragEvent} event The drag event
 * @param {number} index The index of the dragged item
 */
function handleDragStart(event: DragEvent, index: number) {
  // Only allow drag if started from the drag handle
  const target = event.target as HTMLElement;
  if (!target.classList.contains("drag-handle")) {
    event.preventDefault();
    return;
  }

  draggedIndex.value = index;
  // Set data transfer for compatibility
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", String(index));
  }
}

/**
 * Handle drag over
 * @param {DragEvent} event The drag event
 * @param {number} index The target index
 */
function handleDragOver(event: DragEvent, index: number) {
  event.preventDefault();
  if (
    draggedIndex.value === null ||
    draggedIndex.value === index ||
    !schedules.value
  )
    return;

  const items = [...schedules.value];
  const draggedItem = items[draggedIndex.value];
  items.splice(draggedIndex.value, 1);
  items.splice(index, 0, draggedItem);

  props.config.schedules = items;
  draggedIndex.value = index;
}

/**
 * Handle drag end
 */
function handleDragEnd() {
  draggedIndex.value = null;
}

/** Add a new schedule. */
function addSchedule() {
  configStore.addSchedule();
}

/**
 * Duplicate a schedule by its ID.
 * @param {number} id The ID of the schedule to duplicate.
 */
function duplicateSchedule(id: number) {
  configStore.duplicateSchedule(id);
}

/**
 * Remove a schedule by its ID.
 * @param {number} id The ID of the schedule to remove.
 */
function removeSchedule(id: number) {
  configStore.removeSchedule(id);
}
</script>

<template>
  <section class="space-y-6">
    <!-- Header Card -->
    <div
      class="rounded-2xl border border-primary/30 bg-linear-to-br from-primary/10 via-primary/5 to-transparent p-6 shadow-sm backdrop-blur-sm">
      <div class="flex flex-col sm:flex-row items-start gap-5">
        <div
          class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-linear-to-br from-primary to-primary/80 shadow-lg">
          <CleanCalendar class-name="h-7 w-7 text-white" />
        </div>
        <div class="flex-1 min-w-0">
          <h2 class="text-2xl font-bold text-base-content mb-2.5">
            {{ t("schedule.title") }}
          </h2>
          <p class="text-sm text-base-content/70 leading-relaxed mb-4">
            {{ t("schedule.description") }}
          </p>
          <div class="flex flex-wrap gap-2 items-center">
            <div class="badge badge-primary badge-outline gap-1.5 py-3 px-3">
              <InfoIcon class-name="h-3.5 w-3.5" />
              <span class="text-xs font-medium">{{ t("schedule.hint") }}</span>
            </div>
            <div v-if="schedules.length" class="badge badge-ghost gap-1.5 py-3 px-3">
              <span class="font-semibold">{{ schedules.length }}</span>
              <span class="text-xs">{{ t("schedule.totalCount") }}</span>
            </div>
          </div>
        </div>
        <button
          class="btn btn-primary gap-2.5 shadow-md hover:shadow-lg transition-all shrink-0 w-full sm:w-auto font-medium"
          @click="addSchedule()">
          <PlusIcon class-name="h-5 w-5" />
          {{ t("schedule.create") }}
        </button>
      </div>
    </div>

    <!-- Empty State -->
    <div v-if="!schedules.length"
      class="rounded-2xl border-2 border-dashed border-base-300 bg-base-100/50 p-20 text-center">
      <div class="mx-auto max-w-md space-y-5">
        <div class="relative mx-auto h-28 w-28">
          <CleanCalendar class-name="h-28 w-28 text-base-content/10" />
          <div
            class="absolute right-0 top-0 flex h-10 w-10 items-center justify-center rounded-full bg-warning animate-pulse shadow-lg">
            <PlusIcon class-name="h-6 w-6 text-warning-content" />
          </div>
        </div>
        <div>
          <h3 class="text-xl font-semibold text-base-content/80 mb-3">
            {{ t("schedule.empty") }}
          </h3>
          <p class="text-sm text-base-content/60 leading-relaxed">
            {{ t("schedule.description") }}
          </p>
        </div>
        <button class="btn btn-primary btn-lg gap-2.5 shadow-lg hover:shadow-xl transition-all font-medium"
          @click="addSchedule()">
          <PlusIcon class-name="h-5 w-5" />
          {{ t("schedule.create") }}
        </button>
      </div>
    </div>

    <!-- Schedules List -->
    <TransitionGroup v-else name="list" tag="div" class="space-y-6">
      <div v-for="(schedule, index) in schedules" :key="schedule.miniBreaks.id" :class="{
        'opacity-50 scale-95': draggedIndex === index,
        'ring-2 ring-primary/50 rounded-2xl': draggedIndex !== null && draggedIndex !== index,
      }" @dragover="handleDragOver($event, index)" @dragend="handleDragEnd">
        <ScheduleCard :schedule="schedule" :index="index" :dragged-index="draggedIndex"
          @duplicate="duplicateSchedule(schedule.miniBreaks.id)" @remove="removeSchedule(schedule.miniBreaks.id)"
          @dragstart="handleDragStart($event, index)" />
      </div>
    </TransitionGroup>
  </section>
</template>

<style scoped>
.list-move,
.list-enter-active,
.list-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.list-enter-from {
  opacity: 0;
  transform: translateY(-20px);
}

.list-leave-to {
  opacity: 0;
  transform: translateY(20px);
}

.list-leave-active {
  position: absolute;
  width: calc(100% - 2.5rem);
}
</style>
