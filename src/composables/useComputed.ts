/**
 * Composables for creating common computed property patterns
 */

import type { WritableComputedRef } from "vue";
import { computed } from "vue";

/**
 * Create a computed property that gets/sets a nested property
 * @template T - The object type
 * @template K - The key type
 * @param {() => T} getter - Function to get the object
 * @param {K} key - The property key to access
 * @returns {WritableComputedRef<T[K]>} A writable computed ref
 *
 * @example
 * const fontSize = useComputedProp(() => props.theme, 'fontSize');
 * // Now fontSize.value can be read/written directly
 */
export function useComputedProp<T extends object, K extends keyof T>(
  getter: () => T,
  key: K,
): WritableComputedRef<T[K]> {
  return computed({
    get: () => getter()[key],
    set: (value: T[K]) => {
      getter()[key] = value;
    },
  });
}

/**
 * Create a computed property with custom transformation
 * @template T - The source value type
 * @template R - The result value type
 * @param {() => T} getter - Function to get the source value
 * @param {(value: T) => R} transform - Transform source to result
 * @param {(value: R) => T} reverse - Transform result back to source
 * @param {(source: T, result: T) => void} setter - Function to set the source value
 * @returns {WritableComputedRef<R>} A writable computed ref
 *
 * @example
 * // Convert seconds to minutes
 * const minutes = useComputedTransform(
 *   () => props.config.intervalS,
 *   (seconds) => Math.round(seconds / 60),
 *   (minutes) => minutes * 60,
 *   (newValue) => { props.config.intervalS = newValue; }
 * );
 */
export function useComputedTransform<T, R>(
  getter: () => T,
  transform: (value: T) => R,
  reverse: (value: R) => T,
  setter: (value: T) => void,
): WritableComputedRef<R> {
  return computed({
    get: () => transform(getter()),
    set: (value: R) => setter(reverse(value)),
  });
}

/**
 * Create a computed property for seconds <-> minutes conversion
 * @param {() => number} getter - Function to get seconds value
 * @param {(value: number) => void} setter - Function to set seconds value
 * @param {number} [min=1] - Minimum value in minutes
 * @returns {WritableComputedRef<number>} Minutes as computed ref
 *
 * @example
 * const intervalMinutes = useSecondsToMinutes(
 *   () => props.schedule.intervalS,
 *   (value) => { props.schedule.intervalS = value; }
 * );
 */
export function useSecondsToMinutes(
  getter: () => number,
  setter: (value: number) => void,
  min: number = 1,
): WritableComputedRef<number> {
  return useComputedTransform(
    getter,
    (seconds) => Math.round(seconds / 60),
    (minutes) => Math.max(min, Math.round(minutes)) * 60,
    setter,
  );
}

/**
 * Create a computed property for decimal <-> percentage conversion
 * @param {() => number} getter - Function to get decimal value (0.0-1.0)
 * @param {(value: number) => void} setter - Function to set decimal value
 * @param {number} [min=0] - Minimum percentage value
 * @param {number} [max=100] - Maximum percentage value
 * @returns {WritableComputedRef<number>} Percentage as computed ref
 *
 * @example
 * const sizePercent = useDecimalToPercent(
 *   () => props.config.windowSize,
 *   (value) => { props.config.windowSize = value; },
 *   10, 100
 * );
 */
export function useDecimalToPercent(
  getter: () => number,
  setter: (value: number) => void,
  min: number = 0,
  max: number = 100,
): WritableComputedRef<number> {
  return useComputedTransform(
    getter,
    (decimal) => Math.round(decimal * 100),
    (percent) => Math.max(min, Math.min(max, percent)) / 100,
    setter,
  );
}

/**
 * Create a computed property with validation/clamping
 * @template T
 * @param {() => T} getter - Function to get value
 * @param {(value: T) => void} setter - Function to set value
 * @param {(value: T) => T} validate - Function to validate/clamp value
 * @returns {WritableComputedRef<T>} A writable computed ref with validation
 *
 * @example
 * const inactivity = useComputedValidated(
 *   () => props.config.inactiveS,
 *   (value) => { props.config.inactiveS = value; },
 *   (value) => Math.max(30, Math.round(value))
 * );
 */
export function useComputedValidated<T>(
  getter: () => T,
  setter: (value: T) => void,
  validate: (value: T) => T,
): WritableComputedRef<T> {
  return computed({
    get: () => getter(),
    set: (value: T) => setter(validate(value)),
  });
}

/**
 * Create multiple computed properties from an object's properties
 * @template T - The object type
 * @param {() => T} getter - Function to get the object
 * @param {(keyof T)[]} keys - Array of property keys
 * @returns {Record<string, WritableComputedRef<any>>} Map of computed refs
 *
 * @example
 * const { fontSize, opacity, blurRadius } = useComputedProps(
 *   () => props.theme,
 *   ['fontSize', 'opacity', 'blurRadius']
 * );
 */
export function useComputedProps<T extends object>(
  getter: () => T,
  keys: (keyof T)[],
): Record<string, WritableComputedRef<T[keyof T]>> {
  const result: Record<string, WritableComputedRef<T[keyof T]>> = {};
  for (const key of keys) {
    result[String(key)] = useComputedProp(getter, key);
  }
  return result;
}
