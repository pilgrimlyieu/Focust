<script setup lang="ts">
defineProps<{
  className?: string;
}>();
</script>

<template>
  <svg xmlns="http://www.w3.org/2000/svg" :class="className" fill="none" viewBox="0 0 24 24" stroke="currentColor"
    aria-hidden="true" focusable="false">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
      d="M10.3 4.8a2 2 0 013.4 0l7.6 12.7a2 2 0 01-1.7 3H4.4a2 2 0 01-1.7-3l7.6-12.7zM12 9v4m0 4h.01" />
  </svg>
</template>
