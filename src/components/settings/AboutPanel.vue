<script setup lang="ts">
import { getName, getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { markRaw, onMounted, ref } from "vue";
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
const installingUpdate = ref<boolean>(false);
const pendingUpdate = ref<Update | null>(null);

const GITHUB_REPO = "https://github.com/pilgrimlyieu/Focust";
const AUTHOR_PROFILE = "https://github.com/pilgrimlyieu";
const LICENSE_URL = "https://github.com/pilgrimlyieu/Focust/blob/main/LICENSE";

type VersionDiff = "major" | "minor" | "patch" | null;

/**
 * Parse version string and return [major, minor, patch]
 * Handles both "1.2.3" and "v1.2.3" formats
 * @param {string} version Version string
 * @returns {[number, number, number]} Parsed version parts
 */
function parseVersion(version: string): [number, number, number] {
  const cleaned = version.replace(/^v/, "");
  const parts = cleaned.split(".").map(Number);
  return [parts[0] || 0, parts[1] || 0, parts[2] || 0];
}

/**
 * Compare two version strings and determine the difference type
 * @param {string} current Current version (e.g., "0.2.11" or "v0.2.11")
 * @param {string} latest Latest version (e.g., "0.3.0" or "v0.3.0")
 * @returns {VersionDiff} Type of version difference
 */
function compareVersions(current: string, latest: string): VersionDiff {
  const [cMajor, cMinor, cPatch] = parseVersion(current);
  const [lMajor, lMinor, lPatch] = parseVersion(latest);

  if (lMajor > cMajor) return "major";
  if (lMajor < cMajor) return null;

  if (lMinor > cMinor) return "minor";
  if (lMinor < cMinor) return null;

  if (lPatch > cPatch) return "patch";

  return null;
}

/**
 * Check if version is in 0.x (zero-ver) stage
 * @param {string} version Version string
 * @returns {boolean} True if major version is 0
 */
function isZeroVer(version: string): boolean {
  const [major] = parseVersion(version);
  return major === 0;
}

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

/** Check for updates (without auto-installing) */
async function checkForUpdates() {
  checkingUpdate.value = true;
  try {
    const update = await check();

    if (!update) {
      emit("notify", "info", t("about.noUpdateAvailable"));
      return;
    }

    console.log(
      `Update available: ${update.version} (current: ${update.currentVersion})`,
    );
    console.log(`Release date: ${update.date}`);
    console.log(`Release notes: ${update.body}`);

    // Store update for later installation
    pendingUpdate.value = markRaw(update);

    // Determine version difference type
    const versionDiff = compareVersions(update.currentVersion, update.version);
    const isZero = isZeroVer(update.version);

    // Show notification based on version type
    if (versionDiff === "major") {
      emit(
        "notify",
        "info",
        t("about.updateAvailable", { version: update.version }),
      );
    } else if (versionDiff === "minor") {
      emit(
        "notify",
        "success",
        t("about.updateAvailable", { version: update.version }),
      );
    } else if (versionDiff === "patch") {
      emit(
        "notify",
        "info",
        t("about.updateAvailable", { version: update.version }),
      );
    }

    // Log zero-ver warning if applicable
    if (isZero) {
      console.log("⚠️ This is a 0.x version - expect frequent changes");
    }
  } catch (err) {
    console.error("Failed to check updates:", err);
    emit("notify", "error", t("toast.updateCheckFailed"));
  } finally {
    checkingUpdate.value = false;
  }
}

/** Install the pending update */
async function installUpdate() {
  if (!pendingUpdate.value) return;

  installingUpdate.value = true;
  try {
    console.log(`Installing update ${pendingUpdate.value.version}...`);
    await pendingUpdate.value.downloadAndInstall();

    emit("notify", "success", t("about.updateInstalled"));
    console.log("Update installed successfully");

    // Clear pending update
    pendingUpdate.value = null;
  } catch (err) {
    console.error("Failed to install update:", err);
    emit("notify", "error", t("toast.updateCheckFailed"));
  } finally {
    installingUpdate.value = false;
  }
}

/** Dismiss the pending update */
function dismissUpdate() {
  pendingUpdate.value = null;
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

        <!-- Update Available Notification -->
        <div v-if="pendingUpdate" class="alert shadow-lg" :class="{
          'alert-warning': compareVersions(pendingUpdate.currentVersion, pendingUpdate.version) === 'major',
          'alert-info': compareVersions(pendingUpdate.currentVersion, pendingUpdate.version) !== 'major'
        }">
          <div class="w-full space-y-3">
            <!-- Header -->
            <div class="flex items-start justify-between gap-3">
              <div class="flex-1">
                <h4 class="font-bold text-base flex items-center gap-2">
                  {{ t("about.newVersion") }}: v{{ pendingUpdate.version }}
                  <span v-if="compareVersions(pendingUpdate.currentVersion, pendingUpdate.version) === 'major'"
                    class="badge badge-warning badge-sm">
                    {{ t("about.majorUpdate") }}
                  </span>
                  <span v-else-if="compareVersions(pendingUpdate.currentVersion, pendingUpdate.version) === 'minor'"
                    class="badge badge-info badge-sm">
                    {{ t("about.minorUpdate") }}
                  </span>
                  <span v-else-if="compareVersions(pendingUpdate.currentVersion, pendingUpdate.version) === 'patch'"
                    class="badge badge-success badge-sm">
                    {{ t("about.patchUpdate") }}
                  </span>
                </h4>
                <p v-if="compareVersions(pendingUpdate.currentVersion, pendingUpdate.version) === 'major'"
                  class="text-sm mt-1 opacity-90">
                  {{ t("about.majorUpdateDesc") }}
                </p>
                <p v-else-if="compareVersions(pendingUpdate.currentVersion, pendingUpdate.version) === 'minor'"
                  class="text-sm mt-1 opacity-90">
                  {{ t("about.minorUpdateDesc") }}
                </p>
                <p v-else-if="compareVersions(pendingUpdate.currentVersion, pendingUpdate.version) === 'patch'"
                  class="text-sm mt-1 opacity-90">
                  {{ t("about.patchUpdateDesc") }}
                </p>
              </div>
            </div>

            <!-- Zero-ver Warning -->
            <div v-if="isZeroVer(pendingUpdate.version)" class="text-sm opacity-80">
              {{ t("about.zeroVerWarning") }}
            </div>

            <!-- Release Notes -->
            <div v-if="pendingUpdate.body" class="text-sm opacity-90">
              <button class="btn btn-link btn-xs p-0 h-auto text-sm text-primary hover:text-primary-focus"
                @click="openLink(pendingUpdate.body)">
                {{ t("about.releaseNotes") }}
              </button>
            </div>

            <!-- Action Buttons -->
            <div class="flex gap-2 pt-2">
              <button class="btn btn-sm btn-primary gap-2" :disabled="installingUpdate" @click="installUpdate">
                <span v-if="installingUpdate" class="loading loading-spinner loading-xs" />
                {{ installingUpdate ? t("about.installingUpdate") : t("about.installUpdate") }}
              </button>
              <button class="btn btn-sm btn-ghost" :disabled="installingUpdate" @click="dismissUpdate">
                {{ t("about.dismissUpdate") }}
              </button>
            </div>
          </div>
        </div>

        <!-- Check for Updates Button -->
        <button v-if="!pendingUpdate" class="btn btn-primary gap-2 shadow-md hover:shadow-lg transition-all"
          :disabled="checkingUpdate" @click="checkForUpdates">
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
        <span class="badge badge-lg gap-2">Tauri 2</span>
        <span class="badge badge-lg gap-2">Vue 3</span>
        <span class="badge badge-lg gap-2">Rust 2024</span>
        <span class="badge badge-lg gap-2">TypeScript</span>
        <span class="badge badge-lg gap-2">Tailwind CSS</span>
        <span class="badge badge-lg gap-2">DaisyUI</span>
      </div>
    </div>
  </section>
</template>
