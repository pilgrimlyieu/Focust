import { describe, expect, it } from "vitest";

describe("Test Framework Setup", () => {
  it("should run basic test", () => {
    expect(1 + 1).toBe(2);
  });

  it("should support async tests", async () => {
    const result = await Promise.resolve(42);
    expect(result).toBe(42);
  });

  it("should have access to global test APIs", () => {
    expect(describe).toBeDefined();
    expect(it).toBeDefined();
    expect(expect).toBeDefined();
  });
});
