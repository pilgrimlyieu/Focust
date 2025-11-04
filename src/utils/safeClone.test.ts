import { describe, expect, it } from "vitest";
import { safeClone } from "./safeClone";

describe("safeClone", () => {
  it("should clone simple objects", () => {
    const obj = { a: 1, b: "test", c: true };
    const cloned = safeClone(obj);

    expect(cloned).toEqual(obj);
    expect(cloned).not.toBe(obj);
  });

  it("should clone nested objects", () => {
    const obj = {
      a: 1,
      b: {
        c: 2,
        d: {
          e: 3,
        },
      },
    };
    const cloned = safeClone(obj);

    expect(cloned).toEqual(obj);
    expect(cloned).not.toBe(obj);
    expect(cloned.b).not.toBe(obj.b);
    expect(cloned.b.d).not.toBe(obj.b.d);
  });

  it("should clone arrays", () => {
    const arr = [1, 2, 3, { a: 4 }];
    const cloned = safeClone(arr);

    expect(cloned).toEqual(arr);
    expect(cloned).not.toBe(arr);
    expect(cloned[3]).not.toBe(arr[3]);
  });

  it("should handle Date objects", () => {
    const date = new Date("2024-01-01");
    const obj = { date };
    const cloned = safeClone(obj);

    expect(cloned.date).toEqual(date);
    expect(cloned.date).not.toBe(date);
  });

  it("should handle null and undefined", () => {
    const obj = { a: null, b: undefined };
    const cloned = safeClone(obj);

    expect(cloned).toEqual({ a: null });
    expect(cloned.b).toBeUndefined();
  });

  it("should fallback to JSON clone when structuredClone fails", () => {
    // Create an object that structuredClone cannot handle
    const objWithFunction = {
      a: 1,
      b: "test",
      fn: () => "hello", // Functions cannot be cloned by structuredClone
    };

    const cloned = safeClone(objWithFunction);

    // JSON clone will omit functions
    expect(cloned).toEqual({ a: 1, b: "test" });
    expect(cloned).not.toHaveProperty("fn");
  });

  it("should handle circular references with structuredClone", () => {
    // Note: structuredClone CAN handle circular references in modern browsers
    // But JSON.stringify cannot, so if structuredClone fails for some reason,
    // the JSON fallback will throw
    type CircularObj = { a: number; self?: CircularObj };
    const obj: CircularObj = { a: 1 };
    obj.self = obj; // Circular reference

    // This should work with structuredClone
    const cloned = safeClone(obj);
    expect(cloned).toHaveProperty("a", 1);
    expect(cloned).toHaveProperty("self");
    expect(cloned.self).toBe(cloned); // Circular reference preserved
  });

  it("should clone config-like objects", () => {
    const config = {
      language: "en",
      schedules: [
        {
          enabled: true,
          longBreaks: { afterMiniBreaks: 4, id: 2 },
          miniBreaks: { id: 1, intervalS: 1200 },
          name: "Work",
        },
      ],
      themeMode: "dark",
    };

    const cloned = safeClone(config);

    expect(cloned).toEqual(config);
    expect(cloned).not.toBe(config);
    expect(cloned.schedules).not.toBe(config.schedules);
    expect(cloned.schedules[0]).not.toBe(config.schedules[0]);
  });
});
