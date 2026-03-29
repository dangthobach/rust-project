import { Component, Show, For } from 'solid-js';
import { Card, CardContent } from '~/components/ui';
import { cn } from '~/theme/utils';

export type TaskTagTone = 'sky' | 'lime' | 'pale' | 'mint';

interface TaskCardProps {
  onClick?: () => void;
  title: string;
  description: string;
  priority: 'low' | 'medium' | 'high' | 'urgent';
  dueDate: string;
  status: 'todo' | 'in_progress' | 'done' | 'cancelled';
  /** Optional tag chips (e.g. DEVELOPMENT, V2.0) */
  tags?: Array<{ label: string; tone: TaskTagTone }>;
}

const tagTone: Record<TaskTagTone, string> = {
  sky: 'bg-ledger-sky text-black',
  lime: 'bg-ledger-lime text-black',
  pale: 'bg-ledger-pale text-black',
  mint: 'bg-ledger-mint text-black',
};

export const TaskCard: Component<TaskCardProps> = (props) => {
  const showAlert = () => props.priority === 'high' || props.priority === 'urgent';
  const isDone = () => props.status === 'done';

  return (
    <Card
      hoverable={!isDone()}
      class={cn(
        'relative overflow-hidden border-3 border-black shadow-brutal-sm transition-all',
        isDone() && 'opacity-[0.88]',
        props.onClick && !isDone() && 'cursor-pointer',
      )}
      onClick={props.onClick}
    >
      <CardContent class="p-4 font-body">
        <div class="mb-3 flex flex-wrap items-start gap-2">
          <For each={props.tags || []}>
            {(t) => (
              <span
                class={cn(
                  'border-2 border-black px-2 py-0.5 font-heading text-[10px] font-bold uppercase tracking-wide shadow-brutal-sm',
                  tagTone[t.tone],
                )}
              >
                {t.label}
              </span>
            )}
          </For>
          <Show when={showAlert()}>
            <span
              class="ml-auto flex h-7 w-7 shrink-0 items-center justify-center border-2 border-black bg-ledger-orange font-heading font-black text-white shadow-brutal-sm"
              title="High priority"
              aria-hidden="true"
            >
              !
            </span>
          </Show>
        </div>

        <h4 class="font-heading text-lg font-black uppercase leading-tight text-black">{props.title}</h4>

        <p class="mt-2 line-clamp-3 text-sm leading-relaxed text-neutral-darkGray">{props.description || '—'}</p>

        <div class="mt-4 flex items-center justify-between gap-2 border-t-3 border-black pt-3">
          <div class="flex items-center gap-2 text-xs font-heading font-bold uppercase text-neutral-darkGray">
            <span aria-hidden="true">📅</span>
            <span>{props.dueDate}</span>
          </div>
          <div class="flex -space-x-2" aria-hidden="true">
            <span class="inline-flex h-8 w-8 items-center justify-center border-2 border-black bg-ledger-sky text-[10px] font-black shadow-brutal-sm">
              A
            </span>
            <span class="inline-flex h-8 w-8 items-center justify-center border-2 border-black bg-ledger-pale text-[10px] font-black shadow-brutal-sm">
              B
            </span>
            <span class="inline-flex h-8 w-8 items-center justify-center border-2 border-black bg-ledger-mint text-[10px] font-black shadow-brutal-sm">
              +
            </span>
          </div>
        </div>
      </CardContent>

      <Show when={isDone()}>
        <div
          class="pointer-events-none absolute inset-0 z-10 flex items-center justify-center bg-white/25"
          aria-hidden="true"
        >
          <span class="-rotate-12 border-4 border-black bg-white/90 px-4 py-2 font-heading text-2xl font-black uppercase tracking-widest text-black shadow-brutal-sm">
            Completed
          </span>
        </div>
      </Show>
    </Card>
  );
};
