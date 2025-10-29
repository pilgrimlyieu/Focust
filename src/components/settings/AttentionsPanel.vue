<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import ThemeDesigner from "@/components/settings/ThemeDesigner.vue";
import type { AppConfig } from "@/stores/config";
import { useConfigStore } from "@/stores/config";

defineOptions({
  components: {
    ThemeDesigner,
  },
});

const props = defineProps<{ config: AppConfig }>();

const { t } = useI18n();
const configStore = useConfigStore();
const dayOrder = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] as const;

const attentions = computed(() => props.config.attentions);

/**
 * Add a new attention reminder with default settings.
 */
function addAttention() {
  configStore.addAttention();
}

/**
 * Remove an attention reminder by its ID.
 * @param {number} id The ID of the attention to remove.
 */
function removeAttention(id: number) {
  configStore.removeAttention(id);
}

/**
 * Toggle a day of the week for a specific attention reminder.
 * @param {number} attentionIndex The index of the attention in the list.
 * @param {string} day The day to toggle (e.g., "Mon").
 */
function toggleDay(attentionIndex: number, day: string) {
  const target = attentions.value[attentionIndex];
  if (!target) return;
  if (target.daysOfWeek.includes(day)) {
    target.daysOfWeek = target.daysOfWeek.filter((value) => value !== day);
  } else {
    target.daysOfWeek = [...target.daysOfWeek, day];
  }
}

/**
 * Add a new time entry to a specific attention reminder.
 * @param {number} attentionIndex The index of the attention in the list.
 */
function addTime(attentionIndex: number) {
  const target = attentions.value[attentionIndex];
  if (!target) return;
  target.times = [...target.times, "12:00"]; // Default time is noon
}

/**
 * Remove a time entry from a specific attention reminder.
 * @param {number} attentionIndex The index of the attention in the list.
 * @param {number} timeIndex The index of the time entry to remove.
 */
function removeTime(attentionIndex: number, timeIndex: number) {
  const target = attentions.value[attentionIndex];
  if (!target) return;
  target.times = target.times.filter((_, idx) => idx !== timeIndex);
}

/**
 * Update a time entry for a specific attention reminder.
 * @param {number} attentionIndex The index of the attention in the list.
 * @param {number} timeIndex The index of the time entry to update.
 * @param {string} value The new time value in "HH:MM" format.
 */
function updateTime(attentionIndex: number, timeIndex: number, value: string) {
  const target = attentions.value[attentionIndex];
  if (!target) return;
  const sanitized = value.padStart(5, "0").slice(0, 5);
  target.times.splice(timeIndex, 1, sanitized);
}

defineExpose({
  addAttention,
  addTime,
  attentions,
  dayOrder,
  removeAttention,
  removeTime,
  t,
  toggleDay,
  updateTime,
});
</script>

<template>
  <section class="space-y-6">
    <div class="rounded-xl border border-info/20 bg-linear-to-br from-info/5 to-info/10 p-6 shadow-sm">
      <div class="flex flex-col sm:flex-row items-start gap-4">
        <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-full bg-info/20">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-7 w-7 text-info" fill="none" viewBox="0 0 24 24"
            stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
          </svg>
        </div>
        <div class="flex-1 min-w-0">
          <h2 class="text-2xl font-bold text-base-content mb-2">{{ t("attention.title") }}</h2>
          <p class="text-sm text-base-content/70 leading-relaxed mb-3">
            {{ t("attention.description") }}
          </p>
          <div class="flex flex-wrap gap-2">
            <div class="badge badge-info badge-outline gap-1">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              {{ t("attention.example") }}
            </div>
          </div>
        </div>
        <button class="btn btn-primary gap-2 shadow-lg shrink-0 w-full sm:w-auto" @click="addAttention">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          {{ t("attention.create") }}
        </button>
      </div>
    </div>

    <!-- Empty state with better illustration -->
    <div v-if="!attentions.length"
      class="rounded-2xl border-2 border-dashed border-base-300 bg-base-100/50 p-16 text-center backdrop-blur-sm">
      <div class="mx-auto max-w-md space-y-4">
        <div class="relative mx-auto h-24 w-24">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-24 w-24 text-base-content/10" fill="none" viewBox="0 0 24 24"
            stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
              d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
          </svg>
          <div
            class="absolute right-0 top-0 flex h-8 w-8 items-center justify-center rounded-full bg-warning animate-pulse">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-warning-content" fill="none" viewBox="0 0 24 24"
              stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </div>
        </div>
        <div>
          <h3 class="text-lg font-semibold text-base-content/80 mb-2">{{ t("attention.empty") }}</h3>
          <p class="text-sm text-base-content/50">{{ t("attention.description") }}</p>
        </div>
        <button class="btn btn-primary gap-2 shadow-lg" @click="addAttention">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          {{ t("attention.create") }}
        </button>
      </div>
    </div>

    <!-- Attentions list with better cards -->
    <div v-else class="space-y-5">
      <article v-for="(attention, index) in attentions" :key="attention.id"
        class="group rounded-2xl border border-base-300 bg-linear-to-br from-base-100 via-base-100 to-base-200/30 p-6 shadow-md transition-all hover:shadow-xl hover:scale-[1.01]">
        <!-- Header with toggle and delete -->
        <header class="mb-6 flex flex-wrap items-start gap-4">
          <div class="flex items-center gap-3 flex-1 min-w-0">
            <input v-model="attention.enabled" type="checkbox" class="toggle toggle-lg shrink-0"
              :class="{ 'toggle-success': attention.enabled }" />
            <div class="flex-1 min-w-0">
              <input v-model="attention.name" type="text" :placeholder="t('attention.nameHint')"
                class="input input-ghost input-lg -ml-4 w-full font-bold text-xl focus:input-bordered" />
              <p class="text-xs text-base-content/50 ml-0.5 mt-1">
                <template v-if="attention.enabled">
                  <svg xmlns="http://www.w3.org/2000/svg" class="inline h-3 w-3 text-success" fill="none"
                    viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  {{ t("attention.enabledStatus") }}
                </template>
                <template v-else>
                  <svg xmlns="http://www.w3.org/2000/svg" class="inline h-3 w-3 text-base-content/30" fill="none"
                    viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  {{ t("attention.disabledStatus") }}
                </template>
              </p>
            </div>
          </div>
          <div class="shrink-0">
            <button class="btn btn-sm btn-error btn-ghost gap-2 opacity-60 hover:opacity-100 transition-opacity"
              @click="removeAttention(attention.id)">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
              {{ t("actions.delete") }}
            </button>
          </div>
        </header>

        <!-- Content -->
        <div class="space-y-6">
          <!-- Basic info -->
          <div class="grid gap-5 sm:grid-cols-2">
            <label class="form-control w-full">
              <div class="label pb-2">
                <span class="label-text font-medium flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-primary" fill="none" viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
                  </svg>
                  {{ t("attention.titleLabel") }}
                </span>
                <span class="label-text-alt text-base-content/50">{{ t("attention.titleHint") }}</span>
              </div>
              <input v-model="attention.title" type="text" class="input input-bordered focus:input-primary w-full"
                :placeholder="t('attention.titleLabel')" />
            </label>
            <label class="form-control w-full">
              <div class="label pb-2">
                <span class="label-text font-medium flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-primary" fill="none" viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  {{ t("attention.durationSeconds") }}
                </span>
              </div>
              <input v-model.number="attention.durationS" type="number" min="5" max="300"
                class="input input-bordered focus:input-primary w-full" />
            </label>
          </div>

          <!-- Message -->
          <label class="form-control w-full">
            <div class="label pb-2">
              <span class="label-text font-medium flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-primary" fill="none" viewBox="0 0 24 24"
                  stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                </svg>
                {{ t("attention.message") }}
              </span>
              <span class="label-text-alt text-base-content/50">{{ t("attention.messageHint") }}</span>
            </div>
            <textarea v-model="attention.message"
              class="textarea textarea-bordered textarea-lg h-24 resize-none focus:textarea-primary leading-relaxed w-full"
              :placeholder="t('attention.messagePlaceholder')" />
          </label>

          <!-- Days of week -->
          <div class="rounded-xl bg-base-200/40 p-5">
            <div class="label pb-3">
              <span class="label-text font-medium flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-primary" fill="none" viewBox="0 0 24 24"
                  stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
                {{ t("attention.days") }}
              </span>
            </div>
            <div class="flex flex-wrap gap-2">
              <button v-for="day in dayOrder" :key="day" class="btn btn-sm min-w-14 transition-all"
                :class="attention.daysOfWeek.includes(day) ? 'btn-primary shadow-md' : 'btn-ghost btn-outline'"
                @click="toggleDay(index, day)">
                {{ t(`days.${day}`) }}
              </button>
            </div>
          </div>

          <!-- Times -->
          <div class="rounded-xl bg-base-200/40 p-5">
            <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 pb-4">
              <div class="label pb-0">
                <span class="label-text font-medium flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-primary" fill="none" viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  {{ t("attention.times") }}
                  <span class="badge badge-sm badge-ghost">{{ t("attention.timesCount", {
                    count: attention.times.length
                  }) }}</span>
                </span>
              </div>
              <button class="btn btn-xs btn-primary gap-1.5" @click="addTime(index)">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24"
                  stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                </svg>
                {{ t("attention.addTime") }}
              </button>
            </div>
            <div v-if="attention.times.length" class="flex flex-wrap gap-3">
              <div v-for="(_, timeIdx) in attention.times" :key="timeIdx"
                class="group/time flex items-center gap-2 rounded-lg border border-base-300 bg-base-100 p-2.5 shadow-sm hover:shadow-md transition-all">
                <input v-model="attention.times[timeIdx]" type="time" class="input input-sm input-ghost w-32 font-mono"
                  @input="updateTime(index, timeIdx, attention.times[timeIdx])" />
                <button
                  class="btn btn-xs btn-ghost btn-circle text-error opacity-0 group-hover/time:opacity-100 transition-opacity"
                  @click="removeTime(index, timeIdx)">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </div>
            <div v-else class="text-center py-6 text-sm text-base-content/40">
              {{ t("attention.addTimeHint") }}
            </div>
          </div>

          <!-- Theme designer -->
          <div class="rounded-xl border border-base-300/50 bg-base-200/20 p-5">
            <div class="label pb-4">
              <span class="label-text font-medium flex items-center gap-2 text-base">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-primary" fill="none" viewBox="0 0 24 24"
                  stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
                </svg>
                {{ t("schedule.theme") }}
              </span>
            </div>
            <ThemeDesigner :theme="attention.theme" :label="t('schedule.theme')" />
          </div>
        </div>
      </article>
    </div>
  </section>
</template>
