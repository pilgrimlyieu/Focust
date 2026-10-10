<script setup lang="ts">
defineProps<{
  className?: string;
}>();
</script>

<template>
  <svg xmlns="http://www.w3.org/2000/svg" :class="className" fill="none" viewBox="0 0 24 24"
    stroke="currentColor" aria-hidden="true" focusable="false">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
      d="M9 14.25 4.5 9.75 9 5.25M4.5 9.75H15a5.25 5.25 0 0 1 0 10.5h-3.75" />
  </svg>
</template>
