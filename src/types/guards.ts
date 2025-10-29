import type { AudioSettings } from "./generated/AudioSettings";
import type { BackgroundSource } from "./generated/BackgroundSource";
import type { EventKind } from "./generated/EventKind";

/** Type guards for audio sources */
function isBuiltinAudio(audio: AudioSettings) {
  return audio.source === "Builtin";
}
function isFilePathAudio(audio: AudioSettings) {
  return audio.source === "FilePath";
}
function isNoAudio(audio: AudioSettings) {
  return audio.source === "None";
}

/** Type guards for background sources */
function isSolidBackground(background: BackgroundSource) {
  return "Solid" in background;
}
function isImagePathBackground(background: BackgroundSource) {
  return "ImagePath" in background;
}
function isImageFolderBackground(background: BackgroundSource) {
  return "ImageFolder" in background;
}

/**
 * Type guards for EventKind variants
 */
function isNotificationKind(payload: EventKind) {
  return "Notification" in payload;
}
function isMiniBreak(payload: EventKind) {
  return "MiniBreak" in payload;
}
function isLongBreak(payload: EventKind) {
  return "LongBreak" in payload;
}
function isAttention(payload: EventKind) {
  return "Attention" in payload;
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
};
