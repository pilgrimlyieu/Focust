import type { AudioSettings } from "./generated/AudioSettings";
import type { BackgroundSource } from "./generated/BackgroundSource";
import type { ResolvedBackground } from "./generated/ResolvedBackground";
import type { SchedulerEvent } from "./generated/SchedulerEvent";

/** Type guards for audio sources */
function isBuiltinAudio(audio: AudioSettings) {
  return audio.source === "builtin";
}
function isFilePathAudio(audio: AudioSettings) {
  return audio.source === "filePath";
}
function isNoAudio(audio: AudioSettings) {
  return audio.source === "none";
}

/** Type guards for background sources */
function isSolidBackground(background: BackgroundSource) {
  return "solid" in background;
}
function isImagePathBackground(background: BackgroundSource) {
  return "imagePath" in background;
}
function isImageFolderBackground(background: BackgroundSource) {
  return "imageFolder" in background;
}

/**
 * Type guards for SchedulerEvent variants
 */
function isSchedulerMiniBreak(event: SchedulerEvent) {
  return event.type === "miniBreak";
}
function isSchedulerLongBreak(event: SchedulerEvent) {
  return event.type === "longBreak";
}
function isSchedulerAttention(event: SchedulerEvent) {
  return event.type === "attention";
}

/**
 * Type guards for ResolvedBackground variants
 */
function isResolvedImageBackground(background: ResolvedBackground) {
  return background.type === "image";
}
function isResolvedSolidBackground(background: ResolvedBackground) {
  return background.type === "solid";
}

export {
  isBuiltinAudio,
  isFilePathAudio,
  isNoAudio,
  isSolidBackground,
  isImagePathBackground,
  isImageFolderBackground,
  isSchedulerMiniBreak,
  isSchedulerLongBreak,
  isSchedulerAttention,
  isResolvedImageBackground,
  isResolvedSolidBackground,
};
