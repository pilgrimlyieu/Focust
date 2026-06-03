<script setup lang="ts">
import { computed, onMounted, ref, TransitionGroup, watch } from "vue";
import { useI18n } from "vue-i18n";
import CheckIcon from "@/components/icons/CheckIcon.vue";
import CloseIcon from "@/components/icons/CloseIcon.vue";
import DocumentIcon from "@/components/icons/DocumentIcon.vue";
import InfoIcon from "@/components/icons/InfoIcon.vue";
import ListIcon from "@/components/icons/ListIcon.vue";
import PlusIcon from "@/components/icons/PlusIcon.vue";
import RefreshIcon from "@/components/icons/RefreshIcon.vue";
import SuggestionBulb from "@/components/icons/SuggestionBulb.vue";
import { getI18nLocale } from "@/i18n";
import {
  createLanguageSuggestions,
  normalizeLanguageSuggestions,
  SUGGESTION_POOLS,
  type SuggestionPool,
  useSuggestionsStore,
} from "@/stores/suggestions";
import type { LanguageSuggestions, SuggestionsConfig } from "@/types";

const { t } = useI18n();
const suggestionsStore = useSuggestionsStore();

onMounted(() => {
  if (!suggestionsStore.hasLoaded) {
    suggestionsStore.load();
  }
});

const currentLanguage = computed(getI18nLocale);

const activePool = ref<SuggestionPool>("short");

/** UI mode: 'list' or 'bulk' */
const editMode = ref<"list" | "bulk">("list");

/** Suggestions as array (for list mode) */
const suggestionsList = ref<string[]>([]);

/** Suggestions as text (for bulk mode) */
const suggestionsText = ref("");

const newSuggestionInput = ref("");
const isPersisting = ref(false);
const saveError = ref(false);
const savedConfig = ref<SuggestionsConfig | null>(null);
const draftConfig = ref<SuggestionsConfig | null>(null);

const poolTranslationKeys = {
  long: "suggestions.longBreakSuggestions",
  short: "suggestions.shortBreakSuggestions",
} as const satisfies Record<SuggestionPool, string>;

const poolActiveClasses = {
  long: "border-secondary/50 bg-secondary/10 shadow-sm",
  short: "border-primary/50 bg-primary/10 shadow-sm",
} as const satisfies Record<SuggestionPool, string>;

const poolBadgeClasses = {
  long: "badge-secondary",
  short: "badge-primary",
} as const satisfies Record<SuggestionPool, string>;

const isInitialLoading = computed(
  () => suggestionsStore.loading && !draftConfig.value,
);

const poolCounts = computed(() => {
  if (!draftConfig.value) {
    return { long: 0, short: 0 } satisfies Record<SuggestionPool, number>;
  }

  const langSuggestions = normalizeLanguageSuggestions(
    draftConfig.value.byLanguage[currentLanguage.value],
  );

  return {
    long: langSuggestions.longSuggestions.length,
    short: langSuggestions.shortSuggestions.length,
  } satisfies Record<SuggestionPool, number>;
});

const activePoolCount = computed(() => poolCounts.value[activePool.value]);

const bulkLineCount = computed(
  () => normalizeTextSuggestions(suggestionsText.value).length,
);

const hasDraftChanges = computed(
  () => serializeConfig(draftConfig.value) !== serializeConfig(savedConfig.value),
);

function cloneLanguageSuggestions(
  languageConfig: Partial<LanguageSuggestions> | null | undefined,
): LanguageSuggestions {
  const normalized = normalizeLanguageSuggestions(languageConfig);
  return createLanguageSuggestions(
    normalized.shortSuggestions,
    normalized.longSuggestions,
  );
}

function cloneSuggestionsConfig(
  config: SuggestionsConfig | null | undefined,
): SuggestionsConfig | null {
  if (!config) return null;

  return {
    byLanguage: Object.fromEntries(
      Object.entries(config.byLanguage).map(([language, languageConfig]) => [
        language,
        cloneLanguageSuggestions(languageConfig),
      ]),
    ),
  };
}

function serializeConfig(config: SuggestionsConfig | null): string {
  return JSON.stringify(config);
}

function normalizeListSuggestions(values: string[]): string[] {
  return values.map((value) => value.trim()).filter(Boolean);
}

function normalizeTextSuggestions(value: string): string[] {
  return normalizeListSuggestions(value.split("\n"));
}

function ensureDraftConfig() {
  if (draftConfig.value) return;

  draftConfig.value = cloneSuggestionsConfig(suggestionsStore.config) ?? {
    byLanguage: {},
  };
  savedConfig.value ??= cloneSuggestionsConfig(suggestionsStore.config) ?? {
    byLanguage: {},
  };
}

function getDraftLanguageSuggestions(): LanguageSuggestions {
  return cloneLanguageSuggestions(
    draftConfig.value?.byLanguage[currentLanguage.value],
  );
}

function getPoolSuggestions(
  languageSuggestions: LanguageSuggestions,
  pool: SuggestionPool,
): string[] {
  return pool === "short"
    ? languageSuggestions.shortSuggestions
    : languageSuggestions.longSuggestions;
}

function getActivePoolSuggestions(): string[] {
  return getPoolSuggestions(getDraftLanguageSuggestions(), activePool.value);
}

function setDraftLanguageSuggestions(languageSuggestions: LanguageSuggestions) {
  ensureDraftConfig();
  if (!draftConfig.value) return;

  draftConfig.value = {
    byLanguage: {
      ...draftConfig.value.byLanguage,
      [currentLanguage.value]: cloneLanguageSuggestions(languageSuggestions),
    },
  };
  saveError.value = false;
}

function setActivePoolSuggestions(suggestions: string[]) {
  const languageSuggestions = getDraftLanguageSuggestions();
  const shortSuggestions =
    activePool.value === "short"
      ? suggestions
      : languageSuggestions.shortSuggestions;
  const longSuggestions =
    activePool.value === "long"
      ? suggestions
      : languageSuggestions.longSuggestions;

  setDraftLanguageSuggestions(
    createLanguageSuggestions(shortSuggestions, longSuggestions),
  );
}

function refreshEditorFromDraft() {
  const suggestions = getActivePoolSuggestions();
  suggestionsList.value = [...suggestions];
  suggestionsText.value = suggestions.join("\n");
  newSuggestionInput.value = "";
}

function commitActiveEditorToDraft() {
  const suggestions =
    editMode.value === "list"
      ? normalizeListSuggestions(suggestionsList.value)
      : normalizeTextSuggestions(suggestionsText.value);

  setActivePoolSuggestions(suggestions);
  return suggestions;
}

watch(
  () => suggestionsStore.config,
  (config) => {
    if (!config) {
      if (!hasDraftChanges.value) {
        savedConfig.value = null;
        draftConfig.value = null;
        refreshEditorFromDraft();
      }
      return;
    }

    if (isPersisting.value || hasDraftChanges.value) return;

    savedConfig.value = cloneSuggestionsConfig(config);
    draftConfig.value = cloneSuggestionsConfig(config);
    refreshEditorFromDraft();
  },
  { immediate: true },
);

watch(currentLanguage, () => {
  refreshEditorFromDraft();
});

async function saveDraft() {
  if (isPersisting.value || !draftConfig.value) return;

  commitActiveEditorToDraft();

  const configToSave = cloneSuggestionsConfig(draftConfig.value);
  if (!configToSave) return;

  isPersisting.value = true;
  saveError.value = false;
  try {
    await suggestionsStore.save(configToSave);
    const persistedConfig =
      cloneSuggestionsConfig(suggestionsStore.config) ?? configToSave;
    savedConfig.value = cloneSuggestionsConfig(persistedConfig);
    draftConfig.value = cloneSuggestionsConfig(persistedConfig);
    refreshEditorFromDraft();
  } catch (err) {
    console.error("Failed to save suggestions:", err);
    saveError.value = true;
  } finally {
    isPersisting.value = false;
  }
}

function resetDraft() {
  if (!savedConfig.value || isPersisting.value) return;

  draftConfig.value = cloneSuggestionsConfig(savedConfig.value);
  saveError.value = false;
  refreshEditorFromDraft();
}

function selectPool(pool: SuggestionPool) {
  if (pool === activePool.value) return;

  commitActiveEditorToDraft();
  activePool.value = pool;
  refreshEditorFromDraft();
}

function switchMode(mode: "list" | "bulk") {
  if (mode === editMode.value) return;

  commitActiveEditorToDraft();
  editMode.value = mode;
  refreshEditorFromDraft();
}

function commitListEditor() {
  if (editMode.value !== "list") return;
  setActivePoolSuggestions(normalizeListSuggestions(suggestionsList.value));
}

function normalizeListEditor() {
  if (editMode.value !== "list") return;

  const suggestions = normalizeListSuggestions(suggestionsList.value);
  suggestionsList.value = [...suggestions];
  suggestionsText.value = suggestions.join("\n");
  setActivePoolSuggestions(suggestions);
}

function commitTextEditor() {
  if (editMode.value !== "bulk") return;
  setActivePoolSuggestions(normalizeTextSuggestions(suggestionsText.value));
}

/**
 * Add new suggestion from input
 */
function addSuggestion() {
  const text = newSuggestionInput.value.trim();
  if (!text) return;

  suggestionsList.value.push(text);
  newSuggestionInput.value = "";
  commitListEditor();
}

/**
 * Remove suggestion at index
 * @param {number} index Index to remove
 */
function removeSuggestion(index: number) {
  suggestionsList.value.splice(index, 1);
  commitListEditor();
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
              <span class="text-xs font-medium">{{ t("suggestions.currentLanguage", { language: currentLanguage })
                }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isInitialLoading" class="flex justify-center py-12">
      <span class="loading loading-spinner loading-lg text-primary"></span>
    </div>

    <div v-else class="rounded-2xl border border-base-300 bg-base-100/70 p-5 shadow-md space-y-5">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div class="min-w-0">
          <h3 class="text-lg font-bold">{{ t("suggestions.customTitle") }}</h3>
          <p class="mt-1 text-sm text-base-content/60">
            {{ t("suggestions.description") }}
          </p>
        </div>
        <div class="flex flex-wrap items-center gap-2">
          <button class="btn btn-sm btn-ghost gap-2" :disabled="!hasDraftChanges || isPersisting" @click="resetDraft">
            <RefreshIcon class-name="h-4 w-4" />
            {{ t("actions.reset") }}
          </button>
          <button class="btn btn-sm btn-primary gap-2" :disabled="!hasDraftChanges || isPersisting" @click="saveDraft">
            <span v-if="isPersisting" class="loading loading-spinner loading-xs" />
            <CheckIcon v-else class-name="h-4 w-4" />
            {{ t("actions.save") }}
          </button>
        </div>
      </div>

      <div v-if="saveError" class="alert alert-error py-3 text-sm">
        <InfoIcon class-name="h-5 w-5" />
        <span>{{ t("toast.saveFailed") }}</span>
      </div>

      <!-- Suggestion Pool Switcher -->
      <div class="grid gap-3 md:grid-cols-2">
        <button v-for="pool in SUGGESTION_POOLS" :key="pool" type="button"
          class="rounded-xl border p-4 text-left transition-all"
          :class="activePool === pool
            ? poolActiveClasses[pool]
            : 'border-base-300 bg-base-100 hover:border-base-content/20 hover:bg-base-200/40'"
          :aria-pressed="activePool === pool" @click="selectPool(pool)">
          <div class="flex items-start justify-between gap-4">
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-2">
                <span class="badge badge-sm" :class="poolBadgeClasses[pool]">
                  {{ pool === "short" ? "MINI" : "LONG" }}
                </span>
                <span class="text-sm font-semibold">{{ t(poolTranslationKeys[pool]) }}</span>
              </div>
            </div>
            <span class="tabular-nums text-2xl font-semibold leading-none">
              {{ poolCounts[pool] }}
            </span>
          </div>
        </button>
      </div>

      <!-- Mode Switcher -->
      <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div class="tabs tabs-boxed bg-base-200/70 p-1">
          <button type="button" class="tab tab-sm h-9 gap-2 px-3" :class="{ 'tab-active': editMode === 'list' }"
            @click="switchMode('list')">
            <ListIcon class-name="h-4 w-4" />
            {{ t("suggestions.listMode") }}
          </button>
          <button type="button" class="tab tab-sm h-9 gap-2 px-3" :class="{ 'tab-active': editMode === 'bulk' }"
            @click="switchMode('bulk')">
            <DocumentIcon class-name="h-4 w-4" />
            {{ t("suggestions.bulkMode") }}
          </button>
        </div>
        <div class="text-xs text-base-content/50">
          <span class="font-semibold text-base-content/70">{{ activePoolCount }}</span>
          {{ t("suggestions.totalCount") }}
        </div>
      </div>

      <!-- List Mode -->
      <div v-if="editMode === 'list'" class="space-y-4">
        <!-- Add New Suggestion -->
        <div class="flex flex-col gap-2 sm:flex-row">
          <input v-model="newSuggestionInput" type="text" :placeholder="t('suggestions.addPlaceholder')"
            class="input input-bordered w-full min-w-0 transition-all focus:input-primary sm:flex-1"
            @keyup.enter="addSuggestion" />
          <button class="btn btn-primary w-full gap-2 font-medium shadow-md hover:shadow-lg transition-all sm:w-auto"
            :disabled="!newSuggestionInput.trim()" @click="addSuggestion">
            <PlusIcon class-name="h-5 w-5" />
            {{ t("suggestions.add") }}
          </button>
        </div>

        <!-- Suggestions List -->
        <TransitionGroup name="list" tag="div" class="space-y-2 max-h-[30rem] overflow-y-auto pr-1">
          <div v-for="(suggestion, index) in suggestionsList" :key="`suggestion-${index}-${suggestion}`"
            class="flex gap-2 items-center group bg-base-200/50 hover:bg-base-200 rounded-lg p-3 transition-all">
            <span class="text-base-content/40 font-mono text-xs w-8 text-right shrink-0">{{ index + 1 }}</span>
            <input v-model="suggestionsList[index]" type="text"
              class="input input-sm input-bordered min-w-0 flex-1 bg-base-100 transition-all focus:input-primary"
              @input="commitListEditor" @blur="normalizeListEditor"
              @keyup.enter="($event.target as HTMLInputElement).blur()" />
            <button
              class="btn btn-sm btn-ghost btn-circle text-error opacity-100 transition-opacity sm:opacity-0 sm:group-hover:opacity-100"
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
        <div class="alert border-base-300 bg-base-200/60 py-3 shadow-none">
          <InfoIcon class-name="h-5 w-5" />
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
            class="textarea textarea-bordered h-[28rem] font-mono text-sm leading-relaxed resize-none transition-all focus:textarea-primary"
            @input="commitTextEditor"
            :placeholder="t('suggestions.bulkPlaceholder')" /><br />
          <label class="label">
            <span class="label-text-alt">
              <span class="font-semibold">{{ bulkLineCount }}</span>
              {{ t("suggestions.linesDetected") }}
            </span>
          </label>
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
