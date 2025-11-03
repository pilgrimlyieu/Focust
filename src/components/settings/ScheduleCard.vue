<script setup lang="ts">
import { useI18n } from "vue-i18n";
import CheckCircleIcon from "@/components/icons/CheckCircleIcon.vue";
import DuplicateIcon from "@/components/icons/DuplicateIcon.vue";
import GripVerticalIcon from "@/components/icons/GripVerticalIcon.vue";
import PauseCircleIcon from "@/components/icons/PauseCircleIcon.vue";
import TrashIcon from "@/components/icons/TrashIcon.vue";
import AudioPicker from "@/components/settings/AudioPicker.vue";
import SuggestionsToggle from "@/components/settings/SuggestionsToggle.vue";
import ThemeDesigner from "@/components/settings/ThemeDesigner.vue";
import { useSecondsToMinutes } from "@/composables/useComputed";
import type { ScheduleSettings } from "@/stores/config";

const props = defineProps<{
  schedule: ScheduleSettings;
  index: number;
  draggedIndex?: number | null;
}>();

const emit = defineEmits<{
  (event: "duplicate"): void;
  (event: "remove"): void;
  (event: "dragstart", dragEvent: DragEvent): void;
}>();

const { t } = useI18n();

const dayOrder = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] as const;

/** Duplicate the current schedule. */
function duplicateSchedule() {
  emit("duplicate");
}

/** Remove the current schedule. */
function removeSchedule() {
  emit("remove");
}

/**
 * Toggle the inclusion of a day in the schedule's active days.
 * @param {string} day The day to toggle (e.g., "Mon").
 */
function toggleDay(day: string) {
  const list = props.schedule.daysOfWeek;
  if (list.includes(day)) {
    props.schedule.daysOfWeek = list.filter((value) => value !== day);
  } else {
    props.schedule.daysOfWeek = [...list, day];
  }
}

const miniIntervalMinutes = useSecondsToMinutes(
  () => props.schedule.miniBreaks.intervalS,
  (value) => {
    props.schedule.miniBreaks.intervalS = value;
  },
);

const miniPostponeMinutes = useSecondsToMinutes(
  () => props.schedule.miniBreaks.postponedS,
  (value) => {
    props.schedule.miniBreaks.postponedS = value;
  },
);

const longPostponeMinutes = useSecondsToMinutes(
  () => props.schedule.longBreaks.postponedS,
  (value) => {
    props.schedule.longBreaks.postponedS = value;
  },
);
</script>

<template>
  <article
    class="group space-y-6 rounded-2xl border border-base-300 bg-linear-to-br from-base-100 to-base-200/30 p-6 shadow-md hover:shadow-xl transition-all">
    <!-- Header -->
    <header class="flex flex-wrap items-start gap-4">
      <div draggable="true" class="drag-handle flex items-center gap-3 shrink-0 cursor-grab active:cursor-grabbing"
        @dragstart="(e) => emit('dragstart', e)">
        <GripVerticalIcon
          class-name="h-5 w-5 text-base-content/20 group-hover:text-base-content/50 transition-colors" />
        <input v-model="schedule.enabled" type="checkbox" class="toggle toggle-lg transition-all"
          :class="{ 'toggle-success': schedule.enabled }" draggable="false" @mousedown.stop @click.stop />
      </div>
      <div class="flex-1 min-w-0">
        <input v-model="schedule.name" type="text"
          class="input input-ghost input-lg text-lg w-full font-semibold transition-all focus:input-primary"
          :placeholder="t('schedule.name')" />
        <p class="text-xs text-base-content/50 mt-2 flex items-center gap-1.5">
          <template v-if="schedule.enabled">
            <CheckCircleIcon class-name="h-3.5 w-3.5 text-success" />
            <span class="text-success font-medium">{{ t("schedule.enabledStatus") }}</span>
          </template>
          <template v-else>
            <PauseCircleIcon class-name="h-3.5 w-3.5 text-base-content/30" />
            <span class="text-base-content/50">{{ t("schedule.disabledStatus") }}</span>
          </template>
        </p>
      </div>
      <div class="flex gap-2 shrink-0">
        <button class="btn btn-sm btn-ghost gap-2 opacity-60 hover:opacity-100 transition-all"
          :title="t('actions.duplicate')" @click="duplicateSchedule">
          <DuplicateIcon class-name="h-4 w-4" />
          <span class="hidden sm:inline font-medium">{{ t("actions.duplicate") }}</span>
        </button>
        <button class="btn btn-sm btn-error btn-ghost gap-2 opacity-60 hover:opacity-100 transition-all"
          :title="t('actions.delete')" @click="removeSchedule">
          <TrashIcon class-name="h-4 w-4" />
          <span class="hidden sm:inline font-medium">{{ t("actions.delete") }}</span>
        </button>
      </div>
    </header>

    <!-- Time Range and Notification -->
    <section class="grid gap-4 md:grid-cols-3">
      <label class="form-control">
        <span class="label-text text-sm font-medium mb-2">{{ t("schedule.start") }}</span>
        <input v-model="schedule.timeRange.start" type="time"
          class="input input-bordered transition-all focus:input-primary" />
      </label>
      <label class="form-control">
        <span class="label-text text-sm font-medium mb-2">{{ t("schedule.end") }}</span>
        <input v-model="schedule.timeRange.end" type="time"
          class="input input-bordered transition-all focus:input-primary" />
      </label>
      <label class="form-control">
        <span class="label-text text-sm font-medium mb-2">{{ t("schedule.notifyBefore") }}</span>
        <div class="join w-full">
          <input v-model.number="schedule.notificationBeforeS" type="number" min="0"
            class="input input-bordered join-item flex-1 transition-all focus:input-primary" />
          <span class="btn btn-ghost join-item pointer-events-none text-sm">{{ t("schedule.secondsUnit") }}</span>
        </div>
      </label>
    </section>

    <!-- Days of Week -->
    <section class="rounded-xl bg-base-200/50 p-5">
      <span class="label-text text-sm font-medium mb-3 block">{{ t("schedule.days") }}</span>
      <div class="flex flex-wrap gap-2">
        <button v-for="day in dayOrder" :key="day" class="btn btn-sm min-w-14 transition-all font-medium"
          :class="schedule.daysOfWeek.includes(day) ? 'btn-primary shadow-md' : 'btn-outline btn-ghost'"
          @click="toggleDay(day)">
          {{ t(`days.${day}`) }}
        </button>
      </div>
    </section>

    <!-- Breaks Configuration -->
    <section class="grid gap-6 lg:grid-cols-2">
      <!-- Mini Break -->
      <div class="rounded-xl border border-base-300 bg-base-100/50 p-5 space-y-5">
        <h3 class="text-lg font-bold flex items-center gap-2">
          <span class="badge badge-primary badge-sm">MINI</span>
          {{ t("schedule.miniBreak") }}
        </h3>

        <!-- Mini Break Settings -->
        <div class="grid gap-4 sm:grid-cols-2">
          <label class="form-control">
            <span class="label-text text-xs font-medium mb-1.5">{{ t("schedule.intervalMinutes") }}</span>
            <div class="join w-full">
              <input v-model.number="miniIntervalMinutes" type="number" min="1"
                class="input input-sm input-bordered join-item flex-1 transition-all focus:input-primary" />
              <span class="btn btn-sm btn-ghost join-item pointer-events-none text-xs">{{ t("schedule.minutesUnit")
                }}</span>
            </div>
          </label>
          <label class="form-control">
            <span class="label-text text-xs font-medium mb-1.5">{{ t("schedule.durationSeconds") }}</span>
            <div class="join w-full">
              <input v-model.number="schedule.miniBreaks.durationS" type="number" min="1"
                class="input input-sm input-bordered join-item flex-1 transition-all focus:input-primary" />
              <span class="btn btn-sm btn-ghost join-item pointer-events-none text-xs">{{ t("schedule.secondsUnit")
                }}</span>
            </div>
          </label>
          <label class="form-control">
            <span class="label-text text-xs font-medium mb-1.5">{{ t("schedule.postponeMinutes") }}</span>
            <div class="join w-full">
              <input v-model.number="miniPostponeMinutes" type="number" min="1"
                class="input input-sm input-bordered join-item flex-1 transition-all focus:input-primary" />
              <span class="btn btn-sm btn-ghost join-item pointer-events-none text-xs">{{ t("schedule.minutesUnit")
                }}</span>
            </div>
          </label>
          <label class="label cursor-pointer justify-start gap-2 py-2">
            <input v-model="schedule.miniBreaks.strictMode" type="checkbox"
              class="checkbox checkbox-sm transition-all" />
            <span class="label-text text-xs font-medium">{{ t("schedule.strictMode") }}</span>
          </label>
        </div>

        <div class="divider my-3"></div>

        <ThemeDesigner :theme="schedule.miniBreaks.theme" :label="t('schedule.theme')" />
        <AudioPicker :audio="schedule.miniBreaks.audio" :label="t('schedule.audio')" />
        <SuggestionsToggle :suggestions="schedule.miniBreaks.suggestions" :label="t('schedule.suggestions')" />
      </div>

      <!-- Long Break -->
      <div class="rounded-xl border border-base-300 bg-base-100/50 p-5 space-y-5">
        <h3 class="text-lg font-bold flex items-center gap-2">
          <span class="badge badge-secondary badge-sm">LONG</span>
          {{ t("schedule.longBreak") }}
        </h3>

        <!-- Long Break Settings -->
        <div class="grid gap-4 sm:grid-cols-2">
          <label class="form-control">
            <span class="label-text text-xs font-medium mb-1.5">{{ t("schedule.durationSeconds") }}</span>
            <div class="join w-full">
              <input v-model.number="schedule.longBreaks.durationS" type="number" min="1"
                class="input input-sm input-bordered join-item flex-1 transition-all focus:input-primary" />
              <span class="btn btn-sm btn-ghost join-item pointer-events-none text-xs">{{ t("schedule.secondsUnit")
                }}</span>
            </div>
          </label>
          <label class="form-control">
            <span class="label-text text-xs font-medium mb-1.5">{{ t("schedule.postponeMinutes") }}</span>
            <div class="join w-full">
              <input v-model.number="longPostponeMinutes" type="number" min="1"
                class="input input-sm input-bordered join-item flex-1 transition-all focus:input-primary" />
              <span class="btn btn-sm btn-ghost join-item pointer-events-none text-xs">{{ t("schedule.minutesUnit")
                }}</span>
            </div>
          </label>
          <label class="form-control">
            <span class="label-text text-xs font-medium mb-1.5">{{ t("schedule.afterMiniBreaks") }}</span>
            <input v-model.number="schedule.longBreaks.afterMiniBreaks" type="number" min="1"
              class="input input-sm input-bordered transition-all focus:input-primary" />
          </label>
          <label class="label cursor-pointer justify-start gap-2 py-2">
            <input v-model="schedule.longBreaks.strictMode" type="checkbox"
              class="checkbox checkbox-sm transition-all" />
            <span class="label-text text-xs font-medium">{{ t("schedule.strictMode") }}</span>
          </label>
        </div>

        <div class="divider my-3"></div>

        <ThemeDesigner :theme="schedule.longBreaks.theme" :label="t('schedule.theme')" />
        <AudioPicker :audio="schedule.longBreaks.audio" :label="t('schedule.audio')" />
        <SuggestionsToggle :suggestions="schedule.longBreaks.suggestions" :label="t('schedule.suggestions')" />
      </div>
    </section>
  </article>
</template>
