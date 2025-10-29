<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import AudioPicker from "@/components/settings/AudioPicker.vue";
import SuggestionsToggle from "@/components/settings/SuggestionsToggle.vue";
import ThemeDesigner from "@/components/settings/ThemeDesigner.vue";
import type { ScheduleSettings } from "@/stores/config";

defineOptions({
  components: {
    AudioPicker,
    SuggestionsToggle,
    ThemeDesigner,
  },
});

const props = defineProps<{
  schedule: ScheduleSettings;
  index: number;
}>();

const emit = defineEmits<{
  (event: "duplicate"): void;
  (event: "remove"): void;
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

const miniIntervalMinutes = computed({
  get: () => Math.round(props.schedule.miniBreaks.intervalS / 60),
  set: (value: number) => {
    props.schedule.miniBreaks.intervalS = Math.max(1, Math.round(value)) * 60;
  },
});

const miniDurationMinutes = computed({
  get: () => Math.round(props.schedule.miniBreaks.durationS / 60),
  set: (value: number) => {
    props.schedule.miniBreaks.durationS = Math.max(1, Math.round(value)) * 60;
  },
});

const miniPostponeMinutes = computed({
  get: () => Math.round(props.schedule.miniBreaks.postponedS / 60),
  set: (value: number) => {
    props.schedule.miniBreaks.postponedS = Math.max(1, Math.round(value)) * 60;
  },
});

const longDurationMinutes = computed({
  get: () => Math.round(props.schedule.longBreaks.durationS / 60),
  set: (value: number) => {
    props.schedule.longBreaks.durationS = Math.max(1, Math.round(value)) * 60;
  },
});

const longPostponeMinutes = computed({
  get: () => Math.round(props.schedule.longBreaks.postponedS / 60),
  set: (value: number) => {
    props.schedule.longBreaks.postponedS = Math.max(1, Math.round(value)) * 60;
  },
});

defineExpose({
  dayOrder,
  duplicateSchedule,
  longDurationMinutes,
  longPostponeMinutes,
  miniDurationMinutes,
  miniIntervalMinutes,
  miniPostponeMinutes,
  removeSchedule,
  t,
  toggleDay,
});
</script>

<template>
  <article class="space-y-6 rounded-2xl border border-base-300 bg-base-100/70 p-6 shadow-sm">
    <header class="flex flex-wrap items-center justify-between gap-3">
      <div class="flex flex-col sm:flex-row items-start sm:items-center gap-3 flex-1">
        <label class="label cursor-pointer gap-3 shrink-0">
          <input v-model="schedule.enabled" type="checkbox" class="toggle toggle-lg shrink-0"
            :class="{ 'toggle-success': schedule.enabled }" />
        </label>
        <div class="flex flex-col gap-2 flex-1 min-w-0">
          <input v-model="schedule.name" type="text" class="input input-bordered text-lg w-full"
            :placeholder="t('schedule.name')" />
          <p class="text-xs flex items-center gap-1.5">
            <template v-if="schedule.enabled">
              <svg xmlns="http://www.w3.org/2000/svg" class="inline h-3 w-3 text-success" fill="none"
                viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span class="text-success">{{ t("schedule.enabledStatus") }}</span>
            </template>
            <template v-else>
              <svg xmlns="http://www.w3.org/2000/svg" class="inline h-3 w-3 text-base-content/30" fill="none"
                viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span class="text-base-content/50">{{ t("schedule.disabledStatus") }}</span>
            </template>
          </p>
        </div>
      </div>
      <div class="flex gap-2 shrink-0">
        <button class="btn btn-sm btn-ghost" @click="duplicateSchedule">{{ t("actions.duplicate") }}</button>
        <button class="btn btn-sm btn-error" @click="removeSchedule">{{ t("actions.delete") }}</button>
      </div>
    </header>

    <section class="grid gap-4 md:grid-cols-3">
      <label class="form-control">
        <span class="label-text">{{ t("schedule.start") }}</span>
        <input v-model="schedule.timeRange.start" type="time" class="input input-bordered" />
      </label>
      <label class="form-control">
        <span class="label-text">{{ t("schedule.end") }}</span>
        <input v-model="schedule.timeRange.end" type="time" class="input input-bordered" />
      </label>
      <label class="form-control">
        <span class="label-text">{{ t("schedule.notifyBefore") }}</span>
        <input v-model.number="schedule.notificationBeforeS" type="number" min="0" class="input input-bordered" />
      </label>
    </section>

    <section class="space-y-3">
      <span class="label-text">{{ t("schedule.days") }}</span>
      <div class="flex flex-wrap gap-2">
        <button v-for="day in dayOrder" :key="day" class="btn btn-sm" :class="schedule.daysOfWeek.includes(day) ? 'btn-primary' : 'btn-outline'
          " @click="toggleDay(day)">
          {{ t(`days.${day}`) }}
        </button>
      </div>
    </section>

    <section class="grid gap-6 md:grid-cols-2">
      <div class="space-y-4">
        <h3 class="text-lg font-semibold">{{ t("schedule.miniBreak") }}</h3>
        <div class="grid gap-4 md:grid-cols-2">
          <label class="form-control">
            <span class="label-text">{{ t("schedule.intervalMinutes") }}</span>
            <input v-model.number="miniIntervalMinutes" type="number" min="1" class="input input-bordered" />
          </label>
          <label class="form-control">
            <span class="label-text">{{ t("schedule.durationMinutes") }}</span>
            <input v-model.number="miniDurationMinutes" type="number" min="1" class="input input-bordered" />
          </label>
          <label class="form-control">
            <span class="label-text">{{ t("schedule.postponeMinutes") }}</span>
            <input v-model.number="miniPostponeMinutes" type="number" min="1" class="input input-bordered" />
          </label>
          <label class="label cursor-pointer justify-start gap-3">
            <input v-model="schedule.miniBreaks.strictMode" type="checkbox" class="checkbox" />
            <span>{{ t("schedule.strictMode") }}</span>
          </label>
        </div>
        <ThemeDesigner :theme="schedule.miniBreaks.theme" :label="t('schedule.theme')" />
        <AudioPicker :audio="schedule.miniBreaks.audio" :label="t('schedule.audio')" />
        <SuggestionsToggle :suggestions="schedule.miniBreaks.suggestions" :label="t('schedule.suggestions')" />
      </div>

      <div class="space-y-4">
        <h3 class="text-lg font-semibold">{{ t("schedule.longBreak") }}</h3>
        <div class="grid gap-4 md:grid-cols-2">
          <label class="form-control">
            <span class="label-text">{{ t("schedule.durationMinutes") }}</span>
            <input v-model.number="longDurationMinutes" type="number" min="1" class="input input-bordered" />
          </label>
          <label class="form-control">
            <span class="label-text">{{ t("schedule.postponeMinutes") }}</span>
            <input v-model.number="longPostponeMinutes" type="number" min="1" class="input input-bordered" />
          </label>
          <label class="form-control">
            <span class="label-text">{{ t("schedule.afterMiniBreaks") }}</span>
            <input v-model.number="schedule.longBreaks.afterMiniBreaks" type="number" min="1"
              class="input input-bordered" />
          </label>
          <label class="label cursor-pointer justify-start gap-3">
            <input v-model="schedule.longBreaks.strictMode" type="checkbox" class="checkbox" />
            <span>{{ t("schedule.strictMode") }}</span>
          </label>
        </div>
        <ThemeDesigner :theme="schedule.longBreaks.theme" :label="t('schedule.theme')" />
        <AudioPicker :audio="schedule.longBreaks.audio" :label="t('schedule.audio')" />
        <SuggestionsToggle :suggestions="schedule.longBreaks.suggestions" :label="t('schedule.suggestions')" />
      </div>
    </section>
  </article>
</template>
