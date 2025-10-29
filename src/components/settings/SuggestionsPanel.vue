<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { getI18nLocale } from "@/i18n";
import { useSuggestionsStore } from "@/stores/suggestions";

const { t } = useI18n();
const suggestionsStore = useSuggestionsStore();

// Load suggestions on mount
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
</script>

<template>
  <section class="space-y-6">
    <header>
      <h2 class="text-xl font-semibold">{{ t("suggestions.title") }}</h2>
      <p class="text-sm opacity-70">
        {{ t("suggestions.description") }}
      </p>
    </header>

    <div v-if="suggestionsStore.loading" class="flex justify-center py-8">
      <span class="loading loading-spinner loading-lg"></span>
    </div>

    <div v-else class="space-y-6">
      <!-- Language info -->
      <div class="alert alert-info shadow-lg">
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-6 h-6">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
        </svg>
        <div class="flex-1">
          <div class="font-semibold">
            {{ t("suggestions.currentLanguage") }}: {{ currentLanguage }}
          </div>
          <div class="text-sm mt-1 opacity-80">
            {{ t("suggestions.totalCount") }}: {{ suggestionsList.length }}
          </div>
        </div>
      </div>

      <!-- Mode switcher -->
      <div class="flex gap-2">
        <button class="btn btn-sm" :class="{ 'btn-primary': editMode === 'list' }" @click="switchMode('list')">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16" />
          </svg>
          {{ t("suggestions.listMode") }}
        </button>
        <button class="btn btn-sm" :class="{ 'btn-primary': editMode === 'bulk' }" @click="switchMode('bulk')">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          {{ t("suggestions.bulkMode") }}
        </button>
      </div>

      <!-- List mode -->
      <div v-if="editMode === 'list'" class="space-y-4">
        <!-- Add new suggestion -->
        <div class="flex gap-2">
          <input v-model="newSuggestionInput" type="text" :placeholder="t('suggestions.addPlaceholder')"
            class="input input-bordered flex-1" @keyup.enter="addSuggestion" />
          <button class="btn btn-primary" :disabled="!newSuggestionInput.trim()" @click="addSuggestion">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24"
              stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            {{ t("suggestions.add") }}
          </button>
        </div>

        <!-- Suggestions list -->
        <div class="space-y-2 max-h-96 overflow-y-auto pr-2">
          <div v-for="(suggestion, index) in suggestionsList" :key="index"
            class="flex gap-2 items-center group bg-base-200/50 hover:bg-base-200 rounded-lg p-3 transition-all">
            <span class="text-base-content/40 font-mono text-sm w-8 text-right">{{ index + 1 }}</span>
            <input :value="suggestion" type="text" class="input input-sm input-bordered flex-1 bg-base-100" @blur="
              updateSuggestion(
                index,
                ($event.target as HTMLInputElement).value
              );
            saveSuggestions();
            " @keyup.enter="($event.target as HTMLInputElement).blur()" />
            <button
              class="btn btn-sm btn-ghost btn-circle text-error opacity-0 group-hover:opacity-100 transition-opacity"
              @click="removeSuggestion(index)">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div v-if="!suggestionsList.length" class="text-center py-8 text-base-content/50">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 mx-auto mb-2 opacity-20" fill="none"
              viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <p>{{ t("suggestions.emptyList") }}</p>
          </div>
        </div>
      </div>

      <!-- Bulk mode -->
      <div v-if="editMode === 'bulk'" class="space-y-4">
        <!-- Instructions card -->
        <div class="alert alert-warning shadow-lg">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"
            class="stroke-current shrink-0 w-6 h-6">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
          </svg>
          <div>
            <h3 class="font-bold">{{ t("suggestions.bulkModeTitle") }}</h3>
            <div class="text-xs opacity-80">
              {{ t("suggestions.bulkModeDesc") }}
            </div>
          </div>
        </div>

        <!-- Textarea -->
        <div class="form-control">
          <textarea v-model="suggestionsText"
            class="textarea textarea-bordered h-80 font-mono text-sm leading-relaxed resize-none"
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

        <!-- Action buttons -->
        <div class="flex gap-2 justify-end">
          <button class="btn btn-ghost" @click="switchMode('list')">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24"
              stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
            {{ t("suggestions.cancel") }}
          </button>
          <button class="btn btn-primary" @click="importFromBulk">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24"
              stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            {{ t("suggestions.importAndSave") }}
          </button>
        </div>
      </div>
    </div>
  </section>
</template>
