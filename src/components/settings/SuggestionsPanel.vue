<script setup lang="ts">
import { computed, onMounted, ref, TransitionGroup, watch } from "vue";
import { useI18n } from "vue-i18n";
import CloseIcon from "@/components/icons/CloseIcon.vue";
import DocumentIcon from "@/components/icons/DocumentIcon.vue";
import GripVerticalIcon from "@/components/icons/GripVerticalIcon.vue";
import ImportIcon from "@/components/icons/ImportIcon.vue";
import InfoIcon from "@/components/icons/InfoIcon.vue";
import ListIcon from "@/components/icons/ListIcon.vue";
import PlusIcon from "@/components/icons/PlusIcon.vue";
import SuggestionBulb from "@/components/icons/SuggestionBulb.vue";
import { getI18nLocale } from "@/i18n";
import { useSuggestionsStore } from "@/stores/suggestions";

const { t } = useI18n();
const suggestionsStore = useSuggestionsStore();

onMounted(() => {
  if (!suggestionsStore.hasLoaded) {
    suggestionsStore.load();
  }
});

const currentLanguage = computed(getI18nLocale);

/** UI mode: 'list' or 'bulk' */
const editMode = ref<"list" | "bulk">("list");

/** Suggestions as array (for list mode) */
const suggestionsList = ref<string[]>([]);

/** Suggestions as text (for bulk mode) */
const suggestionsText = ref("");

const newSuggestionInput = ref("");
const isSaving = ref(false);
const draggedIndex = ref<number | null>(null);

watch(
  [() => suggestionsStore.config, currentLanguage],
  () => {
    if (isSaving.value) return;

    if (!suggestionsStore.config) {
      suggestionsList.value = [];
      suggestionsText.value = "";
      return;
    }
    const langSuggestions =
      suggestionsStore.config.byLanguage[currentLanguage.value];
    const suggestions = langSuggestions?.suggestions || [];
    suggestionsList.value = [...suggestions];
    suggestionsText.value = suggestions.join("\n");
  },
  { immediate: true },
);

/** Save suggestions to the store */
async function saveSuggestions() {
  if (!suggestionsStore.config) return;

  let suggestions: string[];
  if (editMode.value === "list") {
    suggestions = suggestionsList.value.filter((s) => s.trim().length > 0);
  } else {
    suggestions = suggestionsText.value
      .split("\n")
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
  }

  const newConfig = { ...suggestionsStore.config };
  newConfig.byLanguage[currentLanguage.value] = { suggestions };

  isSaving.value = true;
  await suggestionsStore.save(newConfig);
  isSaving.value = false;
}

/**
 * Add new suggestion from input
 */
function addSuggestion() {
  const text = newSuggestionInput.value.trim();
  if (!text) return;

  suggestionsList.value.push(text);
  newSuggestionInput.value = "";
  saveSuggestions();
}

/**
 * Remove suggestion at index
 * @param {number} index Index to remove
 */
function removeSuggestion(index: number) {
  suggestionsList.value.splice(index, 1);
  saveSuggestions();
}

/**
 * Update suggestion at index
 * @param {number} index Index to update
 * @param {string} text New text
 */
function updateSuggestion(index: number, text: string) {
  suggestionsList.value[index] = text;
}

/**
 * Switch editing mode
 * @param {"list" | "bulk"} mode Mode to switch to
 */
function switchMode(mode: "list" | "bulk") {
  if (mode === "bulk" && editMode.value === "list") {
    suggestionsText.value = suggestionsList.value.join("\n");
  } else if (mode === "list" && editMode.value === "bulk") {
    suggestionsList.value = suggestionsText.value
      .split("\n")
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
  }
  editMode.value = mode;
}

/**
 * Import suggestions from bulk textarea
 */
function importFromBulk() {
  suggestionsList.value = suggestionsText.value
    .split("\n")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
  editMode.value = "list";
  saveSuggestions();
}

/**
 * Handle drag start
 * @param {number} index The index of the dragged item
 */
function handleDragStart(index: number) {
  draggedIndex.value = index;
}

/**
 * Handle drag over
 * @param {DragEvent} event The drag event
 * @param {number} index The target index
 */
function handleDragOver(event: DragEvent, index: number) {
  event.preventDefault();
  if (draggedIndex.value === null || draggedIndex.value === index) return;

  const items = [...suggestionsList.value];
  const draggedItem = items[draggedIndex.value];
  items.splice(draggedIndex.value, 1);
  items.splice(index, 0, draggedItem);

  suggestionsList.value = items;
  draggedIndex.value = index;
}

/**
 * Handle drag end
 */
function handleDragEnd() {
  draggedIndex.value = null;
  saveSuggestions();
}
</script>

<template>
  <section class="space-y-6">
    <!-- Header Card -->
    <div
      class="rounded-2xl border border-success/30 bg-linear-to-br from-success/10 via-success/5 to-transparent p-6 shadow-sm backdrop-blur-sm">
      <div class="flex flex-col sm:flex-row items-start gap-5">
        <div
          class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-linear-to-br from-success to-success/80 shadow-lg">
          <SuggestionBulb class-name="h-7 w-7 text-white" />
        </div>
        <div class="flex-1 min-w-0">
          <h2 class="text-2xl font-bold text-base-content mb-2.5">
            {{ t("suggestions.title") }}
          </h2>
          <p class="text-sm text-base-content/70 leading-relaxed mb-4">
            {{ t("suggestions.description") }}
          </p>
          <div class="flex flex-wrap gap-2 items-center">
            <div class="badge badge-success badge-outline gap-1.5 py-3 px-3">
              <InfoIcon class-name="h-3.5 w-3.5" />
              <span class="text-xs font-medium">{{ t("suggestions.currentLanguage") }}: {{ currentLanguage }}</span>
            </div>
            <div v-if="suggestionsList.length" class="badge badge-ghost gap-1.5 py-3 px-3">
              <span class="font-semibold">{{ suggestionsList.length }}</span>
              <span class="text-xs">{{ t("suggestions.totalCount") }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="suggestionsStore.loading" class="flex justify-center py-12">
      <span class="loading loading-spinner loading-lg text-primary"></span>
    </div>

    <div v-else class="space-y-6">
      <!-- Mode Switcher -->
      <div class="flex gap-2 bg-base-200/50 p-1.5 rounded-xl w-fit">
        <button class="btn btn-sm transition-all font-medium"
          :class="{ 'btn-primary shadow-md': editMode === 'list', 'btn-ghost': editMode !== 'list' }"
          @click="switchMode('list')">
          <ListIcon class-name="h-4 w-4" />
          {{ t("suggestions.listMode") }}
        </button>
        <button class="btn btn-sm transition-all font-medium"
          :class="{ 'btn-primary shadow-md': editMode === 'bulk', 'btn-ghost': editMode !== 'bulk' }"
          @click="switchMode('bulk')">
          <DocumentIcon class-name="h-4 w-4" />
          {{ t("suggestions.bulkMode") }}
        </button>
      </div>

      <!-- List Mode -->
      <div v-if="editMode === 'list'" class="space-y-4">
        <!-- Add New Suggestion -->
        <div class="flex gap-2">
          <input v-model="newSuggestionInput" type="text" :placeholder="t('suggestions.addPlaceholder')"
            class="input input-bordered flex-1 transition-all focus:input-primary" @keyup.enter="addSuggestion" />
          <button class="btn btn-primary gap-2 font-medium shadow-md hover:shadow-lg transition-all"
            :disabled="!newSuggestionInput.trim()" @click="addSuggestion">
            <PlusIcon class-name="h-5 w-5" />
            {{ t("suggestions.add") }}
          </button>
        </div>

        <!-- Suggestions List -->
        <TransitionGroup name="list" tag="div" class="space-y-2 max-h-96 overflow-y-auto pr-2">
          <div v-for="(suggestion, index) in suggestionsList" :key="`suggestion-${index}`" :draggable="true"
            class="flex gap-2 items-center group bg-base-200/50 hover:bg-base-200 rounded-lg p-3 transition-all cursor-move"
            :class="{
              'opacity-50 scale-95': draggedIndex === index,
              'ring-2 ring-primary/50': draggedIndex !== null && draggedIndex !== index,
            }" @dragstart="handleDragStart(index)" @dragover="handleDragOver($event, index)" @dragend="handleDragEnd">
            <GripVerticalIcon
              class-name="h-4 w-4 text-base-content/20 group-hover:text-base-content/50 transition-colors cursor-grab active:cursor-grabbing" />
            <span class="text-base-content/40 font-mono text-xs w-8 text-right shrink-0">{{ index + 1 }}</span>
            <input :value="suggestion" type="text"
              class="input input-sm input-bordered flex-1 bg-base-100 transition-all focus:input-primary" @blur="
                updateSuggestion(index, ($event.target as HTMLInputElement).value);
              saveSuggestions();
              " @keyup.enter="($event.target as HTMLInputElement).blur()" />
            <button
              class="btn btn-sm btn-ghost btn-circle text-error opacity-0 group-hover:opacity-100 transition-opacity"
              :title="t('actions.delete')" @click="removeSuggestion(index)">
              <CloseIcon class-name="h-5 w-5" />
            </button>
          </div>

          <div v-if="!suggestionsList.length" key="empty" class="text-center py-12 text-base-content/50">
            <SuggestionBulb class-name="h-20 w-20 mx-auto mb-4 text-base-content/10" />
            <p class="text-base font-medium">{{ t("suggestions.emptyList") }}</p>
          </div>
        </TransitionGroup>
      </div>

      <!-- Bulk Mode -->
      <div v-if="editMode === 'bulk'" class="space-y-4">
        <!-- Instructions -->
        <div class="alert alert-warning shadow-lg">
          <InfoIcon class-name="h-6 w-6" />
          <div>
            <h3 class="font-bold text-sm">{{ t("suggestions.bulkModeTitle") }}</h3>
            <div class="text-xs opacity-80 mt-1">
              {{ t("suggestions.bulkModeDesc") }}
            </div>
          </div>
        </div>

        <!-- Textarea -->
        <div class="form-control">
          <textarea v-model="suggestionsText"
            class="textarea textarea-bordered h-80 font-mono text-sm leading-relaxed resize-none transition-all focus:textarea-primary"
            :placeholder="t('suggestions.bulkPlaceholder')" /><br />
          <label class="label">
            <span class="label-text-alt">
              <span class="font-semibold">{{
                suggestionsText.split("\n").filter((s) => s.trim()).length
                }}</span>
              {{ t("suggestions.linesDetected") }}
            </span>
          </label>
        </div>

        <!-- Action Buttons -->
        <div class="flex gap-2 justify-end">
          <button class="btn btn-ghost gap-2 font-medium" @click="switchMode('list')">
            <CloseIcon class-name="h-5 w-5" />
            {{ t("suggestions.cancel") }}
          </button>
          <button class="btn btn-primary gap-2 shadow-md hover:shadow-lg transition-all font-medium"
            @click="importFromBulk">
            <ImportIcon class-name="h-5 w-5" />
            {{ t("suggestions.importAndSave") }}
          </button>
        </div>
      </div>
    </div>
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
  transform: translateY(-10px);
}

.list-leave-to {
  opacity: 0;
  transform: translateY(10px);
}

.list-leave-active {
  position: absolute;
  width: calc(100% - 2rem);
}
</style>
