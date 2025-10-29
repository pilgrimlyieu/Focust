import { ref } from "vue";

/** Toast kind */
export type ToastKind = "success" | "error" | "info";

/** Toast item */
export interface ToastItem {
  id: number;
  kind: ToastKind;
  message: string;
  timeout: number;
}

// Singleton state
const toasts = ref<ToastItem[]>([]);

/** Composable for managing toasts */
export function useToast() {
  /**
   * Show a toast message
   * @param {ToastKind} kind The kind of toast
   * @param {string} message The message to display
   * @param {number} duration Duration in milliseconds before the toast is dismissed
   */
  function show(kind: ToastKind, message: string, duration: number = 3000) {
    const id = Date.now() + Math.random();
    const item: ToastItem = { id, kind, message, timeout: duration };
    toasts.value.push(item);
    setTimeout(() => dismiss(id), duration);
  }

  /**
   * Dismiss a toast by ID
   * @param {number} id The ID of the toast to dismiss
   */
  function dismiss(id: number) {
    toasts.value = toasts.value.filter((toast) => toast.id !== id);
  }

  return {
    dismiss,
    show,
    toasts,
  };
}
