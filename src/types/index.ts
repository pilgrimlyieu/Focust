/**
 * Central export point for type-related utilities
 *
 * This module re-exports:
 * - Generated types from ./generated/*
 * - Factory functions from ./factories
 * - Type guards from ./guards
 */

// Factory functions
export * from "./factories";
// Commonly used generated types
export type { AppConfig } from "./generated/AppConfig";
export type { AttentionSettings } from "./generated/AttentionSettings";
export type { AudioSettings } from "./generated/AudioSettings";
export type { BackgroundSource } from "./generated/BackgroundSource";
export type { BreakKind } from "./generated/BreakKind";
export type { BreakPayload } from "./generated/BreakPayload";
export type { EventKind } from "./generated/EventKind";
export type { FontFamily } from "./generated/FontFamily";
export type { HexColor } from "./generated/HexColor";
export type { LongBreakSettings } from "./generated/LongBreakSettings";
export type { MiniBreakSettings } from "./generated/MiniBreakSettings";
export type { ResolvedBackground } from "./generated/ResolvedBackground";
export type { SchedulerStatus } from "./generated/SchedulerStatus";
export type { ScheduleSettings } from "./generated/ScheduleSettings";
export type { SuggestionsConfig } from "./generated/SuggestionsConfig";
export type { SuggestionsSettings } from "./generated/SuggestionsSettings";
export type { ThemeSettings } from "./generated/ThemeSettings";
// Type guards
export * from "./guards";
