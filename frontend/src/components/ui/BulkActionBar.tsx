import { Component, JSX, Show } from 'solid-js';
import { Portal } from 'solid-js/web';

interface BulkActionBarProps {
  selectedCount: number;
  actions: JSX.Element;
  onClearSelection: () => void;
}

/**
 * Floating bar that appears at the bottom of the screen when rows are selected.
 * Rendered in a Portal so it overlays regardless of scroll position.
 */
export const BulkActionBar: Component<BulkActionBarProps> = (props) => {
  return (
    <Portal>
      <Show when={props.selectedCount > 0}>
        <div class="fixed bottom-6 left-1/2 z-40 flex -translate-x-1/2 items-center gap-3 border-[3px] border-black bg-white px-5 py-3 shadow-brutal">
          <span class="font-heading text-xs font-black uppercase tracking-wide">
            {props.selectedCount} đã chọn
          </span>
          <div class="h-4 w-px bg-black/20" />
          {props.actions}
          <button
            class="ml-2 font-mono text-[10px] uppercase underline decoration-dotted underline-offset-2 hover:text-red-600"
            onClick={props.onClearSelection}
          >
            Bỏ chọn
          </button>
        </div>
      </Show>
    </Portal>
  );
};
