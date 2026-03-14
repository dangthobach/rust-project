import { Component, createSignal, Show } from 'solid-js';
import { Button, Badge } from '~/components/ui';

interface ExportButtonProps {
  onExport: (format: 'csv' | 'json' | 'pdf') => void;
  isExporting?: boolean;
  label?: string;
}

const ExportButton: Component<ExportButtonProps> = (props) => {
  const [showMenu, setShowMenu] = createSignal(false);

  const handleExport = (format: 'csv' | 'json' | 'pdf') => {
    props.onExport(format);
    setShowMenu(false);
  };

  return (
    <div class="relative">
      <Button
        variant="secondary"
        size="md"
        onClick={() => setShowMenu(!showMenu())}
        disabled={props.isExporting}
        class="flex items-center gap-2"
      >
        <span class="text-xl">📥</span>
        <span class="font-bold">{props.label || 'Export'}</span>
        <Show when={props.isExporting}>
          <Badge variant="warning" class="animate-pulse">
            Processing...
          </Badge>
        </Show>
      </Button>

      <Show when={showMenu()}>
        <div class="absolute right-0 mt-2 w-48 bg-white border-4 border-black shadow-brutal-lg z-50">
          <div class="p-2">
            <button
              class="w-full text-left px-4 py-3 hover:bg-primary border-3 border-transparent hover:border-black transition-all font-bold uppercase text-sm flex items-center gap-2"
              onClick={() => handleExport('csv')}
            >
              <span class="text-xl">📊</span>
              CSV Format
            </button>
            <button
              class="w-full text-left px-4 py-3 hover:bg-secondary border-3 border-transparent hover:border-black transition-all font-bold uppercase text-sm flex items-center gap-2"
              onClick={() => handleExport('json')}
            >
              <span class="text-xl">📄</span>
              JSON Format
            </button>
            <button
              class="w-full text-left px-4 py-3 hover:bg-accent-yellow border-3 border-transparent hover:border-black transition-all font-bold uppercase text-sm flex items-center gap-2"
              onClick={() => handleExport('pdf')}
            >
              <span class="text-xl">📑</span>
              PDF Format
            </button>
          </div>
        </div>
      </Show>

      {/* Backdrop to close menu */}
      <Show when={showMenu()}>
        <div
          class="fixed inset-0 z-40"
          onClick={() => setShowMenu(false)}
        />
      </Show>
    </div>
  );
};

export default ExportButton;
