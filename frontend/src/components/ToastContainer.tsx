import { Component, For, Show, ErrorBoundary } from 'solid-js';
import { getToasts, removeToast, type Toast } from '~/lib/toast';

const ToastContainer: Component = () => {
  const getToastColor = (type: string) => {
    switch (type) {
      case 'success': return 'bg-green-500 text-white';
      case 'warning': return 'bg-yellow-500 text-black';
      case 'error': return 'bg-red-500 text-white';
      case 'info': return 'bg-blue-500 text-white';
      default: return 'bg-neutral-beige text-black';
    }
  };

  const getToastIcon = (type: string) => {
    switch (type) {
      case 'success': return '✓';
      case 'warning': return '⚠';
      case 'error': return '✕';
      case 'info': return 'ℹ';
      default: return '•';
    }
  };

  return (
    <ErrorBoundary fallback={() => null}>
      <div class="fixed top-4 right-4 z-50 space-y-3 max-w-md pointer-events-none">
        <For each={getToasts() || []}>
          {(toast: Toast) => (
            <div
              class={`p-4 rounded border-3 border-black shadow-brutal ${getToastColor(toast.type)} animate-slide-in pointer-events-auto`}
            >
              <div class="flex items-start justify-between gap-3">
                <div class="flex items-start gap-2 flex-1">
                  <span class="text-lg font-bold">{getToastIcon(toast.type)}</span>
                  <div class="flex-1">
                    <h4 class="font-bold text-sm">{toast.title}</h4>
                    <Show when={toast.message}>
                      <p class="text-sm mt-1 opacity-90">{toast.message}</p>
                    </Show>
                  </div>
                </div>
                <button
                  onClick={() => removeToast(toast.id)}
                  class="flex-shrink-0 text-lg font-bold hover:scale-110 transition-transform opacity-70 hover:opacity-100"
                  aria-label="Close"
                >
                  ✕
                </button>
              </div>
            </div>
          )}
        </For>
      </div>
    </ErrorBoundary>
  );
};

export default ToastContainer;
