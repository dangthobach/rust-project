import { Component, For, Show, createResource } from 'solid-js';
import { A, useLocation } from '@solidjs/router';
import { Card, Spinner } from '~/components/ui';
import { api } from '~/lib/api';

const Search: Component = () => {
  const location = useLocation();

  const [results] = createResource(
    () => location.search,
    async (search: string) => {
      const query = new URLSearchParams(search).get('q')?.trim() ?? '';
      if (query.length < 2) return { tasks: [], clients: [] };
      return api.unifiedSearch(query, 12);
    },
  );

  return (
    <div class="font-body">
      <h1 class="mb-2 font-heading text-2xl font-black uppercase tracking-tight">Search</h1>
      <p class="mb-6 font-mono text-sm text-neutral-darkGray">
        Query:{' '}
        <span class="font-bold text-black">
          {new URLSearchParams(location.search).get('q') || '(empty)'}
        </span>
      </p>

      <Show
        when={
          (new URLSearchParams(location.search).get('q')?.trim().length ?? 0) > 0 &&
          (new URLSearchParams(location.search).get('q')?.trim().length ?? 0) < 2
        }
      >
        <p class="text-sm text-neutral-darkGray">Enter at least 2 characters.</p>
      </Show>

      <Show when={results.loading}>
        <div class="flex justify-center py-12">
          <Spinner />
        </div>
      </Show>

      <Show when={results()}>
        <div class="grid gap-6 lg:grid-cols-2">
          <Card class="border-[3px] border-black p-4 shadow-brutal">
            <div class="mb-3 font-heading text-xs font-black uppercase">Tasks</div>
            <Show when={(results()?.tasks.length ?? 0) === 0}>
              <p class="font-mono text-sm text-neutral-darkGray">No task matches.</p>
            </Show>
            <ul class="space-y-2">
              <For each={results()?.tasks ?? []}>
                {(t) => (
                  <li>
                    <A
                      href={`/tasks/${t.id}`}
                      class="flex flex-col border-2 border-black bg-white px-3 py-2 no-underline shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5"
                    >
                      <span class="font-heading font-bold text-black">{t.title}</span>
                      <span class="font-mono text-[10px] uppercase text-neutral-darkGray">{t.status}</span>
                    </A>
                  </li>
                )}
              </For>
            </ul>
          </Card>

          <Card class="border-[3px] border-black p-4 shadow-brutal">
            <div class="mb-3 font-heading text-xs font-black uppercase">Clients</div>
            <Show when={(results()?.clients.length ?? 0) === 0}>
              <p class="font-mono text-sm text-neutral-darkGray">No client matches.</p>
            </Show>
            <ul class="space-y-2">
              <For each={results()?.clients ?? []}>
                {(c) => (
                  <li>
                    <A
                      href={`/clients/${c.id}`}
                      class="flex flex-col border-2 border-black bg-white px-3 py-2 no-underline shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5"
                    >
                      <span class="font-heading font-bold text-black">{c.name}</span>
                      <span class="font-mono text-[10px] text-neutral-darkGray">{c.company || '—'}</span>
                    </A>
                  </li>
                )}
              </For>
            </ul>
          </Card>
        </div>
      </Show>
    </div>
  );
};

export default Search;
