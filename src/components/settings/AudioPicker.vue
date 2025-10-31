<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import type { AudioSettings } from "@/types";
import {
  convertToBuiltinAudio,
  convertToFilePathAudio,
  convertToNoAudio,
  getAudioFilePath,
  getAudioSourceType,
  getBuiltinAudioName,
  isBuiltinAudio,
  isFilePathAudio,
  isNoAudio,
} from "@/types";

const props = defineProps<{
  audio: AudioSettings;
  label: string;
}>();

const { t } = useI18n();

// Built-in audio options
const providedOptions = [
  { label: "Gentle Bell", value: "gentle-bell" },
  { label: "Soft Gong", value: "soft-gong" },
  { label: "Notification", value: "notification" },
  { label: "Bright Notification", value: "bright-notification" },
];

const selectedType = computed<"none" | "builtin" | "filepath">({
  get: () => {
    const type = getAudioSourceType(props.audio);
    // Convert "filePath" to "filepath" for UI consistency
    return type === "filePath" ? "filepath" : type;
  },
  set: (value) => {
    if (value === "none") {
      convertToNoAudio(props.audio);
    } else if (value === "builtin") {
      convertToBuiltinAudio(
        props.audio,
        providedOptions[0]?.value ?? "gentle-bell",
      );
    } else {
      convertToFilePathAudio(props.audio, "");
    }
  },
});

const builtinValue = computed({
  get: () => {
    return getBuiltinAudioName(props.audio) ?? "";
  },
  set: (value: string) => {
    convertToBuiltinAudio(props.audio, value);
  },
});

const filePath = computed({
  get: () => {
    return getAudioFilePath(props.audio) ?? "";
  },
  set: (value: string) => {
    convertToFilePathAudio(props.audio, value);
  },
});

const isPreviewing = ref(false);

/**
 * Stop any ongoing audio preview.
 */
async function stopPreview() {
  try {
    await invoke("stop_audio");
  } catch (error) {
    console.error("Failed to stop audio:", error);
  }
  isPreviewing.value = false;
}

/**
 * Play a preview of the selected audio.
 */
async function playPreview() {
  await stopPreview();
  const audio = props.audio;

  if (isNoAudio(audio)) {
    return;
  }

  try {
    isPreviewing.value = true;

    if (isFilePathAudio(audio)) {
      const path = audio.path;
      await invoke("play_audio", { path, volume: audio.volume });
    } else if (isBuiltinAudio(audio)) {
      const name = audio.name;
      await invoke("play_builtin_audio", {
        resourceName: name,
        volume: audio.volume,
      });
    }

    // Auto-stop preview flag after 3 seconds (audio might finish before)
    setTimeout(() => {
      isPreviewing.value = false;
    }, 3000);
  } catch (error) {
    console.error("Failed to play audio:", error);
    isPreviewing.value = false;
  }
}

/**
 * Open a file dialog to pick a custom audio file.
 */
async function pickAudioFile() {
  const file = await open({
    filters: [{ extensions: ["mp3", "wav", "ogg", "flac"], name: "Audio" }],
    multiple: false,
  });
  if (typeof file === "string") {
    filePath.value = file;
  }
}

defineExpose({
  builtinValue,
  filePath,
  isPreviewing,
  pickAudioFile,
  playPreview,
  providedOptions,
  selectedType,
  stopPreview,
  t,
});
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium">{{ label }}</h3>
      <span class="badge badge-ghost badge-sm">{{ t("audio.label") }}</span>
    </div>

    <!-- Audio Source Selection -->
    <label class="form-control">
      <span class="label-text font-medium mb-2">{{ t("audio.source") }}</span>
      <select v-model="selectedType" class="select select-bordered">
        <option value="none">{{ t("audio.none") }}</option>
        <option value="builtin">{{ t("audio.builtin") }}</option>
        <option value="filepath">{{ t("audio.customFile") }}</option>
      </select>
    </label>

    <!-- Builtin Audio Selection -->
    <div v-if="selectedType === 'builtin'" class="space-y-3 rounded-lg bg-base-200/50 p-4">
      <div class="form-control">
        <span class="label-text font-medium mb-2">{{ t("audio.preset") }}</span>
        <select v-model="builtinValue" class="select select-bordered">
          <option v-for="option in providedOptions" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>
      </div>
      <p class="text-xs opacity-70">
        {{ t("audio.chooseBuiltin") }}
      </p>
    </div>

    <!-- Custom File Selection -->
    <div v-else-if="selectedType === 'filepath'" class="space-y-3 rounded-lg bg-base-200/50 p-4">
      <div class="form-control">
        <span class="label-text font-medium mb-2">{{ t("audio.filePath") }}</span>
        <div class="flex gap-2">
          <input v-model="filePath" type="text" class="input input-bordered flex-1" readonly
            :placeholder="t('audio.noFileSelected')" />
          <button class="btn btn-outline" @click="pickAudioFile">
            📁 {{ t("audio.browse") }}
          </button>
        </div>
      </div>
      <p class="text-xs opacity-70">
        {{ t("audio.supportedFormats") }}
      </p>
    </div>

    <!-- Volume Control -->
    <div class="form-control space-y-2">
      <div class="flex items-center justify-between">
        <span class="label-text font-medium">{{ t("audio.volume") }}</span>
        <span class="text-sm font-mono opacity-70">{{ (audio.volume * 100).toFixed(0) }}%</span>
      </div>
      <input v-model.number="audio.volume" class="range range-sm range-primary" type="range" min="0" max="1"
        step="0.05" />
      <div class="flex justify-between text-xs opacity-50">
        <span>0%</span>
        <span>50%</span>
        <span>100%</span>
      </div>
    </div>

    <!-- Preview Controls -->
    <div v-if="selectedType !== 'none'" class="flex gap-2 pt-2">
      <button class="btn btn-sm btn-primary flex-1" :disabled="isPreviewing" @click="playPreview">
        <span v-if="!isPreviewing">▶️ {{ t("audio.preview") }}</span>
        <span v-else>
          <span class="loading loading-spinner loading-xs mr-2"></span>
          {{ t("audio.playing") }}
        </span>
      </button>
      <button class="btn btn-sm btn-ghost" :disabled="!isPreviewing" @click="stopPreview">
        ⏹️ {{ t("audio.stop") }}
      </button>
    </div>
  </div>
</template>
