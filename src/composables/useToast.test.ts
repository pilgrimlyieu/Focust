import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useToast } from "./useToast";

describe("useToast", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    // Clear toasts before each test (singleton pattern requires manual cleanup)
    const { toasts } = useToast();
    toasts.value = [];
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("initialization", () => {
    it("should start with empty toasts", () => {
      const { toasts } = useToast();
      expect(toasts.value).toEqual([]);
    });

    it("should provide show and dismiss functions", () => {
      const toast = useToast();
      expect(toast.show).toBeDefined();
      expect(toast.dismiss).toBeDefined();
      expect(typeof toast.show).toBe("function");
      expect(typeof toast.dismiss).toBe("function");
    });
  });

  describe("show", () => {
    it("should add a toast with success kind", () => {
      const { toasts, show } = useToast();
      show("success", "Operation completed");

      expect(toasts.value).toHaveLength(1);
      expect(toasts.value[0].kind).toBe("success");
      expect(toasts.value[0].message).toBe("Operation completed");
      expect(toasts.value[0].timeout).toBe(3000); // default duration
    });

    it("should add a toast with error kind", () => {
      const { toasts, show } = useToast();
      show("error", "An error occurred");

      expect(toasts.value).toHaveLength(1);
      expect(toasts.value[0].kind).toBe("error");
      expect(toasts.value[0].message).toBe("An error occurred");
    });

    it("should add a toast with info kind", () => {
      const { toasts, show } = useToast();
      show("info", "Here is some information");

      expect(toasts.value).toHaveLength(1);
      expect(toasts.value[0].kind).toBe("info");
      expect(toasts.value[0].message).toBe("Here is some information");
    });

    it("should respect custom duration", () => {
      const { toasts, show } = useToast();
      show("success", "Custom timeout", 5000);

      expect(toasts.value[0].timeout).toBe(5000);
    });

    it("should generate unique IDs for toasts", () => {
      const { toasts, show } = useToast();
      show("success", "First toast");
      show("info", "Second toast");

      expect(toasts.value).toHaveLength(2);
      expect(toasts.value[0].id).not.toBe(toasts.value[1].id);
    });

    it("should add multiple toasts", () => {
      const { toasts, show } = useToast();
      show("success", "First");
      show("error", "Second");
      show("info", "Third");

      expect(toasts.value).toHaveLength(3);
      expect(toasts.value[0].message).toBe("First");
      expect(toasts.value[1].message).toBe("Second");
      expect(toasts.value[2].message).toBe("Third");
    });
  });

  describe("dismiss", () => {
    it("should remove a toast by ID", () => {
      const { toasts, show, dismiss } = useToast();
      show("success", "Toast 1");
      show("info", "Toast 2");

      const firstId = toasts.value[0].id;
      dismiss(firstId);

      expect(toasts.value).toHaveLength(1);
      expect(toasts.value[0].message).toBe("Toast 2");
    });

    it("should do nothing when dismissing non-existent ID", () => {
      const { toasts, show, dismiss } = useToast();
      show("success", "Only toast");

      dismiss(99999);

      expect(toasts.value).toHaveLength(1);
      expect(toasts.value[0].message).toBe("Only toast");
    });

    it("should remove all toasts when dismissing each one", () => {
      const { toasts, show, dismiss } = useToast();
      show("success", "Toast 1");
      show("info", "Toast 2");
      show("error", "Toast 3");

      const ids = toasts.value.map((t) => t.id);
      ids.forEach((id) => void dismiss(id));

      expect(toasts.value).toHaveLength(0);
    });
  });

  describe("auto-dismiss", () => {
    it("should auto-dismiss toast after default duration", () => {
      const { toasts, show } = useToast();
      show("success", "Auto-dismiss test");

      expect(toasts.value).toHaveLength(1);

      // Fast-forward time by 3000ms (default duration)
      vi.advanceTimersByTime(3000);

      expect(toasts.value).toHaveLength(0);
    });

    it("should auto-dismiss toast after custom duration", () => {
      const { toasts, show } = useToast();
      show("success", "Custom auto-dismiss", 2000);

      expect(toasts.value).toHaveLength(1);

      // Should still be there after 1900ms
      vi.advanceTimersByTime(1900);
      expect(toasts.value).toHaveLength(1);

      // Should be dismissed after 2000ms
      vi.advanceTimersByTime(100);
      expect(toasts.value).toHaveLength(0);
    });

    it("should auto-dismiss multiple toasts at different times", () => {
      const { toasts, show } = useToast();
      show("success", "Toast 1", 1000);
      show("info", "Toast 2", 2000);
      show("error", "Toast 3", 3000);

      expect(toasts.value).toHaveLength(3);

      // After 1000ms, first toast dismissed
      vi.advanceTimersByTime(1000);
      expect(toasts.value).toHaveLength(2);
      expect(toasts.value[0].message).toBe("Toast 2");

      // After another 1000ms (2000ms total), second toast dismissed
      vi.advanceTimersByTime(1000);
      expect(toasts.value).toHaveLength(1);
      expect(toasts.value[0].message).toBe("Toast 3");

      // After another 1000ms (3000ms total), third toast dismissed
      vi.advanceTimersByTime(1000);
      expect(toasts.value).toHaveLength(0);
    });
  });

  describe("edge cases", () => {
    it("should handle empty message", () => {
      const { toasts, show } = useToast();
      show("success", "");

      expect(toasts.value).toHaveLength(1);
      expect(toasts.value[0].message).toBe("");
    });

    it("should handle very long messages", () => {
      const { toasts, show } = useToast();
      const longMessage = "A".repeat(1000);
      show("info", longMessage);

      expect(toasts.value).toHaveLength(1);
      expect(toasts.value[0].message).toBe(longMessage);
    });

    it("should handle zero duration (immediate dismiss)", () => {
      const { toasts, show } = useToast();
      show("success", "Zero duration", 0);

      expect(toasts.value).toHaveLength(1);

      // Immediately dismiss
      vi.advanceTimersByTime(0);
      expect(toasts.value).toHaveLength(0);
    });

    it("should handle very large duration", () => {
      const { toasts, show } = useToast();
      show("success", "Very long toast", 999999);

      expect(toasts.value).toHaveLength(1);
      expect(toasts.value[0].timeout).toBe(999999);

      // Should still be there after 10 seconds
      vi.advanceTimersByTime(10000);
      expect(toasts.value).toHaveLength(1);
    });
  });

  describe("reactive behavior", () => {
    it("should be reactive when adding toasts", () => {
      const { toasts, show } = useToast();
      const initialLength = toasts.value.length;

      show("success", "Reactive test");

      expect(toasts.value.length).toBe(initialLength + 1);
    });

    it("should be reactive when dismissing toasts", () => {
      const { toasts, show, dismiss } = useToast();
      show("success", "Toast 1");
      show("info", "Toast 2");

      const firstId = toasts.value[0].id;
      const initialLength = toasts.value.length;

      dismiss(firstId);

      expect(toasts.value.length).toBe(initialLength - 1);
    });

    it("should maintain reactivity across multiple operations", () => {
      const { toasts, show, dismiss } = useToast();

      show("success", "First");
      expect(toasts.value.length).toBe(1);

      show("error", "Second");
      expect(toasts.value.length).toBe(2);

      const firstId = toasts.value[0].id;
      dismiss(firstId);
      expect(toasts.value.length).toBe(1);

      vi.advanceTimersByTime(3000);
      expect(toasts.value.length).toBe(0);
    });
  });

  describe("concurrent usage", () => {
    it("should handle rapid successive shows", () => {
      const { toasts, show } = useToast();

      for (let i = 0; i < 10; i++) {
        show("success", `Toast ${i}`);
      }

      expect(toasts.value).toHaveLength(10);
      expect(toasts.value[0].message).toBe("Toast 0");
      expect(toasts.value[9].message).toBe("Toast 9");
    });

    it("should handle showing and dismissing simultaneously", () => {
      const { toasts, show, dismiss } = useToast();
      show("success", "Toast 1", 1000);
      show("info", "Toast 2", 2000);

      const firstId = toasts.value[0].id;

      // Manually dismiss first toast
      dismiss(firstId);
      expect(toasts.value).toHaveLength(1);

      // Second toast should auto-dismiss after its duration
      vi.advanceTimersByTime(2000);
      expect(toasts.value).toHaveLength(0);
    });

    it("should not interfere with already dismissed toasts", () => {
      const { toasts, show, dismiss } = useToast();
      show("success", "Toast 1", 5000);

      const toastId = toasts.value[0].id;

      // Manually dismiss
      dismiss(toastId);
      expect(toasts.value).toHaveLength(0);

      // Auto-dismiss timer should not cause issues
      vi.advanceTimersByTime(5000);
      expect(toasts.value).toHaveLength(0);
    });
  });
});
