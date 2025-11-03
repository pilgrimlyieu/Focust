<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import type { BackgroundSource, ThemeSettings } from "@/types";
import {
  createImageFolderBackground,
  createImagePathBackground,
  createSolidBackground,
  getImageFolder,
  getImagePath,
  getSolidColor,
  isImageFolderBackground,
  isImagePathBackground,
  isSolidBackground,
} from "@/types";

const props = defineProps<{
  theme: ThemeSettings;
  label: string;
}>();

const { t } = useI18n();

const backgroundType = ref<"solid" | "image" | "folder">("solid");
const imagePreview = ref<string | null>(null);
const lastSolidColor = ref<string>("#1f2937");
const lastImagePath = ref<string>("");
const lastFolder = ref<string>("");

// Use factory functions for background access
const solidColor = computed({
  get: () => {
    const bg = props.theme.background;
    return isSolidBackground(bg)
      ? getSolidColor(bg) || lastSolidColor.value
      : lastSolidColor.value;
  },
  set: (value: string) => updateBackground(createSolidBackground(value)),
});

const imagePath = computed({
  get: () => {
    const bg = props.theme.background;
    return isImagePathBackground(bg)
      ? getImagePath(bg) || lastImagePath.value
      : lastImagePath.value;
  },
  set: (value: string) => updateBackground(createImagePathBackground(value)),
});

const folderPath = computed({
  get: () => {
    const bg = props.theme.background;
    return isImageFolderBackground(bg)
      ? getImageFolder(bg) || lastFolder.value
      : lastFolder.value;
  },
  set: (value: string) => updateBackground(createImageFolderBackground(value)),
});

/**
 * Update the background source in the theme.
 * @param {BackgroundSource} source The new background source.
 */
function updateBackground(source: BackgroundSource) {
  props.theme.background = source;
  if (isImagePathBackground(source)) {
    const path = getImagePath(source);
    if (path) {
      imagePreview.value = convertFileSrc(path);
      lastImagePath.value = path;
    }
  } else if (isImageFolderBackground(source)) {
    imagePreview.value = null;
    const folder = getImageFolder(source);
    if (folder) {
      lastFolder.value = folder;
    }
  } else if (isSolidBackground(source)) {
    imagePreview.value = null;
    const color = getSolidColor(source);
    if (color) {
      lastSolidColor.value = color;
    }
  }
}

watch(
  () => props.theme.background,
  (background) => {
    if (isSolidBackground(background)) {
      backgroundType.value = "solid";
      imagePreview.value = null;
      const color = getSolidColor(background);
      if (color) lastSolidColor.value = color;
    } else if (isImagePathBackground(background)) {
      backgroundType.value = "image";
      const path = getImagePath(background);
      if (path) {
        imagePreview.value = convertFileSrc(path);
        lastImagePath.value = path;
      }
    } else if (isImageFolderBackground(background)) {
      backgroundType.value = "folder";
      imagePreview.value = null;
      const folder = getImageFolder(background);
      if (folder) lastFolder.value = folder;
    }
  },
  { immediate: true },
);

watch(backgroundType, (mode) => {
  if (mode === "solid") {
    updateBackground({ solid: lastSolidColor.value });
  } else if (mode === "image") {
    updateBackground({ imagePath: lastImagePath.value });
  } else {
    updateBackground({ imageFolder: lastFolder.value });
  }
});

/**
 * Open a file picker to select an image file.
 */
async function pickImage() {
  const file = await open({
    filters: [
      {
        extensions: ["png", "jpg", "jpeg", "webp", "gif", "bmp"],
        name: "Images",
      },
    ],
    multiple: false,
  });
  if (typeof file === "string") {
    updateBackground({ imagePath: file });
  }
}

/**
 * Open a folder picker to select an image folder.
 */
async function pickFolder() {
  const folder = await open({ directory: true, multiple: false });
  if (typeof folder === "string") {
    updateBackground({ imageFolder: folder });
  }
}

const cardStyle = computed(() => {
  const background = props.theme.background;
  if (isSolidBackground(background)) {
    return {
      background: background.solid,
    };
  }
  if (imagePreview.value) {
    return {
      backgroundImage: `url("${imagePreview.value}")`,
      backgroundPosition: "center",
      backgroundSize: "cover",
    };
  }
  return {
    background: "#1f2937",
  };
});

const overlayStyle = computed(() => ({
  backdropFilter: `blur(${props.theme.blurRadius}px)`,
  backgroundColor: `rgba(15, 23, 42, ${1 - props.theme.opacity})`,
  color: props.theme.textColor,
  fontFamily: props.theme.fontFamily,
  fontSize: `${props.theme.fontSize}px`,
}));

defineExpose({
  backgroundType,
  cardStyle,
  folderPath,
  imagePath,
  imagePreview,
  overlayStyle,
  pickFolder,
  pickImage,
  solidColor,
  t,
});
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium">{{ label }}</h3>
      <span class="badge badge-ghost badge-sm">{{ t("theme.title") }}</span>
    </div>

    <!-- Background Configuration -->
    <div class="space-y-4 rounded-lg bg-base-200/50 p-5">
      <h4 class="font-medium text-base">{{ t("theme.background") }}</h4>

      <label class="form-control w-full">
        <span class="label-text font-medium mb-2">{{ t("theme.backgroundType") }}</span>
        <select v-model="backgroundType" class="select select-bordered w-full">
          <option value="solid">{{ t("theme.solidColor") }}</option>
          <option value="image">{{ t("theme.singleImage") }}</option>
          <option value="folder">{{ t("theme.imageFolder") }}</option>
        </select>
      </label>

      <div v-if="backgroundType === 'solid'" class="form-control w-full pt-2">
        <span class="label-text font-medium mb-2">{{ t("theme.solidColor") }}</span>
        <input v-model="solidColor" type="color" class="input input-bordered h-12 cursor-pointer w-full" />
      </div>

      <div v-else-if="backgroundType === 'image'" class="space-y-2 pt-2">
        <span class="label-text font-medium">{{ t("theme.imagePath") }}</span>
        <div class="flex gap-2">
          <input v-model="imagePath" type="text" class="input input-bordered flex-1" readonly />
          <button class="btn btn-outline shrink-0" @click="pickImage">📁 {{ t("audio.browse") }}</button>
        </div>
      </div>

      <div v-else class="space-y-2 pt-2">
        <span class="label-text font-medium">{{ t("theme.folderPath") }}</span>
        <div class="flex gap-2">
          <input v-model="folderPath" type="text" class="input input-bordered flex-1" readonly />
          <button class="btn btn-outline shrink-0" @click="pickFolder">📁 {{ t("audio.browse") }}</button>
        </div>
        <p class="text-xs opacity-70 pt-1">{{ t("theme.randomImageHint") }}</p>
      </div>
    </div>

    <!-- Text & Overlay Configuration -->
    <div class="space-y-5 rounded-lg bg-base-200/50 p-5">
      <h4 class="font-medium text-base">{{ t("theme.textAndOverlay") }}</h4>

      <div class="grid gap-4 md:grid-cols-2">
        <label class="form-control w-full">
          <span class="label-text font-medium mb-2">{{ t("theme.textColor") }}</span>
          <input v-model="theme.textColor" type="color" class="input input-bordered h-12 cursor-pointer w-full" />
        </label>

        <label class="form-control w-full">
          <span class="label-text font-medium mb-2">{{ t("theme.fontSize") }}</span>
          <input v-model.number="theme.fontSize" type="number" min="12" max="96" class="input input-bordered w-full" />
        </label>
      </div>

      <label class="form-control w-full">
        <span class="label-text font-medium mb-2">{{ t("theme.fontFamily") }}</span>
        <input v-model="theme.fontFamily" type="text" class="input input-bordered w-full"
          :placeholder="t('theme.fontPlaceholder')" />
      </label>

      <div class="grid gap-6 md:grid-cols-2 pt-2">
        <label class="form-control w-full">
          <div class="flex items-center justify-between mb-2">
            <span class="label-text font-medium">{{ t("theme.opacity") }}</span>
            <span class="text-sm font-mono opacity-70">{{ (theme.opacity * 100).toFixed(0) }}%</span>
          </div>
          <input v-model.number="theme.opacity" type="range" min="0.3" max="1" step="0.01"
            class="range range-sm range-primary" />
          <div class="flex justify-between text-xs opacity-50 mt-1">
            <span>30%</span>
            <span>100%</span>
          </div>
          <p class="text-xs opacity-70 mt-2">{{ t("theme.opacityHint") }}</p>
        </label>

        <label class="form-control w-full">
          <div class="flex items-center justify-between mb-2">
            <span class="label-text font-medium">{{ t("theme.blur") }}</span>
            <span class="text-sm font-mono opacity-70">{{ theme.blurRadius }}px</span>
          </div>
          <input v-model.number="theme.blurRadius" type="range" min="0" max="30" step="1"
            class="range range-sm range-primary" />
          <div class="flex justify-between text-xs opacity-50 mt-1">
            <span>0px</span>
            <span>30px</span>
          </div>
          <p class="text-xs opacity-70 mt-2">{{ t("theme.blurHint") }}</p>
        </label>
      </div>
    </div>

    <!-- Preview -->
    <div class="space-y-3">
      <h4 class="font-medium text-base">{{ t("theme.preview") }}</h4>
      <div class="rounded-xl border border-base-300 bg-base-100/70 p-6">
        <div class="rounded-lg p-6 shadow-inner" :style="cardStyle">
          <div class="rounded-lg bg-base-100/30 p-6" :style="overlayStyle">
            <p class="text-xl font-semibold">{{ t("break.attention") }}</p>
            <p class="opacity-80">{{ t("general.postponeHint") }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
