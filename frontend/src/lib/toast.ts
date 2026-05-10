import { createSignal } from 'solid-js';

export interface Toast {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title?: string;
  message: string;
}

const [toasts, setToasts] = createSignal<Toast[]>([]);

export const getToasts = toasts;

export const showToast = (type: Toast['type'], titleOrMessage: string, message?: string) => {
  const id = Math.random().toString(36).substring(2, 9);
  
  const title = message ? titleOrMessage : undefined;
  const actualMessage = message || titleOrMessage;

  setToasts((prev) => [...prev, { id, type, title, message: actualMessage }]);
  
  setTimeout(() => {
    removeToast(id);
  }, 5000);
};

export const removeToast = (id: string) => {
  setToasts((prev) => prev.filter((t) => t.id !== id));
};

export const toast = {
  success: (message: string, title?: string) => showToast('success', title ?? message, title ? message : undefined),
  error: (message: string, title?: string) => showToast('error', title ?? message, title ? message : undefined),
  info: (message: string, title?: string) => showToast('info', title ?? message, title ? message : undefined),
  warning: (message: string, title?: string) => showToast('warning', title ?? message, title ? message : undefined),
};
