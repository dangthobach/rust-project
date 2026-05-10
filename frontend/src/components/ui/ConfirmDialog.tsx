import { Component, Show } from 'solid-js';
import { Portal } from 'solid-js/web';
import { Button } from './Button';

interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: 'danger' | 'default';
  loading?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export const ConfirmDialog: Component<ConfirmDialogProps> = (props) => {
  return (
    <Portal>
      <Show when={props.open}>
        {/* Backdrop */}
        <div
          class="fixed inset-0 z-50 bg-black/40"
          onClick={props.onCancel}
        />

        {/* Dialog */}
        <div
          class="fixed left-1/2 top-1/2 z-50 w-full max-w-sm -translate-x-1/2 -translate-y-1/2 border-[3px] border-black bg-white p-6 shadow-brutal"
          role="dialog"
          aria-modal="true"
        >
          <h2 class="font-heading text-lg font-black uppercase tracking-tight">
            {props.title}
          </h2>
          <p class="mt-2 font-mono text-sm text-neutral-darkGray">{props.message}</p>

          <div class="mt-6 flex justify-end gap-2">
            <Button
              variant="secondary"
              size="sm"
              disabled={props.loading}
              onClick={props.onCancel}
            >
              {props.cancelLabel ?? 'Huỷ'}
            </Button>
            <Button
              variant={props.variant === 'danger' ? 'danger' : 'primary'}
              size="sm"
              disabled={props.loading}
              onClick={props.onConfirm}
            >
              {props.loading ? '...' : (props.confirmLabel ?? 'Xác nhận')}
            </Button>
          </div>
        </div>
      </Show>
    </Portal>
  );
};
