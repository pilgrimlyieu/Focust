<script setup lang="ts">
import { computed, ref } from "vue";

interface Props {
  modelValue: string;
  label?: string;
  placeholder?: string;
}

const props = withDefaults(defineProps<Props>(), {
  label: "",
  placeholder: "",
});

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const isCapturing = ref(false);

const displayValue = computed(() => {
  return props.modelValue || "";
});

/**
 * Start capturing keyboard input.
 */
function startCapture() {
  isCapturing.value = true;
}

/**
 * Stop capturing keyboard input.
 */
function stopCapture() {
  isCapturing.value = false;
}

/**
 * Handle keydown events to capture the shortcut.
 * @param {KeyboardEvent} event The keyboard event.
 */
function handleKeyDown(event: KeyboardEvent) {
  if (!isCapturing.value) return;

  // Ignore modifier keys when pressed alone
  if (["Control", "Alt", "Shift", "Meta"].includes(event.key)) {
    return;
  }

  // Build shortcut string
  const parts: string[] = [];

  if (event.ctrlKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  if (event.metaKey) parts.push("Meta");

  // Get the key name
  let keyName = event.key;

  // Normalize special keys
  if (keyName === " ") {
    keyName = "Space";
  } else if (keyName.length === 1) {
    // For single character keys, use uppercase
    keyName = keyName.toUpperCase();
  } else {
    // For other keys (F1, Escape, etc.), capitalize first letter
    keyName = keyName.charAt(0).toUpperCase() + keyName.slice(1);
  }

  parts.push(keyName);

  const shortcut = parts.join("+");
  emit("update:modelValue", shortcut);

  // Blur the input to stop capturing
  inputRef.value?.blur();
}

/**
 * Clear the current shortcut.
 */
function clearShortcut() {
  emit("update:modelValue", "");
}
</script>

<template>
  <div class="form-control w-full">
    <label v-if="label" class="label pb-2">
      <span class="label-text font-medium text-sm">{{ label }}</span>
    </label>
    <div class="relative">
      <input ref="inputRef" type="text" :value="displayValue" :placeholder="placeholder"
        class="input input-bordered w-full pr-20" :class="{ 'input-warning': isCapturing }" readonly
        @focus="startCapture" @blur="stopCapture" @keydown.prevent="handleKeyDown" />
      <button v-if="modelValue" type="button" class="btn btn-ghost btn-sm absolute right-1 top-1"
        @click="clearShortcut">
        ✕
      </button>
    </div>
    <label v-if="isCapturing" class="label">
      <span class="label-text-alt" :class="{ 'text-warning': isCapturing }">
        {{ $t('general.keyCaptureHint') }}
      </span>
    </label>
  </div>
</template>
