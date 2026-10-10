<script setup lang="ts">
defineProps<{
  className?: string;
}>();
</script>

<template>
  <svg xmlns="http://www.w3.org/2000/svg" :class="className" fill="none" viewBox="0 0 24 24" stroke="currentColor"
    aria-hidden="true" focusable="false">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
      d="M12 3l2.78 5.63L21 9.54l-4.5 4.39 1.06 6.2L12 17.2l-5.56 2.93 1.06-6.2L3 9.54l6.22-.91L12 3z" />
  </svg>
</template>
