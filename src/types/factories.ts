/**
 * Type factories and conversion utilities
 *
 * This module provides factory functions and conversion utilities for
 * working with generated types. It abstracts away the underlying data
 * structure details, making the codebase more maintainable when types change.
 */

import type { AudioSettings } from "./generated/AudioSettings";
import type { BackgroundSource } from "./generated/BackgroundSource";
import type { FontFamily } from "./generated/FontFamily";
import type { HexColor } from "./generated/HexColor";
import type { ResolvedBackground } from "./generated/ResolvedBackground";
import type { SuggestionsSettings } from "./generated/SuggestionsSettings";
import type { ThemeSettings } from "./generated/ThemeSettings";
import type { TimeRange } from "./generated/TimeRange";
import {
  isBuiltinAudio,
  isFilePathAudio,
  isImageFolderBackground,
  isImagePathBackground,
  isSolidBackground,
} from "./guards"; // ============================================================================
// BackgroundSource Factory
// ============================================================================

/**
 * Create a solid color background
 * @param {HexColor} color Hex color string (e.g., "#1f2937")
 * @returns {BackgroundSource} BackgroundSource with solid variant
 */
export function createSolidBackground(color: HexColor): BackgroundSource {
  return { solid: color };
}

/**
 * Create an image path background
 * @param {string} path Absolute path to image file
 * @returns {BackgroundSource} BackgroundSource with imagePath variant
 */
export function createImagePathBackground(path: string): BackgroundSource {
  return { imagePath: path };
}

/**
 * Create an image folder background
 * @param {string} folder Absolute path to image folder
 * @returns {BackgroundSource} BackgroundSource with imageFolder variant
 */
export function createImageFolderBackground(folder: string): BackgroundSource {
  return { imageFolder: folder };
}

/**
 * Get the color value from a solid background
 * @param {BackgroundSource} background BackgroundSource to extract from
 * @returns {HexColor | null} Color value or null if not solid
 */
export function getSolidColor(background: BackgroundSource): HexColor | null {
  return isSolidBackground(background) ? background.solid : null;
}

/**
 * Get the image path from an imagePath background
 * @param {BackgroundSource} background BackgroundSource to extract from
 * @returns {string | null} Image path or null if not imagePath
 */
export function getImagePath(background: BackgroundSource): string | null {
  return isImagePathBackground(background) ? background.imagePath : null;
}

/**
 * Get the image folder from an imageFolder background
 * @param {BackgroundSource} background BackgroundSource to extract from
 * @returns {string | null} Image folder path or null if not imageFolder
 */
export function getImageFolder(background: BackgroundSource): string | null {
  return isImageFolderBackground(background) ? background.imageFolder : null;
}

/**
 * Set a new color for a solid background (mutating)
 * @param {BackgroundSource} background BackgroundSource to modify
 * @param {HexColor} color New color value
 * @returns {boolean} true if modified, false if not solid background
 */
export function setSolidColor(
  background: BackgroundSource,
  color: HexColor,
): boolean {
  if (!isSolidBackground(background)) {
    return false;
  }
  background.solid = color;
  return true;
}

/**
 * Convert any BackgroundSource to a solid background (mutating)
 * @param {BackgroundSource} background BackgroundSource to convert
 * @param {HexColor} color Color value
 */
export function convertToSolidBackground(
  background: BackgroundSource,
  color: HexColor,
) {
  const bg = background as Record<string, unknown>;
  // Remove all existing properties
  delete bg.imagePath;
  delete bg.imageFolder;
  // Set new property
  bg.solid = color;
}

/**
 * Convert any BackgroundSource to an image path background (mutating)
 * @param {BackgroundSource} background BackgroundSource to convert
 * @param {string} path Image path
 */
export function convertToImagePathBackground(
  background: BackgroundSource,
  path: string,
) {
  const bg = background as Record<string, unknown>;
  delete bg.solid;
  delete bg.imageFolder;
  bg.imagePath = path;
}

/**
 * Convert any BackgroundSource to an image folder background (mutating)
 * @param {BackgroundSource} background BackgroundSource to convert
 * @param {string} folder Image folder path
 */
export function convertToImageFolderBackground(
  background: BackgroundSource,
  folder: string,
) {
  const bg = background as Record<string, unknown>;
  delete bg.solid;
  delete bg.imagePath;
  bg.imageFolder = folder;
}

// ============================================================================
// AudioSettings Factory
// ============================================================================

/**
 * Create an audio settings with no sound
 * @param {number} volume Volume level (0.0 to 1.0)
 * @returns {AudioSettings} AudioSettings with none source
 */
export function createNoAudio(volume: number = 0.6): AudioSettings {
  return { source: "none", volume };
}

/**
 * Create an audio settings with built-in sound
 * @param {string} name Built-in sound name
 * @param {number} volume Volume level (0.0 to 1.0)
 * @returns {AudioSettings} AudioSettings with builtin source
 */
export function createBuiltinAudio(
  name: string,
  volume: number = 0.6,
): AudioSettings {
  return { name, source: "builtin", volume };
}

/**
 * Create an audio settings with custom file
 * @param {string} path Absolute path to audio file
 * @param {number} volume Volume level (0.0 to 1.0)
 * @returns {AudioSettings} AudioSettings with filePath source
 */
export function createFilePathAudio(
  path: string,
  volume: number = 0.6,
): AudioSettings {
  return { path, source: "filePath", volume };
}

/**
 * Get the built-in audio name
 * @param {AudioSettings} audio AudioSettings to extract from
 * @returns {string | null} Audio name or null if not builtin
 */
export function getBuiltinAudioName(audio: AudioSettings): string | null {
  return isBuiltinAudio(audio) ? audio.name : null;
}

/**
 * Get the audio file path
 * @param {AudioSettings} audio AudioSettings to extract from
 * @returns {string | null} File path or null if not filePath
 */
export function getAudioFilePath(audio: AudioSettings): string | null {
  return isFilePathAudio(audio) ? audio.path : null;
}

/**
 * Convert any AudioSettings to none (mutating)
 * @param {AudioSettings} audio AudioSettings to convert
 */
export function convertToNoAudio(audio: AudioSettings) {
  const a = audio as Record<string, unknown>;
  a.source = "none";
  delete a.name;
  delete a.path;
}

/**
 * Convert any AudioSettings to builtin (mutating)
 * @param {AudioSettings} audio AudioSettings to convert
 * @param {string} name Built-in audio name
 */
export function convertToBuiltinAudio(audio: AudioSettings, name: string) {
  const a = audio as Record<string, unknown>;
  a.source = "builtin";
  a.name = name;
  delete a.path;
}

/**
 * Convert any AudioSettings to filePath (mutating)
 * @param {AudioSettings} audio AudioSettings to convert
 * @param {string} path Audio file path
 */
export function convertToFilePathAudio(audio: AudioSettings, path: string) {
  const a = audio as Record<string, unknown>;
  a.source = "filePath";
  a.path = path;
  delete a.name;
}

/**
 * Get audio source type as a simple string
 * @param {AudioSettings} audio AudioSettings to check
 * @returns { "none" | "builtin" | "filePath" } Audio source type
 */
export function getAudioSourceType(
  audio: AudioSettings,
): "none" | "builtin" | "filePath" {
  if (isBuiltinAudio(audio)) return "builtin";
  if (isFilePathAudio(audio)) return "filePath";
  return "none";
}

// ============================================================================
// ResolvedBackground Factory
// ============================================================================

/**
 * Create a resolved solid background
 * @param {HexColor} color Hex color value
 * @returns {ResolvedBackground} ResolvedBackground with solid type
 */
export function createResolvedSolidBackground(
  color: HexColor,
): ResolvedBackground {
  return { type: "solid", value: color };
}

/**
 * Create a resolved image background
 * @param {string} path Image file path
 * @returns {ResolvedBackground} ResolvedBackground with image type
 */
export function createResolvedImageBackground(
  path: string,
): ResolvedBackground {
  return { type: "image", value: path };
}

/**
 * Get the value from a resolved background
 * @param {ResolvedBackground} background ResolvedBackground to extract from
 * @returns {string} Background value (color or image path)
 */
export function getResolvedBackgroundValue(
  background: ResolvedBackground,
): string {
  return background.value;
}

/**
 * Get the type of a resolved background
 * @param {ResolvedBackground} background ResolvedBackground to check
 * @returns { "solid" | "image" } Background type
 */
export function getResolvedBackgroundType(
  background: ResolvedBackground,
): "solid" | "image" {
  return background.type;
}

// ============================================================================
// ThemeSettings Factory
// ============================================================================

/**
 * Create default theme settings
 * @param {Object} options Optional overrides for default settings
 * @returns {ThemeSettings} ThemeSettings object with defaults
 */
export function createDefaultTheme(options?: {
  backgroundColor?: HexColor;
  textColor?: HexColor;
  blurRadius?: number;
  opacity?: number;
  fontSize?: number;
  fontFamily?: FontFamily;
}): ThemeSettings {
  return {
    background: createSolidBackground(options?.backgroundColor ?? "#1f2937"),
    blurRadius: options?.blurRadius ?? 8,
    fontFamily: options?.fontFamily ?? "Arial",
    fontSize: options?.fontSize ?? 24,
    opacity: options?.opacity ?? 0.9,
    textColor: options?.textColor ?? "#f8fafc",
  };
}

/**
 * Clone theme settings with overrides
 * @param {ThemeSettings} theme Original theme settings
 * @param {Partial<ThemeSettings>} overrides Properties to override
 * @returns {ThemeSettings} New ThemeSettings object
 */
export function cloneTheme(
  theme: ThemeSettings,
  overrides?: Partial<ThemeSettings>,
): ThemeSettings {
  return {
    ...theme,
    ...overrides,
  };
}

/**
 * Update theme background (mutating)
 * @param {ThemeSettings} theme ThemeSettings to modify
 * @param {BackgroundSource} background New background source
 */
export function setThemeBackground(
  theme: ThemeSettings,
  background: BackgroundSource,
) {
  theme.background = background;
}

/**
 * Update theme text color (mutating)
 * @param {ThemeSettings} theme ThemeSettings to modify
 * @param {HexColor} color New text color
 */
export function setThemeTextColor(theme: ThemeSettings, color: HexColor) {
  theme.textColor = color;
}

/**
 * Update theme blur radius (mutating)
 * @param {ThemeSettings} theme ThemeSettings to modify
 * @param {number} radius New blur radius (0-50)
 */
export function setThemeBlurRadius(theme: ThemeSettings, radius: number) {
  theme.blurRadius = Math.max(0, Math.min(50, radius));
}

/**
 * Update theme opacity (mutating)
 * @param {ThemeSettings} theme ThemeSettings to modify
 * @param {number} opacity New opacity (0.0-1.0)
 */
export function setThemeOpacity(theme: ThemeSettings, opacity: number) {
  theme.opacity = Math.max(0, Math.min(1, opacity));
}

/**
 * Update theme font size (mutating)
 * @param {ThemeSettings} theme ThemeSettings to modify
 * @param {number} size New font size in pixels
 */
export function setThemeFontSize(theme: ThemeSettings, size: number) {
  theme.fontSize = Math.max(8, Math.min(100, size));
}

/**
 * Update theme font family (mutating)
 * @param {ThemeSettings} theme ThemeSettings to modify
 * @param {FontFamily} fontFamily New font family
 */
export function setThemeFontFamily(
  theme: ThemeSettings,
  fontFamily: FontFamily,
) {
  theme.fontFamily = fontFamily;
}

// ============================================================================
// TimeRange Factory
// ============================================================================

/**
 * Create a time range
 * @param {string} start Start time in "HH:MM" format (e.g., "09:00")
 * @param {string} end End time in "HH:MM" format (e.g., "18:00")
 * @returns {TimeRange} TimeRange object
 */
export function createTimeRange(start: string, end: string): TimeRange {
  return { end, start };
}

/**
 * Create a time range for all-day (00:00 to 00:00)
 * @returns {TimeRange} TimeRange for all-day schedule
 */
export function createAllDayTimeRange(): TimeRange {
  return { end: "00:00", start: "00:00" };
}

/**
 * Check if a time range represents all-day
 * @param {TimeRange} timeRange TimeRange to check
 * @returns {boolean} true if all-day, false otherwise
 */
export function isAllDayTimeRange(timeRange: TimeRange): boolean {
  return timeRange.start === "00:00" && timeRange.end === "00:00";
}

/**
 * Get start time from time range
 * @param {TimeRange} timeRange TimeRange to extract from
 * @returns {string} Start time string
 */
export function getTimeRangeStart(timeRange: TimeRange): string {
  return timeRange.start;
}

/**
 * Get end time from time range
 * @param {TimeRange} timeRange TimeRange to extract from
 * @returns {string} End time string
 */
export function getTimeRangeEnd(timeRange: TimeRange): string {
  return timeRange.end;
}

/**
 * Update start time of time range (mutating)
 * @param {TimeRange} timeRange TimeRange to modify
 * @param {string} start New start time
 */
export function setTimeRangeStart(timeRange: TimeRange, start: string) {
  timeRange.start = start;
}

/**
 * Update end time of time range (mutating)
 * @param {TimeRange} timeRange TimeRange to modify
 * @param {string} end New end time
 */
export function setTimeRangeEnd(timeRange: TimeRange, end: string) {
  timeRange.end = end;
}

/**
 * Clone a time range
 * @param {TimeRange} timeRange Original time range
 * @returns {TimeRange} New TimeRange object
 */
export function cloneTimeRange(timeRange: TimeRange): TimeRange {
  return { end: timeRange.end, start: timeRange.start };
}

// ============================================================================
// SuggestionsSettings Factory
// ============================================================================

/**
 * Create suggestions settings
 * @param {boolean} show Whether to show suggestions
 * @returns {SuggestionsSettings} SuggestionsSettings object
 */
export function createSuggestionsSettings(
  show: boolean = true,
): SuggestionsSettings {
  return { show };
}

/**
 * Check if suggestions should be shown
 * @param {SuggestionsSettings} settings SuggestionsSettings to check
 * @returns {boolean} true if suggestions should be shown
 */
export function shouldShowSuggestions(settings: SuggestionsSettings): boolean {
  return settings.show;
}

/**
 * Update whether to show suggestions (mutating)
 * @param {SuggestionsSettings} settings SuggestionsSettings to modify
 * @param {boolean} show New show value
 */
export function setShowSuggestions(
  settings: SuggestionsSettings,
  show: boolean,
) {
  settings.show = show;
}

// ============================================================================
// Generic Property Accessors
// ============================================================================

/**
 * Generic getter for simple object properties
 * Reduces boilerplate for basic property access
 *
 * @example
 * const theme = createDefaultTheme();
 * const fontSize = getProp(theme, 'fontSize'); // Type-safe!
 *
 * @template T Object type
 * @template K Key of object type
 * @param {T} obj Object to get property from
 * @param {K} key Property key
 * @returns {T[K]} Property value
 */
export function getProp<T, K extends keyof T>(obj: T, key: K): T[K] {
  return obj[key];
}

/**
 * Generic setter for simple object properties
 * Reduces boilerplate for basic property updates
 *
 * @example
 * const theme = createDefaultTheme();
 * setProp(theme, 'fontSize', 32); // Type-safe!
 *
 * @template T Object type
 * @template K Key of object type
 * @param {T} obj Object to modify
 * @param {K} key Property key
 * @param {T[K]} value New property value
 */
export function setProp<T, K extends keyof T>(obj: T, key: K, value: T[K]) {
  obj[key] = value;
}
