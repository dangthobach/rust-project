import { Component, For, Show, createMemo, createSignal } from 'solid-js';
import { A } from '@solidjs/router';
import { Badge, Button, Card, CardContent, CardHeader, CardTitle, Input, Spinner } from '~/components/ui';
import { useRolesPaged } from '~/lib/hooks';

const RbacRoles: Component = () => {
  const [page, setPage] = createSignal(1);
  const [search, setSearch] = createSignal('');

  const roles = useRolesPaged(() => ({
    page: page(),
    limit: 20,
    search: search().trim() || undefined,
  }));

  const meta = createMemo(() => roles.data);
  const items = createMemo(() => meta()?.items ?? []);

  return (
    <div class="font-body">
      <div class="mb-6 flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 class="font-heading text-2xl font-black uppercase tracking-tight text-shadow-brutal sm:text-heading-1">
            RBAC — Roles
          </h1>
          <p class="mt-1 font-mono text-xs font-semibold uppercase tracking-wide text-neutral-darkGray">
            Paginated list (page/limit/search) — backend `/api/admin/rbac/roles/paged`
          </p>
        </div>
        <div class="flex items-center gap-2">
          <A
            href="/admin/rbac/permissions"
            class="border-[3px] border-black bg-white px-4 py-2 font-heading text-xs font-black uppercase shadow-brutal-sm no-underline hover:-translate-x-0.5 hover:-translate-y-0.5 transition-all"
          >
            Permissions →
          </A>
        </div>
      </div>

      <Card class="border-[3px] border-black bg-white shadow-brutal">
        <CardHeader>
          <CardTitle class="font-mono text-xs uppercase tracking-widest">Search</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="flex flex-wrap items-center gap-3">
            <div class="min-w-[240px] flex-1">
              <Input
                type="text"
                placeholder="Search by slug..."
                value={search()}
                onInput={(e: any) => {
                  setSearch(e.currentTarget.value);
                  setPage(1);
                }}
              />
            </div>
            <Button
              variant="secondary"
              onClick={() => {
                setSearch('');
                setPage(1);
              }}
            >
              Clear
            </Button>
          </div>
        </CardContent>
      </Card>

      <div class="mt-6">
        <Show when={roles.isPending}>
          <div class="flex justify-center p-10">
            <Spinner />
          </div>
        </Show>

        <Show when={roles.isError}>
          <Card class="border-[3px] border-red-600 bg-red-50 p-6">
            <div class="font-heading font-bold uppercase text-red-800">Failed to load roles</div>
            <div class="mt-1 font-mono text-sm text-neutral-darkGray">{String(roles.error?.message ?? '')}</div>
            <Button variant="secondary" size="sm" class="mt-4" onClick={() => roles.refetch()}>
              Retry
            </Button>
          </Card>
        </Show>

        <Show when={roles.data}>
          <div class="overflow-x-auto border-[3px] border-black bg-white shadow-brutal-sm">
            <table class="w-full min-w-[720px] border-collapse font-mono text-xs">
              <thead>
                <tr class="bg-neutral-lightGray">
                  <th class="border-b-[3px] border-black px-3 py-2 text-left font-heading text-[10px] font-black uppercase">
                    Slug
                  </th>
                  <th class="border-b-[3px] border-black px-3 py-2 text-left font-heading text-[10px] font-black uppercase">
                    Description
                  </th>
                  <th class="border-b-[3px] border-black px-3 py-2 text-left font-heading text-[10px] font-black uppercase">
                    Active
                  </th>
                  <th class="border-b-[3px] border-black px-3 py-2 text-left font-heading text-[10px] font-black uppercase">
                    Created
                  </th>
                </tr>
              </thead>
              <tbody>
                <For each={items()}>
                  {(r) => (
                    <tr class="bg-white">
                      <td class="border-b-[3px] border-black px-3 py-3 font-bold">{r.slug}</td>
                      <td class="border-b-[3px] border-black px-3 py-3 text-neutral-darkGray">
                        {r.description ?? ''}
                      </td>
                      <td class="border-b-[3px] border-black px-3 py-3">
                        <Badge variant={r.is_active ? 'success' : 'secondary'}>
                          {r.is_active ? 'Active' : 'Inactive'}
                        </Badge>
                      </td>
                      <td class="border-b-[3px] border-black px-3 py-3 text-neutral-darkGray">{r.created_at}</td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>

          <div class="mt-5 flex flex-wrap items-center justify-between gap-3">
            <div class="font-mono text-[10px] font-semibold uppercase text-neutral-darkGray">
              Total: {meta()?.total ?? 0} · Page {meta()?.page ?? 1} / {meta()?.total_pages ?? 1}
            </div>
            <div class="flex items-center gap-2">
              <Button
                variant="secondary"
                disabled={(meta()?.page ?? 1) <= 1}
                onClick={() => setPage((p) => Math.max(1, p - 1))}
              >
                ← Prev
              </Button>
              <Button
                variant="secondary"
                disabled={(meta()?.page ?? 1) >= (meta()?.total_pages ?? 1)}
                onClick={() => setPage((p) => p + 1)}
              >
                Next →
              </Button>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
};

export default RbacRoles;

