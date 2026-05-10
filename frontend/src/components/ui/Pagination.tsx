import { Component, For, createMemo, Show } from 'solid-js';
import { Button } from './Button';

interface PaginationProps {
  page: number;
  totalPages: number;
  total: number;
  limit: number;
  onPageChange: (page: number) => void;
  class?: string;
}

export const Pagination: Component<PaginationProps> = (props) => {
  const from = createMemo(() => (props.page - 1) * props.limit + 1);
  const to = createMemo(() => Math.min(props.page * props.limit, props.total));

  const visiblePages = createMemo(() => {
    const total = props.totalPages;
    const cur = props.page;
    if (total <= 7) return Array.from({ length: total }, (_, i) => i + 1);
    const pages: (number | null)[] = [];
    pages.push(1);
    if (cur > 3) pages.push(null);
    for (let i = Math.max(2, cur - 1); i <= Math.min(total - 1, cur + 1); i++) pages.push(i);
    if (cur < total - 2) pages.push(null);
    pages.push(total);
    return pages;
  });

  return (
    <div class={`flex flex-wrap items-center justify-between gap-3 ${props.class ?? ''}`}>
      <span class="font-mono text-[10px] font-semibold uppercase text-neutral-darkGray">
        {from()}–{to()} / {props.total}
      </span>

      <div class="flex items-center gap-1">
        <Button
          variant="secondary"
          size="xs"
          disabled={props.page <= 1}
          onClick={() => props.onPageChange(props.page - 1)}
        >
          ←
        </Button>

        <For each={visiblePages()}>
          {(p) => (
            <Show
              when={p !== null}
              fallback={
                <span class="px-1 font-mono text-xs text-neutral-darkGray">…</span>
              }
            >
              <button
                class={`min-w-[28px] border-[2px] border-black px-2 py-1 font-heading text-[10px] font-black uppercase transition-all ${
                  p === props.page
                    ? 'bg-black text-white'
                    : 'bg-white hover:-translate-x-0.5 hover:-translate-y-0.5 hover:shadow-brutal-sm'
                }`}
                onClick={() => typeof p === 'number' && props.onPageChange(p)}
              >
                {p}
              </button>
            </Show>
          )}
        </For>

        <Button
          variant="secondary"
          size="xs"
          disabled={props.page >= props.totalPages}
          onClick={() => props.onPageChange(props.page + 1)}
        >
          →
        </Button>
      </div>
    </div>
  );
};
