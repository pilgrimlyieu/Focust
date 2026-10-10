<script setup lang="ts">
defineProps<{
  className?: string;
}>();
</script>

<template>
  <svg xmlns="http://www.w3.org/2000/svg" :class="className" fill="none" viewBox="0 0 24 24" stroke="currentColor"
    aria-hidden="true" focusable="false">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
      d="M16 4a2.828 2.828 0 014 4L7 21H3v-4L16 4zM14 6l4 4" />
  </svg>
</template>
