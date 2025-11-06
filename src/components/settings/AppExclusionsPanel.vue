<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import DocumentIcon from "@/components/icons/DocumentIcon.vue";
import InfoIcon from "@/components/icons/InfoIcon.vue";
import ListIcon from "@/components/icons/ListIcon.vue";
import PauseCircleIcon from "@/components/icons/PauseCircleIcon.vue";
import PlayIcon from "@/components/icons/PlayIcon.vue";
import PlusIcon from "@/components/icons/PlusIcon.vue";
import TrashIcon from "@/components/icons/TrashIcon.vue";
import { useConfigStore } from "@/stores/config";
import type { AppExclusion, ExclusionRule } from "@/types";
import AppExclusionIcon from "../icons/AppExclusionIcon.vue";

const { t } = useI18n();
const configStore = useConfigStore();

// Get current config (draft or original)
const config = computed(() => configStore.draft ?? configStore.original);
const appExclusions = computed({
  get: () => config.value?.appExclusions ?? [],
  set: (value: AppExclusion[]) => {
    if (configStore.draft) {
      configStore.draft.appExclusions = value;
    }
  },
});

// Form state for adding new exclusion
const showAddForm = ref(false);
const newExclusionProcesses = ref("");
const newExclusionRule = ref<ExclusionRule>("pause");

// Editing state
const editingIndex = ref<number | null>(null);
const editingProcesses = ref("");

/** Add a new exclusion rule */
function addExclusion() {
  if (!newExclusionProcesses.value.trim()) {
    return;
  }

  const processes = newExclusionProcesses.value
    .split(",")
    .map((p) => p.trim())
    .filter((p) => p.length > 0);

  if (processes.length === 0) {
    return;
  }

  const newExclusion: AppExclusion = {
    active: true,
    processes,
    rule: newExclusionRule.value,
  };

  appExclusions.value = [...appExclusions.value, newExclusion];

  // Reset form
  newExclusionProcesses.value = "";
  newExclusionRule.value = "pause";
  showAddForm.value = false;
}

/**
 * Remove an exclusion rule
 * @param {number} index Index of the exclusion to remove
 */
function removeExclusion(index: number) {
  appExclusions.value = appExclusions.value.filter((_, i) => i !== index);
}

/**
 * Toggle exclusion active state
 * @param {number} index Index of the exclusion to toggle
 */
function toggleExclusion(index: number) {
  const exclusions = [...appExclusions.value];
  exclusions[index] = {
    ...exclusions[index],
    active: !exclusions[index].active,
  };
  appExclusions.value = exclusions;
}

/**
 * Start editing an exclusion
 * @param {number} index Index of the exclusion to edit
 */
function startEditing(index: number) {
  editingIndex.value = index;
  editingProcesses.value = appExclusions.value[index].processes.join(", ");
}

/**
 * Save edited exclusion
 * @param {number} index Index of the exclusion being edited
 */
function saveEdit(index: number) {
  const processes = editingProcesses.value
    .split(",")
    .map((p) => p.trim())
    .filter((p) => p.length > 0);

  if (processes.length === 0) {
    return;
  }

  const exclusions = [...appExclusions.value];
  exclusions[index] = {
    ...exclusions[index],
    processes,
  };
  appExclusions.value = exclusions;

  editingIndex.value = null;
  editingProcesses.value = "";
}

/** Cancel editing */
function cancelEdit() {
  editingIndex.value = null;
  editingProcesses.value = "";
}

/**
 * Toggle rule type for an exclusion
 * @param {number} index Index of the exclusion to toggle
 */
function toggleRule(index: number) {
  const exclusions = [...appExclusions.value];
  exclusions[index] = {
    ...exclusions[index],
    rule: exclusions[index].rule === "pause" ? "resume" : "pause",
  };
  appExclusions.value = exclusions;
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
          <AppExclusionIcon class-name="h-7 w-7 text-white" />
        </div>
        <div class="flex-1 min-w-0">
          <h2 class="text-2xl font-bold text-base-content mb-2.5">
            {{ t("appExclusions.title") }}
          </h2>
          <p class="text-sm text-base-content/70 leading-relaxed mb-4">
            {{ t("appExclusions.description") }}
          </p>
          <div class="flex flex-wrap gap-2 items-center">
            <div class="badge badge-info badge-outline gap-1.5 py-3 px-3">
              <InfoIcon class-name="h-3.5 w-3.5" />
              <span class="text-xs font-medium">{{ t("appExclusions.hint") }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Exclusion Rules List -->
    <div class="rounded-2xl border border-base-300 bg-base-100/70 p-6 shadow-md">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-bold flex items-center gap-2">
          <DocumentIcon class-name="h-5 w-5 text-info" />
          {{ t("appExclusions.rules") }}
        </h3>
        <button v-if="!showAddForm" class="btn btn-sm btn-primary gap-2" @click="showAddForm = true">
          <PlusIcon class-name="h-4 w-4" />
          {{ t("appExclusions.addRule") }}
        </button>
      </div>

      <!-- Add Form -->
      <div v-if="showAddForm" class="mb-4 p-4 rounded-xl border border-primary/30 bg-primary/5">
        <div class="space-y-3">
          <div>
            <label class="label">
              <span class="label-text font-medium">{{ t("appExclusions.ruleType") }}</span>
            </label>
            <div class="flex gap-2">
              <label class="label cursor-pointer gap-2 flex-1 justify-start">
                <input type="radio" value="pause" v-model="newExclusionRule" class="radio radio-primary" />
                <span class="label-text">
                  <PauseCircleIcon class-name="h-4 w-4 inline mr-1" />
                  {{ t("appExclusions.rulePause") }}
                </span>
              </label>
              <label class="label cursor-pointer gap-2 flex-1 justify-start">
                <input type="radio" value="resume" v-model="newExclusionRule" class="radio radio-primary" />
                <span class="label-text">
                  <PlayIcon class-name="h-4 w-4 inline mr-1" />
                  {{ t("appExclusions.ruleResume") }}
                </span>
              </label>
            </div>
          </div>

          <div>
            <label class="label">
              <span class="label-text font-medium">{{ t("appExclusions.processes") }}</span>
            </label>
            <input type="text" v-model="newExclusionProcesses" :placeholder="t('appExclusions.processesPlaceholder')"
              class="input input-bordered w-full" />
            <p class="text-xs text-base-content/60 mt-1">
              {{ t("appExclusions.processesHint") }}
            </p>
          </div>

          <div class="flex gap-2 justify-end">
            <button class="btn btn-sm btn-ghost" @click="showAddForm = false">
              {{ t("actions.cancel") }}
            </button>
            <button class="btn btn-sm btn-primary" @click="addExclusion" :disabled="!newExclusionProcesses.trim()">
              {{ t("actions.add") }}
            </button>
          </div>
        </div>
      </div>

      <!-- Exclusion List -->
      <div v-if="appExclusions.length === 0" class="text-center py-8 text-base-content/60">
        <ListIcon class-name="h-12 w-12 mx-auto mb-2 opacity-30" />
        <p>{{ t("appExclusions.noRules") }}</p>
      </div>

      <div v-else class="space-y-3">
        <div v-for="(exclusion, index) in appExclusions" :key="index" class="p-4 rounded-xl border" :class="[
            exclusion.active
              ? 'border-base-300 bg-base-100'
              : 'border-base-200 bg-base-200/50 opacity-60',
          ]">
          <div class="flex items-start gap-3">
            <!-- Active Toggle -->
            <input type="checkbox" :checked="exclusion.active" @change="toggleExclusion(index)"
              class="checkbox checkbox-primary mt-1" />

            <div class="flex-1 min-w-0">
              <!-- Rule Type Badge -->
              <div class="flex items-center gap-2 mb-2">
                <span class="badge gap-1.5" :class="[
                    exclusion.rule === 'pause'
                      ? 'badge-warning'
                      : 'badge-success',
                  ]">
                  <PauseCircleIcon v-if="exclusion.rule === 'pause'" class-name="h-3.5 w-3.5" />
                  <PlayIcon v-else class-name="h-3.5 w-3.5" />
                  {{
                  exclusion.rule === "pause"
                  ? t("appExclusions.rulePause")
                  : t("appExclusions.ruleResume")
                  }}
                </span>
                <button class="btn btn-xs btn-ghost" @click="toggleRule(index)" :title="t('appExclusions.toggleRule')">
                  🔄
                </button>
              </div>

              <!-- Processes List -->
              <div v-if="editingIndex !== index">
                <div class="flex flex-wrap gap-1.5">
                  <span v-for="(process, pIndex) in exclusion.processes" :key="pIndex"
                    class="badge badge-outline badge-sm">
                    {{ process }}
                  </span>
                </div>
              </div>

              <!-- Edit Form -->
              <div v-else class="space-y-2">
                <input type="text" v-model="editingProcesses" class="input input-sm input-bordered w-full"
                  @keyup.enter="saveEdit(index)" @keyup.esc="cancelEdit()" />
                <div class="flex gap-2">
                  <button class="btn btn-xs btn-ghost" @click="cancelEdit()">
                    {{ t("actions.cancel") }}
                  </button>
                  <button class="btn btn-xs btn-primary" @click="saveEdit(index)" :disabled="!editingProcesses.trim()">
                    {{ t("actions.save") }}
                  </button>
                </div>
              </div>
            </div>

            <!-- Action Buttons -->
            <div class="flex gap-1">
              <button v-if="editingIndex !== index" class="btn btn-sm btn-ghost btn-square" @click="startEditing(index)"
                :title="t('actions.edit')">
                ✏️
              </button>
              <button class="btn btn-sm btn-ghost btn-square text-error" @click="removeExclusion(index)"
                :title="t('actions.delete')">
                <TrashIcon class-name="h-4 w-4" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Info Section -->
    <div class="rounded-2xl border border-base-300 bg-base-100/70 p-6 shadow-md">
      <h3 class="text-lg font-bold mb-3 flex items-center gap-2">
        <InfoIcon class-name="h-5 w-5 text-info" />
        {{ t("appExclusions.howItWorks") }}
      </h3>
      <div class="space-y-3 text-sm text-base-content/80">
        <div class="flex gap-3">
          <PauseCircleIcon class-name="h-5 w-5 text-warning shrink-0 mt-0.5" />
          <div>
            <p class="font-medium">{{ t("appExclusions.rulePause") }}</p>
            <p class="text-base-content/60">{{ t("appExclusions.rulePauseExplanation") }}</p>
          </div>
        </div>
        <div class="flex gap-3">
          <PlayIcon class-name="h-5 w-5 text-success shrink-0 mt-0.5" />
          <div>
            <p class="font-medium">{{ t("appExclusions.ruleResume") }}</p>
            <p class="text-base-content/60">{{ t("appExclusions.ruleResumeExplanation") }}</p>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
