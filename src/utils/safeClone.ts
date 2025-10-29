/**
 * Safely clone an object using structuredClone with JSON fallback
 * @param obj Object to clone
 * @returns Deep cloned copy of the object
 */
export function safeClone<T>(obj: T): T {
  try {
    return structuredClone(obj);
  } catch {
    // Fallback to JSON clone if structuredClone fails
    // This handles cases where the object contains non-cloneable properties
    return JSON.parse(JSON.stringify(obj));
  }
}
