<script setup lang="ts">
import { computed, ref, TransitionGroup } from "vue";
import { useI18n } from "vue-i18n";
import BellIcon from "@/components/icons/BellIcon.vue";
import CheckCircleIcon from "@/components/icons/CheckCircleIcon.vue";
import ChevronDownIcon from "@/components/icons/ChevronDownIcon.vue";
import CloseIcon from "@/components/icons/CloseIcon.vue";
import DuplicateIcon from "@/components/icons/DuplicateIcon.vue";
import InfoIcon from "@/components/icons/InfoIcon.vue";
import PauseCircleIcon from "@/components/icons/PauseCircleIcon.vue";
import PlusIcon from "@/components/icons/PlusIcon.vue";
import TrashIcon from "@/components/icons/TrashIcon.vue";
import ThemeDesigner from "@/components/settings/ThemeDesigner.vue";
import type { AppConfig } from "@/stores/config";
import { useConfigStore } from "@/stores/config";
import { safeClone } from "@/utils/safeClone";

const props = defineProps<{ config: AppConfig }>();

const { t } = useI18n();
const configStore = useConfigStore();
const dayOrder = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] as const;

const attentions = computed(() => props.config.attentions);
const expandedIds = ref<Set<number>>(new Set());

/**
 * Toggle the expanded state of an attention item.
 * @param {number} id The ID of the attention to toggle.
 */
function toggleExpand(id: number) {
  if (expandedIds.value.has(id)) {
    expandedIds.value.delete(id);
  } else {
    expandedIds.value.add(id);
  }
}

/**
 * Add a new attention reminder with default settings.
 */
function addAttention() {
  configStore.addAttention();
}

/**
 * Duplicate an attention reminder by its ID.
 * @param {number} id The ID of the attention to duplicate.
 */
function duplicateAttention(id: number) {
  const target = attentions.value.find((a) => a.id === id);
  if (!target) return;

  const newAttention = safeClone(target);
  newAttention.id = Date.now();
  newAttention.name = `${target.name} (${t("actions.copy")})`;
  props.config.attentions = [...attentions.value, newAttention];
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
</script>

<template>
  <section class="space-y-6">
    <!-- Header Card -->
    <div
      class="rounded-2xl border border-info/30 bg-linear-to-br from-info/10 via-info/5 to-transparent p-6 shadow-sm backdrop-blur-sm">
      <div class="flex flex-col sm:flex-row items-start gap-5">
        <div
          class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-linear-to-br from-info to-info/80 shadow-lg">
          <BellIcon class-name="h-7 w-7 text-white" />
        </div>
        <div class="flex-1 min-w-0">
          <h2 class="text-2xl font-bold text-base-content mb-2.5">
            {{ t("attention.title") }}
          </h2>
          <p class="text-sm text-base-content/70 leading-relaxed mb-4">
            {{ t("attention.description") }}
          </p>
          <div class="flex flex-wrap gap-2 items-center">
            <div class="badge badge-info badge-outline gap-1.5 py-3 px-3">
              <InfoIcon class-name="h-3.5 w-3.5" />
              <span class="text-xs font-medium">{{ t("attention.example") }}</span>
            </div>
            <div v-if="attentions.length" class="badge badge-ghost gap-1.5 py-3 px-3">
              <span class="font-semibold">{{ attentions.length }}</span>
              <span class="text-xs">{{ t("attention.totalCount") }}</span>
            </div>
          </div>
        </div>
        <button
          class="btn btn-primary gap-2.5 shadow-md hover:shadow-lg transition-all shrink-0 w-full sm:w-auto font-medium"
          @click="addAttention">
          <PlusIcon class-name="h-5 w-5" />
          {{ t("attention.create") }}
        </button>
      </div>
    </div>

    <!-- Empty State -->
    <div v-if="!attentions.length"
      class="rounded-2xl border-2 border-dashed border-base-300 bg-base-100/50 p-20 text-center">
      <div class="mx-auto max-w-md space-y-5">
        <div class="relative mx-auto h-28 w-28">
          <BellIcon class-name="h-28 w-28 text-base-content/10" />
          <div
            class="absolute right-0 top-0 flex h-10 w-10 items-center justify-center rounded-full bg-warning animate-pulse shadow-lg">
            <PlusIcon class-name="h-6 w-6 text-warning-content" />
          </div>
        </div>
        <div>
          <h3 class="text-xl font-semibold text-base-content/80 mb-3">
            {{ t("attention.empty") }}
          </h3>
          <p class="text-sm text-base-content/60 leading-relaxed">
            {{ t("attention.description") }}
          </p>
        </div>
        <button class="btn btn-primary btn-lg gap-2.5 shadow-lg hover:shadow-xl transition-all font-medium"
          @click="addAttention">
          <PlusIcon class-name="h-5 w-5" />
          {{ t("attention.create") }}
        </button>
      </div>
    </div>

    <!-- Attentions List -->
    <TransitionGroup v-else name="list" tag="div" class="space-y-5">
      <article v-for="(attention, index) in attentions" :key="attention.id"
        class="group rounded-2xl border border-base-300 bg-linear-to-br from-base-100 to-base-200/30 shadow-md transition-all hover:shadow-xl overflow-hidden">
        <!-- Header (Clickable) -->
        <header class="flex flex-wrap items-start gap-4 p-6 cursor-pointer" @click="toggleExpand(attention.id)">
          <div class="flex items-center gap-3 shrink-0" @click.stop>
            <input v-model="attention.enabled" type="checkbox" class="toggle toggle-lg transition-all"
              :class="{ 'toggle-success': attention.enabled }" @click.stop />
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-3">
              <input v-model="attention.name" type="text" :placeholder="t('attention.nameHint')"
                class="input input-ghost input-lg -ml-4 w-full font-bold text-xl focus:input-bordered transition-all"
                @click.stop />
              <ChevronDownIcon class-name="h-5 w-5 text-base-content/50 transition-transform duration-300 shrink-0"
                :class="{ 'rotate-180': expandedIds.has(attention.id) }" />
            </div>
            <div class="text-xs text-base-content/50 ml-0.5 mt-1.5 flex flex-wrap items-center gap-x-3 gap-y-1">
              <div class="flex items-center gap-1.5">
                <template v-if="attention.enabled">
                  <CheckCircleIcon class-name="h-3.5 w-3.5 text-success" />
                  <span class="text-success font-medium">{{ t("attention.enabledStatus") }}</span>
                </template>
                <template v-else>
                  <PauseCircleIcon class-name="h-3.5 w-3.5 text-base-content/30" />
                  <span class="text-base-content/50">{{ t("attention.disabledStatus") }}</span>
                </template>
              </div>
              <div class="badge badge-sm badge-ghost">
                {{ attention.times.length }} {{ t("attention.times") }}
              </div>
              <div class="badge badge-sm badge-ghost">
                {{ attention.daysOfWeek.length }} {{ t("attention.daysActive") }}
              </div>
            </div>
          </div>
          <div class="flex gap-2 shrink-0" @click.stop>
            <button class="btn btn-sm btn-ghost gap-2 opacity-60 hover:opacity-100 transition-all"
              :title="t('actions.duplicate')" @click="duplicateAttention(attention.id)">
              <DuplicateIcon class-name="h-4 w-4" />
              <span class="hidden sm:inline">{{ t("actions.duplicate") }}</span>
            </button>
            <button class="btn btn-sm btn-error btn-ghost gap-2 opacity-60 hover:opacity-100 transition-all"
              :title="t('actions.delete')" @click="removeAttention(attention.id)">
              <TrashIcon class-name="h-4 w-4" />
              <span class="hidden sm:inline">{{ t("actions.delete") }}</span>
            </button>
          </div>
        </header>

        <!-- Content (Collapsible) -->
        <Transition name="expand">
          <div v-if="expandedIds.has(attention.id)" class="space-y-6 px-6 pb-6">
            <!-- Basic Info -->
            <div class="grid gap-5 sm:grid-cols-2">
              <label class="form-control w-full">
                <div class="label pb-2">
                  <span class="label-text font-medium text-sm">{{ t("attention.titleLabel") }}</span>
                  <span class="label-text-alt text-base-content/50 text-xs">{{ t("attention.titleHint") }}</span>
                </div>
                <input v-model="attention.title" type="text"
                  class="input input-bordered focus:input-primary w-full transition-all"
                  :placeholder="t('attention.titleLabel')" />
              </label>
              <label class="form-control w-full">
                <div class="label pb-2">
                  <span class="label-text font-medium text-sm">{{ t("attention.durationSeconds") }}</span>
                </div>
                <div class="join w-full">
                  <input v-model.number="attention.durationS" type="number" min="5" max="300"
                    class="input input-bordered join-item flex-1 focus:input-primary transition-all" />
                  <span class="btn btn-ghost join-item pointer-events-none text-sm">{{ t("schedule.secondsUnit")
                    }}</span>
                </div>
              </label>
            </div>

            <!-- Message -->
            <label class="form-control w-full">
              <div class="label pb-2">
                <span class="label-text font-medium text-sm">{{ t("attention.message") }}</span>
                <span class="label-text-alt text-base-content/50 text-xs">{{ t("attention.messageHint") }}</span>
              </div>
              <textarea v-model="attention.message"
                class="textarea textarea-bordered textarea-lg h-24 resize-none focus:textarea-primary leading-relaxed w-full transition-all"
                :placeholder="t('attention.messagePlaceholder')" />
            </label>

            <!-- Days of Week -->
            <div class="rounded-xl bg-base-200/50 p-5">
              <div class="label pb-3">
                <span class="label-text font-medium text-sm">{{ t("attention.days") }}</span>
              </div>
              <div class="flex flex-wrap gap-2">
                <button v-for="day in dayOrder" :key="day" class="btn btn-sm min-w-14 transition-all font-medium"
                  :class="attention.daysOfWeek.includes(day) ? 'btn-primary shadow-md' : 'btn-ghost btn-outline'"
                  @click="toggleDay(index, day)">
                  {{ t(`days.${day}`) }}
                </button>
              </div>
            </div>

            <!-- Times -->
            <div class="rounded-xl bg-base-200/50 p-5">
              <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 pb-4">
                <div class="label pb-0">
                  <span class="label-text font-medium text-sm flex items-center gap-2">
                    {{ t("attention.times") }}
                    <span class="badge badge-sm badge-ghost font-normal">
                      {{ attention.times.length }}
                    </span>
                  </span>
                </div>
                <button class="btn btn-xs btn-primary gap-1.5 font-medium" @click="addTime(index)">
                  <PlusIcon class-name="h-3.5 w-3.5" />
                  {{ t("attention.addTime") }}
                </button>
              </div>
              <div v-if="attention.times.length" class="flex flex-wrap gap-3">
                <div v-for="(_, timeIdx) in attention.times" :key="timeIdx"
                  class="group/time flex items-center gap-2 rounded-lg border border-base-300 bg-base-100 p-2.5 shadow-sm hover:shadow-md transition-all">
                  <input v-model="attention.times[timeIdx]" type="time"
                    class="input input-sm input-ghost w-32 font-mono text-sm"
                    @input="updateTime(index, timeIdx, attention.times[timeIdx])" />
                  <button
                    class="btn btn-xs btn-ghost btn-circle text-error opacity-0 group-hover/time:opacity-100 transition-opacity"
                    :title="t('actions.delete')" @click="removeTime(index, timeIdx)">
                    <CloseIcon class-name="h-3.5 w-3.5" />
                  </button>
                </div>
              </div>
              <div v-else class="text-center py-6 text-sm text-base-content/40">
                {{ t("attention.addTimeHint") }}
              </div>
            </div>

            <!-- Theme Designer -->
            <div class="rounded-xl border border-base-300/50 bg-base-200/20 p-5">
              <div class="label pb-4">
                <span class="label-text font-medium text-base">{{ t("schedule.theme") }}</span>
              </div>
              <ThemeDesigner :theme="attention.theme" :label="t('schedule.theme')" />
            </div>
          </div>
        </Transition>
      </article>
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

.expand-enter-active,
.expand-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  max-height: 0;
}

.expand-enter-to,
.expand-leave-from {
  opacity: 1;
  max-height: 2000px;
}
</style>
