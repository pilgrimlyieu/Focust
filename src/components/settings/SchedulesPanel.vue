<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import ScheduleCard from "@/components/settings/ScheduleCard.vue";
import { type AppConfig, useConfigStore } from "@/stores/config";

defineOptions({
  components: {
    ScheduleCard,
  },
});

const props = defineProps<{ config: AppConfig }>();

const { t } = useI18n();
const configStore = useConfigStore();

const schedules = computed(() => props.config.schedules);

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

defineExpose({ addSchedule, duplicateSchedule, removeSchedule, schedules, t });
</script>

<template>
  <section class="space-y-6">
    <header class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 class="text-xl font-semibold">{{ t("nav.schedules") }}</h2>
        <p class="text-sm opacity-70">{{ t("schedule.empty") }}</p>
      </div>
      <button class="btn btn-primary" @click="addSchedule()">
        {{ t("schedule.create") }}
      </button>
    </header>

    <div v-if="!schedules.length" class="rounded-xl border border-dashed border-base-300 p-8 text-center">
      <p class="text-sm opacity-70">{{ t("schedule.empty") }}</p>
    </div>
    <div v-else class="space-y-6">
      <ScheduleCard v-for="(schedule, index) in schedules" :key="schedule.miniBreaks.id" :schedule="schedule"
        :index="index" @duplicate="duplicateSchedule(schedule.miniBreaks.id)"
        @remove="removeSchedule(schedule.miniBreaks.id)" />
    </div>
  </section>
</template>
