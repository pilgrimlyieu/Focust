<script setup lang="ts">
import { getName, getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { check } from "@tauri-apps/plugin-updater";
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import ExternalLinkIcon from "@/components/icons/ExternalLinkIcon.vue";
import InfoCircleIcon from "@/components/icons/InfoCircleIcon.vue";
import type { ToastKind } from "@/composables/useToast";

const emit =
  defineEmits<(event: "notify", kind: ToastKind, message: string) => void>();

const { t } = useI18n();

const appName = ref<string>("Focust");
const appVersion = ref<string>("0.1.0");
const checkingUpdate = ref<boolean>(false);

const GITHUB_REPO = "https://github.com/pilgrimlyieu/Focust";
const AUTHOR_PROFILE = "https://github.com/pilgrimlyieu";
const LICENSE_URL = "https://github.com/pilgrimlyieu/Focust/blob/main/LICENSE";

/** Load application info on mount */
onMounted(async () => {
  try {
    appName.value = await getName();
    appVersion.value = await getVersion();
  } catch (err) {
    console.error("Failed to load app info:", err);
  }
});

/**
 * Open URL in the default browser
 * @param {string} url The URL to open
 */
async function openLink(url: string) {
  try {
    await openUrl(url);
  } catch (err) {
    console.error("Failed to open URL:", err);
    emit("notify", "error", t("toast.openUrlFailed", { url }));
  }
}

/** Check for updates */
async function checkForUpdates() {
  checkingUpdate.value = true;
  try {
    const update = await check();
    if (update) {
      console.log(
        `found update ${update.version} from ${update.date} with notes ${update.body}`,
      );
      await update.downloadAndInstall();
      console.log("update installed");
    }
  } catch (err) {
    console.error("Failed to check updates:", err);
    emit("notify", "error", t("toast.updateCheckFailed"));
  } finally {
    checkingUpdate.value = false;
  }
}
</script>

<template>
  <section class="space-y-6">
    <!-- Header Card -->
    <div
      class="rounded-2xl border border-info/30 bg-linear-to-br from-info/10 via-info/5 to-transparent p-6 shadow-sm backdrop-blur-sm">
      <div class="flex flex-col sm:flex-row items-start gap-5">
        <div
          class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-linear-to-br from-info to-info/80 shadow-lg">
          <InfoCircleIcon class-name="h-7 w-7 text-white" />
        </div>
        <div class="flex-1 min-w-0">
          <h2 class="text-2xl font-bold text-base-content mb-2.5">
            {{ t("about.title") }}
          </h2>
          <p class="text-sm text-base-content/70 leading-relaxed">
            {{ t("about.appDescription") }}
          </p>
        </div>
      </div>
    </div>

    <!-- Version Info Card -->
    <div class="rounded-2xl border border-base-300 bg-base-100/70 p-6 shadow-md">
      <h3 class="text-lg font-bold mb-4 flex items-center gap-2">
        {{ t("about.version") }}
      </h3>
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <span class="text-sm text-base-content/70">{{ t("about.currentVersion") }}</span>
          <span class="badge badge-primary badge-lg font-mono">v{{ appVersion }}</span>
        </div>
        <button class="btn btn-primary gap-2 shadow-md hover:shadow-lg transition-all" :disabled="checkingUpdate"
          @click="checkForUpdates">
          <span v-if="checkingUpdate" class="loading loading-spinner loading-sm" />
          <span>{{ checkingUpdate ? t("about.checkingUpdates") : t("about.checkForUpdates") }}</span>
        </button>
      </div>
    </div>

    <!-- Project Links Card -->
    <div class="rounded-2xl border border-base-300 bg-base-100/70 p-6 shadow-md">
      <h3 class="text-lg font-bold mb-4">{{ t("about.github") }}</h3>
      <div class="space-x-5">
        <button class="btn btn-outline gap-2 shadow-sm hover:shadow-md transition-all" @click="openLink(GITHUB_REPO)">
          <ExternalLinkIcon class-name="h-5 w-5" />
          {{ t("about.starOnGitHub") }}
        </button>
        <button class="btn btn-ghost gap-2 shadow-sm hover:shadow-md transition-all" @click="openLink(LICENSE_URL)">
          <ExternalLinkIcon class-name="h-5 w-5" />
          {{ t("about.viewLicense") }} {{ t("about.projectLicense") }}
        </button>
      </div>
    </div>

    <!-- Author Info Card -->
    <div class="rounded-2xl border border-base-300 bg-base-100/70 p-6 shadow-md">
      <h3 class="text-lg font-bold mb-4">{{ t("about.author") }}</h3>
      <button class="btn btn-ghost gap-2 shadow-sm hover:shadow-md transition-all" @click="openLink(AUTHOR_PROFILE)">
        <ExternalLinkIcon class-name="h-5 w-5" />
        {{ t("about.authorName") }}
      </button>
    </div>

    <!-- Credits Card -->
    <div class="rounded-2xl border border-base-300 bg-base-100/70 p-6 shadow-md">
      <h3 class="text-lg font-bold mb-4">Built With</h3>
      <div class="flex flex-wrap gap-2">
        <span class="badge badge-lg gap-2">
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 0L2.5 6v12L12 24l9.5-6V6L12 0zm0 2.2l7.5 4.7v9.4L12 21.8l-7.5-4.7V6.9L12 2.2z" />
          </svg>
          Tauri 2.9
        </span>
        <span class="badge badge-lg gap-2">
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M2 12.5L12 2l10 10.5-10 9.5L2 12.5z" />
          </svg>
          Vue 3.5
        </span>
        <span class="badge badge-lg gap-2">
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path
              d="M23.8 12.7c0-.7-.6-1.2-1.2-1.2h-2.8c-.1-1.2-.4-2.4-.9-3.5l2.4-1.4c.6-.4.8-1.1.5-1.7-.4-.6-1.1-.8-1.7-.5l-2.4 1.4c-.8-1-1.7-1.9-2.7-2.7l1.4-2.4c.4-.6.2-1.3-.5-1.7-.6-.4-1.3-.2-1.7.5L12.8 2c-1.1-.5-2.3-.8-3.5-.9V.5c0-.7-.6-1.2-1.2-1.2-.7 0-1.2.6-1.2 1.2v2.8c-1.2.1-2.4.4-3.5.9L1.9.9C1.5.3.8.1.2.5c-.6.4-.8 1.1-.5 1.7L2.1 4.6c-1 .8-1.9 1.7-2.7 2.7L-2 5.9c-.6-.4-1.3-.2-1.7.5-.4.6-.2 1.3.5 1.7l2.4 1.4c-.5 1.1-.8 2.3-.9 3.5H-4c-.7 0-1.2.6-1.2 1.2s.6 1.2 1.2 1.2h2.8c.1 1.2.4 2.4.9 3.5l-2.4 1.4c-.6.4-.8 1.1-.5 1.7.3.4.7.6 1.1.6.2 0 .4-.1.6-.2l2.4-1.4c.8 1 1.7 1.9 2.7 2.7l-1.4 2.4c-.4.6-.2 1.3.5 1.7.2.1.4.2.6.2.4 0 .8-.2 1.1-.6l1.4-2.4c1.1.5 2.3.8 3.5.9v2.8c0 .7.6 1.2 1.2 1.2.7 0 1.2-.6 1.2-1.2v-2.8c1.2-.1 2.4-.4 3.5-.9l1.4 2.4c.3.4.7.6 1.1.6.2 0 .4-.1.6-.2.6-.4.8-1.1.5-1.7l-1.4-2.4c1-.8 1.9-1.7 2.7-2.7l2.4 1.4c.2.1.4.2.6.2.4 0 .8-.2 1.1-.6.4-.6.2-1.3-.5-1.7l-2.4-1.4c.5-1.1.8-2.3.9-3.5h2.8c.7 0 1.2-.5 1.2-1.2z" />
          </svg>
          Rust 2024
        </span>
        <span class="badge badge-lg gap-2">TypeScript</span>
        <span class="badge badge-lg gap-2">Tailwind CSS</span>
        <span class="badge badge-lg gap-2">DaisyUI</span>
      </div>
    </div>
  </section>
</template>
