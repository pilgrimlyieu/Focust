<script setup lang="ts">
defineProps<{
  className?: string;
}>();
</script>

<template>
  <svg xmlns="http://www.w3.org/2000/svg" :class="className" fill="none" viewBox="0 0 512 512" stroke="currentColor">
    <defs>
      <linearGradient id="bg_vibrant" x1="256" y1="0" x2="256" y2="512" gradientUnits="userSpaceOnUse">
        <stop stop-color="#4338CA" />
        <stop offset="1" stop-color="#312E81" />
      </linearGradient>

      <linearGradient id="accent_gradient_v4" x1="0" y1="0" x2="1" y2="1">
        <stop stop-color="#5EEAD4" />
        <stop offset="1" stop-color="#0D9488" />
      </linearGradient>

      <filter id="teal_glow" x="-50%" y="-50%" width="200%" height="200%">
        <feGaussianBlur in="SourceGraphic" stdDeviation="8" result="blur" />
        <feColorMatrix in="blur" type="matrix" values="0 0 0 0 0.18  0 0 0 0 0.83  0 0 0 0 0.75  0 0 0 1 0"
          result="coloredBlur" />
        <feMerge>
          <feMergeNode in="coloredBlur" />
          <feMergeNode in="SourceGraphic" />
        </feMerge>
      </filter>
    </defs>

    <rect x="0" y="0" width="512" height="512" rx="115" fill="url(#bg_vibrant)" />

    <g transform="translate(256 256)">
      <circle cx="0" cy="0" r="145" stroke="#FFFFFF" stroke-width="46" stroke-opacity="0.12" />

      <path d="M 0 -145 A 145 145 0 1 1 -139 42" stroke="#FFFFFF" stroke-width="46" stroke-linecap="butt" fill="none" />

      <circle cx="0" cy="-145" r="23" fill="#FFFFFF" />

      <circle cx="-139" cy="42" r="26" fill="url(#accent_gradient_v4)" filter="url(#teal_glow)" />

      <rect x="-60" y="-70" width="38" height="140" rx="19" fill="#FFFFFF" />
      <rect x="22" y="-70" width="38" height="140" rx="19" fill="#FFFFFF" />
    </g>
  </svg>

</template>
