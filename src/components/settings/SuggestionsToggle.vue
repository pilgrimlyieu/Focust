<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { getI18nLocale } from "@/i18n";
import { useSuggestionsStore } from "@/stores/suggestions";
import type { SuggestionsSettings } from "@/types/generated/SuggestionsSettings";

const props = defineProps<{
  suggestions: SuggestionsSettings;
  label: string;
}>();

const { t } = useI18n();
const suggestionsStore = useSuggestionsStore();

onMounted(() => {
  if (!suggestionsStore.hasLoaded) {
    suggestionsStore.load();
  }
});

const preview = computed(() => {
  if (!props.suggestions.show) {
    return [];
  }
  return suggestionsStore.sampleMany(getI18nLocale(), 3);
});

// Computed property for toggle
const showSuggestions = computed({
  get: () => props.suggestions.show,
  set: (value: boolean) => {
    props.suggestions.show = value;
  },
});

defineExpose({ preview, showSuggestions, t });
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium">{{ label }}</h3>
      <span class="badge badge-ghost badge-sm">{{ t("suggestions.label") }}</span>
    </div>

    <!-- Simple toggle: Show suggestions or not -->
    <label class="form-control">
      <div class="flex items-center justify-between">
        <span class="label-text">{{ t("suggestions.toggle") }}</span>
        <input v-model="showSuggestions" type="checkbox" class="toggle toggle-primary" />
      </div>
      <span class="label-text-alt opacity-70 mt-1">
        {{ showSuggestions ? t("suggestions.enabled") : t("suggestions.disabled") }}
      </span>
    </label>

    <!-- Preview box -->
    <div class="rounded-lg border border-base-300 bg-base-100/70 p-4 text-sm min-h-24">
      <template v-if="showSuggestions && preview.length">
        <p v-for="suggestion in preview" :key="suggestion" class="opacity-80 leading-relaxed">
          • {{ suggestion }}
        </p>
      </template>
      <p v-else class="opacity-50 text-center py-4">{{ t("suggestions.noPreview") }}</p>
    </div>
  </div>
</template>
