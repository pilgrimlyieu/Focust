<script setup lang="ts">
defineProps<{
  className?: string;
}>();
</script>

<template>
  <svg xmlns="http://www.w3.org/2000/svg" :class="className" fill="none" viewBox="0 0 24 24" stroke="currentColor"
    aria-hidden="true" focusable="false">
    <rect x="8" y="8" width="8" height="8" rx="1" stroke-width="2" />
    <circle cx="12" cy="12" r="9" stroke-width="2" />
  </svg>
</template>
