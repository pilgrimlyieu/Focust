import type { AudioSettings } from "./generated/AudioSettings";
import type { BackgroundSource } from "./generated/BackgroundSource";
import type { EventKind } from "./generated/EventKind";
import type { ResolvedBackground } from "./generated/ResolvedBackground";

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
 * Type guards for EventKind variants
 */
function isNotificationKind(payload: EventKind) {
  return "notification" in payload;
}
function isMiniBreak(payload: EventKind) {
  return "miniBreak" in payload;
}
function isLongBreak(payload: EventKind) {
  return "longBreak" in payload;
}
function isAttention(payload: EventKind) {
  return "attention" in payload;
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
  isNotificationKind,
  isMiniBreak,
  isLongBreak,
  isAttention,
  isResolvedImageBackground,
  isResolvedSolidBackground,
};
