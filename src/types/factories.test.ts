/**
 * Unit tests for type factories and conversion utilities
 */

import { describe, expect, it } from "vitest";
import {
  cloneTheme,
  // TimeRange factories
  cloneTimeRange,
  convertToBuiltinAudio,
  convertToFilePathAudio,
  convertToImageFolderBackground,
  convertToImagePathBackground,
  convertToNoAudio,
  convertToSolidBackground,
  createAllDayTimeRange,
  createBuiltinAudio,
  // ThemeSettings factories
  createDefaultTheme,
  createFilePathAudio,
  createImageFolderBackground,
  createImagePathBackground,
  // AudioSettings factories
  createNoAudio,
  createResolvedImageBackground,
  // ResolvedBackground factories
  createResolvedSolidBackground,
  // BackgroundSource factories
  createSolidBackground,
  // SuggestionsSettings factories
  createSuggestionsSettings,
  createTimeRange,
  getAudioFilePath,
  getAudioSourceType,
  getBuiltinAudioName,
  getImageFolder,
  getImagePath,
  // Generic accessors
  getProp,
  getResolvedBackgroundType,
  getResolvedBackgroundValue,
  getSolidColor,
  getTimeRangeEnd,
  getTimeRangeStart,
  isAllDayTimeRange,
  setProp,
  setShowSuggestions,
  setSolidColor,
  setThemeBackground,
  setThemeBlurRadius,
  setThemeFontFamily,
  setThemeFontSize,
  setThemeOpacity,
  setThemeTextColor,
  setTimeRangeEnd,
  setTimeRangeStart,
  shouldShowSuggestions,
} from "./factories";
import type { AudioSettings } from "./generated/AudioSettings";
import type { BackgroundSource } from "./generated/BackgroundSource";

// ============================================================================
// BackgroundSource Tests
// ============================================================================

describe("BackgroundSource Factories", () => {
  describe("createSolidBackground", () => {
    it("should create a solid background with given color", () => {
      const bg = createSolidBackground("#1f2937");
      expect(bg.current).toBe("solid");
      expect(bg.solid).toBe("#1f2937");
      expect(bg.imagePath).toBeNull();
      expect(bg.imageFolder).toBeNull();
    });
  });

  describe("createImagePathBackground", () => {
    it("should create an image path background", () => {
      const bg = createImagePathBackground("/path/to/image.jpg");
      expect(bg.current).toBe("imagePath");
      expect(bg.imagePath).toBe("/path/to/image.jpg");
      expect(bg.solid).toBeNull();
      expect(bg.imageFolder).toBeNull();
    });
  });

  describe("createImageFolderBackground", () => {
    it("should create an image folder background", () => {
      const bg = createImageFolderBackground("/path/to/folder");
      expect(bg.current).toBe("imageFolder");
      expect(bg.imageFolder).toBe("/path/to/folder");
      expect(bg.solid).toBeNull();
      expect(bg.imagePath).toBeNull();
    });
  });

  describe("getSolidColor", () => {
    it("should extract color from solid background", () => {
      const bg = createSolidBackground("#1f2937");
      expect(getSolidColor(bg)).toBe("#1f2937");
    });

    it("should return null for non-solid background", () => {
      const bg = createImagePathBackground("/path/to/image.jpg");
      expect(getSolidColor(bg)).toBeNull();
    });
  });

  describe("getImagePath", () => {
    it("should extract path from imagePath background", () => {
      const bg = createImagePathBackground("/path/to/image.jpg");
      expect(getImagePath(bg)).toBe("/path/to/image.jpg");
    });

    it("should return null for non-imagePath background", () => {
      const bg = createSolidBackground("#1f2937");
      expect(getImagePath(bg)).toBeNull();
    });
  });

  describe("getImageFolder", () => {
    it("should extract folder from imageFolder background", () => {
      const bg = createImageFolderBackground("/path/to/folder");
      expect(getImageFolder(bg)).toBe("/path/to/folder");
    });

    it("should return null for non-imageFolder background", () => {
      const bg = createSolidBackground("#1f2937");
      expect(getImageFolder(bg)).toBeNull();
    });
  });

  describe("setSolidColor", () => {
    it("should update color for solid background", () => {
      const bg = createSolidBackground("#1f2937");
      const result = setSolidColor(bg, "#ffffff");
      expect(result).toBe(true);
      expect(getSolidColor(bg)).toBe("#ffffff");
    });

    it("should return false for non-solid background", () => {
      const bg = createImagePathBackground("/path/to/image.jpg");
      const result = setSolidColor(bg, "#ffffff");
      expect(result).toBe(false);
      expect(getImagePath(bg)).toBe("/path/to/image.jpg");
    });
  });

  describe("convertToSolidBackground", () => {
    it("should convert imagePath to solid", () => {
      const bg: BackgroundSource =
        createImagePathBackground("/path/to/image.jpg");
      convertToSolidBackground(bg, "#1f2937");
      expect(bg.current).toBe("solid");
      expect(getSolidColor(bg)).toBe("#1f2937");
      // Old value is preserved
      expect(getImagePath(bg)).toBe("/path/to/image.jpg");
    });

    it("should convert imageFolder to solid", () => {
      const bg: BackgroundSource =
        createImageFolderBackground("/path/to/folder");
      convertToSolidBackground(bg, "#1f2937");
      expect(bg.current).toBe("solid");
      expect(getSolidColor(bg)).toBe("#1f2937");
      // Old value is preserved
      expect(getImageFolder(bg)).toBe("/path/to/folder");
    });

    it("should update existing solid background", () => {
      const bg: BackgroundSource = createSolidBackground("#ffffff");
      convertToSolidBackground(bg, "#1f2937");
      expect(bg.current).toBe("solid");
      expect(getSolidColor(bg)).toBe("#1f2937");
    });
  });

  describe("convertToImagePathBackground", () => {
    it("should convert solid to imagePath", () => {
      const bg: BackgroundSource = createSolidBackground("#1f2937");
      convertToImagePathBackground(bg, "/path/to/image.jpg");
      expect(bg.current).toBe("imagePath");
      expect(getImagePath(bg)).toBe("/path/to/image.jpg");
      // Old value is preserved
      expect(getSolidColor(bg)).toBe("#1f2937");
    });

    it("should convert imageFolder to imagePath", () => {
      const bg: BackgroundSource =
        createImageFolderBackground("/path/to/folder");
      convertToImagePathBackground(bg, "/path/to/image.jpg");
      expect(bg.current).toBe("imagePath");
      expect(getImagePath(bg)).toBe("/path/to/image.jpg");
      // Old value is preserved
      expect(getImageFolder(bg)).toBe("/path/to/folder");
    });
  });

  describe("convertToImageFolderBackground", () => {
    it("should convert solid to imageFolder", () => {
      const bg: BackgroundSource = createSolidBackground("#1f2937");
      convertToImageFolderBackground(bg, "/path/to/folder");
      expect(bg.current).toBe("imageFolder");
      expect(getImageFolder(bg)).toBe("/path/to/folder");
      // Old value is preserved
      expect(getSolidColor(bg)).toBe("#1f2937");
    });

    it("should convert imagePath to imageFolder", () => {
      const bg: BackgroundSource =
        createImagePathBackground("/path/to/image.jpg");
      convertToImageFolderBackground(bg, "/path/to/folder");
      expect(bg.current).toBe("imageFolder");
      expect(getImageFolder(bg)).toBe("/path/to/folder");
      // Old value is preserved
      expect(getImagePath(bg)).toBe("/path/to/image.jpg");
    });
  });
});

// ============================================================================
// AudioSettings Tests
// ============================================================================

describe("AudioSettings Factories", () => {
  describe("createNoAudio", () => {
    it("should create no audio with default volume", () => {
      const audio = createNoAudio();
      expect(audio.source.current).toBe("none");
      expect(audio.source.builtinName).toBeNull();
      expect(audio.source.filePath).toBeNull();
      expect(audio.volume).toBe(0.6);
    });

    it("should create no audio with custom volume", () => {
      const audio = createNoAudio(0.8);
      expect(audio.source.current).toBe("none");
      expect(audio.volume).toBe(0.8);
    });
  });

  describe("createBuiltinAudio", () => {
    it("should create builtin audio with default volume", () => {
      const audio = createBuiltinAudio("gentle-bell");
      expect(audio.source.current).toBe("builtin");
      expect(audio.source.builtinName).toBe("gentle-bell");
      expect(audio.source.filePath).toBeNull();
      expect(audio.volume).toBe(0.6);
    });

    it("should create builtin audio with custom volume", () => {
      const audio = createBuiltinAudio("soft-gong", 0.5);
      expect(audio.source.current).toBe("builtin");
      expect(audio.source.builtinName).toBe("soft-gong");
      expect(audio.volume).toBe(0.5);
    });
  });

  describe("createFilePathAudio", () => {
    it("should create filePath audio with default volume", () => {
      const audio = createFilePathAudio("/path/to/audio.mp3");
      expect(audio.source.current).toBe("filePath");
      expect(audio.source.builtinName).toBeNull();
      expect(audio.source.filePath).toBe("/path/to/audio.mp3");
      expect(audio.volume).toBe(0.6);
    });

    it("should create filePath audio with custom volume", () => {
      const audio = createFilePathAudio("/path/to/audio.mp3", 0.7);
      expect(audio.source.current).toBe("filePath");
      expect(audio.source.filePath).toBe("/path/to/audio.mp3");
      expect(audio.volume).toBe(0.7);
    });
  });

  describe("getBuiltinAudioName", () => {
    it("should extract name from builtin audio", () => {
      const audio = createBuiltinAudio("gentle-bell");
      expect(getBuiltinAudioName(audio)).toBe("gentle-bell");
    });

    it("should return null for non-builtin audio", () => {
      const audio = createNoAudio();
      expect(getBuiltinAudioName(audio)).toBeNull();
    });
  });

  describe("getAudioFilePath", () => {
    it("should extract path from filePath audio", () => {
      const audio = createFilePathAudio("/path/to/audio.mp3");
      expect(getAudioFilePath(audio)).toBe("/path/to/audio.mp3");
    });

    it("should return null for non-filePath audio", () => {
      const audio = createNoAudio();
      expect(getAudioFilePath(audio)).toBeNull();
    });
  });

  describe("getAudioSourceType", () => {
    it("should return 'none' for no audio", () => {
      const audio = createNoAudio();
      expect(getAudioSourceType(audio)).toBe("none");
    });

    it("should return 'builtin' for builtin audio", () => {
      const audio = createBuiltinAudio("gentle-bell");
      expect(getAudioSourceType(audio)).toBe("builtin");
    });

    it("should return 'filePath' for filePath audio", () => {
      const audio = createFilePathAudio("/path/to/audio.mp3");
      expect(getAudioSourceType(audio)).toBe("filePath");
    });
  });

  describe("convertToNoAudio", () => {
    it("should convert builtin to none", () => {
      const audio: AudioSettings = createBuiltinAudio("gentle-bell");
      convertToNoAudio(audio);
      expect(getAudioSourceType(audio)).toBe("none");
      // Old value is preserved
      expect(getBuiltinAudioName(audio)).toBe("gentle-bell");
    });

    it("should convert filePath to none", () => {
      const audio: AudioSettings = createFilePathAudio("/path/to/audio.mp3");
      convertToNoAudio(audio);
      expect(getAudioSourceType(audio)).toBe("none");
      // Old value is preserved
      expect(getAudioFilePath(audio)).toBe("/path/to/audio.mp3");
    });

    it("should preserve volume", () => {
      const audio: AudioSettings = createBuiltinAudio("gentle-bell", 0.8);
      convertToNoAudio(audio);
      expect(audio.volume).toBe(0.8);
    });
  });

  describe("convertToBuiltinAudio", () => {
    it("should convert none to builtin", () => {
      const audio: AudioSettings = createNoAudio();
      convertToBuiltinAudio(audio, "gentle-bell");
      expect(getAudioSourceType(audio)).toBe("builtin");
      expect(getBuiltinAudioName(audio)).toBe("gentle-bell");
    });

    it("should convert filePath to builtin", () => {
      const audio: AudioSettings = createFilePathAudio("/path/to/audio.mp3");
      convertToBuiltinAudio(audio, "soft-gong");
      expect(getAudioSourceType(audio)).toBe("builtin");
      expect(getBuiltinAudioName(audio)).toBe("soft-gong");
      // Old value is preserved
      expect(getAudioFilePath(audio)).toBe("/path/to/audio.mp3");
    });

    it("should preserve volume", () => {
      const audio: AudioSettings = createNoAudio(0.7);
      convertToBuiltinAudio(audio, "gentle-bell");
      expect(audio.volume).toBe(0.7);
    });
  });

  describe("convertToFilePathAudio", () => {
    it("should convert none to filePath", () => {
      const audio: AudioSettings = createNoAudio();
      convertToFilePathAudio(audio, "/path/to/audio.mp3");
      expect(getAudioSourceType(audio)).toBe("filePath");
      expect(getAudioFilePath(audio)).toBe("/path/to/audio.mp3");
    });

    it("should convert builtin to filePath", () => {
      const audio: AudioSettings = createBuiltinAudio("gentle-bell");
      convertToFilePathAudio(audio, "/path/to/audio.mp3");
      expect(getAudioSourceType(audio)).toBe("filePath");
      expect(getAudioFilePath(audio)).toBe("/path/to/audio.mp3");
      // Old value is preserved
      expect(getBuiltinAudioName(audio)).toBe("gentle-bell");
    });

    it("should preserve volume", () => {
      const audio: AudioSettings = createNoAudio(0.5);
      convertToFilePathAudio(audio, "/path/to/audio.mp3");
      expect(audio.volume).toBe(0.5);
    });
  });
});

// ============================================================================
// ResolvedBackground Tests
// ============================================================================

describe("ResolvedBackground Factories", () => {
  describe("createResolvedSolidBackground", () => {
    it("should create resolved solid background", () => {
      const bg = createResolvedSolidBackground("#1f2937");
      expect(bg).toEqual({ kind: "solid", value: "#1f2937" });
    });
  });

  describe("createResolvedImageBackground", () => {
    it("should create resolved image background", () => {
      const bg = createResolvedImageBackground("/path/to/image.jpg");
      expect(bg).toEqual({ kind: "image", value: "/path/to/image.jpg" });
    });
  });

  describe("getResolvedBackgroundValue", () => {
    it("should extract value from solid background", () => {
      const bg = createResolvedSolidBackground("#1f2937");
      expect(getResolvedBackgroundValue(bg)).toBe("#1f2937");
    });

    it("should extract value from image background", () => {
      const bg = createResolvedImageBackground("/path/to/image.jpg");
      expect(getResolvedBackgroundValue(bg)).toBe("/path/to/image.jpg");
    });
  });

  describe("getResolvedBackgroundType", () => {
    it("should return 'solid' for solid background", () => {
      const bg = createResolvedSolidBackground("#1f2937");
      expect(getResolvedBackgroundType(bg)).toBe("solid");
    });

    it("should return 'image' for image background", () => {
      const bg = createResolvedImageBackground("/path/to/image.jpg");
      expect(getResolvedBackgroundType(bg)).toBe("image");
    });
  });
});

// ============================================================================
// ThemeSettings Tests
// ============================================================================

describe("ThemeSettings Factories", () => {
  describe("createDefaultTheme", () => {
    it("should create theme with default values", () => {
      const theme = createDefaultTheme();
      expect(theme.background.current).toBe("solid");
      expect(theme.background.solid).toBe("#1f2937");
      expect(theme.textColor).toBe("#f8fafc");
      expect(theme.blurRadius).toBe(8);
      expect(theme.opacity).toBe(0.9);
      expect(theme.fontSize).toBe(24);
      expect(theme.fontFamily).toBe("Arial");
    });

    it("should create theme with custom values", () => {
      const theme = createDefaultTheme({
        backgroundColor: "#000000",
        blurRadius: 10,
        fontFamily: "Helvetica",
        fontSize: 32,
        opacity: 0.8,
        textColor: "#ffffff",
      });
      expect(getSolidColor(theme.background)).toBe("#000000");
      expect(theme.textColor).toBe("#ffffff");
      expect(theme.blurRadius).toBe(10);
      expect(theme.opacity).toBe(0.8);
      expect(theme.fontSize).toBe(32);
      expect(theme.fontFamily).toBe("Helvetica");
    });

    it("should create theme with partial custom values", () => {
      const theme = createDefaultTheme({
        backgroundColor: "#123456",
        fontSize: 28,
      });
      expect(getSolidColor(theme.background)).toBe("#123456");
      expect(theme.fontSize).toBe(28);
      expect(theme.textColor).toBe("#f8fafc"); // default
      expect(theme.blurRadius).toBe(8); // default
    });
  });

  describe("cloneTheme", () => {
    it("should create a shallow copy of theme", () => {
      const original = createDefaultTheme();
      const cloned = cloneTheme(original);
      expect(cloned).toEqual(original);
      expect(cloned).not.toBe(original);
    });

    it("should clone theme with overrides", () => {
      const original = createDefaultTheme();
      const cloned = cloneTheme(original, {
        fontSize: 32,
        textColor: "#000000",
      });
      expect(cloned.textColor).toBe("#000000");
      expect(cloned.fontSize).toBe(32);
      expect(cloned.blurRadius).toBe(original.blurRadius);
      expect(original.textColor).toBe("#f8fafc"); // original unchanged
    });
  });

  describe("setThemeBackground", () => {
    it("should update theme background", () => {
      const theme = createDefaultTheme();
      const newBg = createImagePathBackground("/path/to/image.jpg");
      setThemeBackground(theme, newBg);
      expect(theme.background).toBe(newBg);
    });
  });

  describe("setThemeTextColor", () => {
    it("should update theme text color", () => {
      const theme = createDefaultTheme();
      setThemeTextColor(theme, "#000000");
      expect(theme.textColor).toBe("#000000");
    });
  });

  describe("setThemeBlurRadius", () => {
    it("should update theme blur radius", () => {
      const theme = createDefaultTheme();
      setThemeBlurRadius(theme, 15);
      expect(theme.blurRadius).toBe(15);
    });

    it("should clamp blur radius to valid range", () => {
      const theme = createDefaultTheme();
      setThemeBlurRadius(theme, -5);
      expect(theme.blurRadius).toBe(0);
      setThemeBlurRadius(theme, 100);
      expect(theme.blurRadius).toBe(50);
    });
  });

  describe("setThemeOpacity", () => {
    it("should update theme opacity", () => {
      const theme = createDefaultTheme();
      setThemeOpacity(theme, 0.5);
      expect(theme.opacity).toBe(0.5);
    });

    it("should clamp opacity to valid range", () => {
      const theme = createDefaultTheme();
      setThemeOpacity(theme, -0.5);
      expect(theme.opacity).toBe(0);
      setThemeOpacity(theme, 1.5);
      expect(theme.opacity).toBe(1);
    });
  });

  describe("setThemeFontSize", () => {
    it("should update theme font size", () => {
      const theme = createDefaultTheme();
      setThemeFontSize(theme, 32);
      expect(theme.fontSize).toBe(32);
    });

    it("should clamp font size to valid range", () => {
      const theme = createDefaultTheme();
      setThemeFontSize(theme, 5);
      expect(theme.fontSize).toBe(8);
      setThemeFontSize(theme, 150);
      expect(theme.fontSize).toBe(100);
    });
  });

  describe("setThemeFontFamily", () => {
    it("should update theme font family", () => {
      const theme = createDefaultTheme();
      setThemeFontFamily(theme, "Helvetica");
      expect(theme.fontFamily).toBe("Helvetica");
    });
  });
});

// ============================================================================
// TimeRange Tests
// ============================================================================

describe("TimeRange Factories", () => {
  describe("createTimeRange", () => {
    it("should create a time range with given times", () => {
      const timeRange = createTimeRange("09:00", "18:00");
      expect(timeRange).toEqual({ end: "18:00", start: "09:00" });
    });
  });

  describe("createAllDayTimeRange", () => {
    it("should create an all-day time range", () => {
      const timeRange = createAllDayTimeRange();
      expect(timeRange).toEqual({ end: "00:00", start: "00:00" });
    });
  });

  describe("isAllDayTimeRange", () => {
    it("should return true for all-day range", () => {
      const timeRange = createAllDayTimeRange();
      expect(isAllDayTimeRange(timeRange)).toBe(true);
    });

    it("should return false for non-all-day range", () => {
      const timeRange = createTimeRange("09:00", "18:00");
      expect(isAllDayTimeRange(timeRange)).toBe(false);
    });
  });

  describe("getTimeRangeStart", () => {
    it("should extract start time", () => {
      const timeRange = createTimeRange("09:00", "18:00");
      expect(getTimeRangeStart(timeRange)).toBe("09:00");
    });
  });

  describe("getTimeRangeEnd", () => {
    it("should extract end time", () => {
      const timeRange = createTimeRange("09:00", "18:00");
      expect(getTimeRangeEnd(timeRange)).toBe("18:00");
    });
  });

  describe("setTimeRangeStart", () => {
    it("should update start time", () => {
      const timeRange = createTimeRange("09:00", "18:00");
      setTimeRangeStart(timeRange, "08:00");
      expect(timeRange.start).toBe("08:00");
      expect(timeRange.end).toBe("18:00");
    });
  });

  describe("setTimeRangeEnd", () => {
    it("should update end time", () => {
      const timeRange = createTimeRange("09:00", "18:00");
      setTimeRangeEnd(timeRange, "19:00");
      expect(timeRange.start).toBe("09:00");
      expect(timeRange.end).toBe("19:00");
    });
  });

  describe("cloneTimeRange", () => {
    it("should create a copy of time range", () => {
      const original = createTimeRange("09:00", "18:00");
      const cloned = cloneTimeRange(original);
      expect(cloned).toEqual(original);
      expect(cloned).not.toBe(original);
    });

    it("should not affect original when clone is modified", () => {
      const original = createTimeRange("09:00", "18:00");
      const cloned = cloneTimeRange(original);
      setTimeRangeStart(cloned, "10:00");
      expect(original.start).toBe("09:00");
      expect(cloned.start).toBe("10:00");
    });
  });
});

// ============================================================================
// SuggestionsSettings Tests
// ============================================================================

describe("SuggestionsSettings Factories", () => {
  describe("createSuggestionsSettings", () => {
    it("should create settings with default show=true", () => {
      const settings = createSuggestionsSettings();
      expect(settings).toEqual({ show: true });
    });

    it("should create settings with custom show value", () => {
      const settings = createSuggestionsSettings(false);
      expect(settings).toEqual({ show: false });
    });
  });

  describe("shouldShowSuggestions", () => {
    it("should return true when show is true", () => {
      const settings = createSuggestionsSettings(true);
      expect(shouldShowSuggestions(settings)).toBe(true);
    });

    it("should return false when show is false", () => {
      const settings = createSuggestionsSettings(false);
      expect(shouldShowSuggestions(settings)).toBe(false);
    });
  });

  describe("setShowSuggestions", () => {
    it("should update show value", () => {
      const settings = createSuggestionsSettings(true);
      setShowSuggestions(settings, false);
      expect(settings.show).toBe(false);
    });
  });
});

// ============================================================================
// Generic Accessor Tests
// ============================================================================

describe("Generic Property Accessors", () => {
  describe("getProp", () => {
    it("should get property value from object", () => {
      const theme = createDefaultTheme();
      expect(getProp(theme, "fontSize")).toBe(24);
      expect(getProp(theme, "opacity")).toBe(0.9);
      expect(getProp(theme, "fontFamily")).toBe("Arial");
    });

    it("should work with any object type", () => {
      const timeRange = createTimeRange("09:00", "18:00");
      expect(getProp(timeRange, "start")).toBe("09:00");
      expect(getProp(timeRange, "end")).toBe("18:00");
    });
  });

  describe("setProp", () => {
    it("should set property value on object", () => {
      const theme = createDefaultTheme();
      setProp(theme, "fontSize", 32);
      expect(theme.fontSize).toBe(32);
      setProp(theme, "opacity", 0.5);
      expect(theme.opacity).toBe(0.5);
    });

    it("should work with any object type", () => {
      const timeRange = createTimeRange("09:00", "18:00");
      setProp(timeRange, "start", "10:00");
      expect(timeRange.start).toBe("10:00");
      setProp(timeRange, "end", "19:00");
      expect(timeRange.end).toBe("19:00");
    });

    it("should be type-safe", () => {
      const theme = createDefaultTheme();
      // TypeScript will catch type errors at compile time
      setProp(theme, "fontSize", 32); // OK
      // setProp(theme, "fontSize", "invalid"); // Type error
      // setProp(theme, "nonexistent", 123); // Type error
    });
  });
});
