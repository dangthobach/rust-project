import { Component, For, Show, createMemo, createSignal } from 'solid-js';
import { A } from '@solidjs/router';
import { Badge, Button, Card, CardContent, CardHeader, CardTitle, Input, Spinner } from '~/components/ui';
import { usePermissionsPaged } from '~/lib/hooks';

const RbacPermissions: Component = () => {
  const [page, setPage] = createSignal(1);
  const [search, setSearch] = createSignal('');

  const perms = usePermissionsPaged(() => ({
    page: page(),
    limit: 20,
    search: search().trim() || undefined,
  }));

  const meta = createMemo(() => perms.data);
  const items = createMemo(() => meta()?.items ?? []);

  return (
    <div class="font-body">
      <div class="mb-6 flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 class="font-heading text-2xl font-black uppercase tracking-tight text-shadow-brutal sm:text-heading-1">
            RBAC — Permissions
          </h1>
          <p class="mt-1 font-mono text-xs font-semibold uppercase tracking-wide text-neutral-darkGray">
            Paginated list (page/limit/search) — backend `/api/admin/rbac/permissions/paged`
          </p>
        </div>
        <div class="flex items-center gap-2">
          <A
            href="/admin/rbac/roles"
            class="border-[3px] border-black bg-white px-4 py-2 font-heading text-xs font-black uppercase shadow-brutal-sm no-underline hover:-translate-x-0.5 hover:-translate-y-0.5 transition-all"
          >
            ← Roles
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
                placeholder="Search by code..."
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
        <Show when={perms.isPending}>
          <div class="flex justify-center p-10">
            <Spinner />
          </div>
        </Show>

        <Show when={perms.isError}>
          <Card class="border-[3px] border-red-600 bg-red-50 p-6">
            <div class="font-heading font-bold uppercase text-red-800">Failed to load permissions</div>
            <div class="mt-1 font-mono text-sm text-neutral-darkGray">{String(perms.error?.message ?? '')}</div>
            <Button variant="secondary" size="sm" class="mt-4" onClick={() => perms.refetch()}>
              Retry
            </Button>
          </Card>
        </Show>

        <Show when={perms.data}>
          <div class="overflow-x-auto border-[3px] border-black bg-white shadow-brutal-sm">
            <table class="w-full min-w-[820px] border-collapse font-mono text-xs">
              <thead>
                <tr class="bg-neutral-lightGray">
                  <th class="border-b-[3px] border-black px-3 py-2 text-left font-heading text-[10px] font-black uppercase">
                    Code
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
                  {(p) => (
                    <tr class="bg-white">
                      <td class="border-b-[3px] border-black px-3 py-3 font-bold">{p.code}</td>
                      <td class="border-b-[3px] border-black px-3 py-3 text-neutral-darkGray">
                        {p.description ?? ''}
                      </td>
                      <td class="border-b-[3px] border-black px-3 py-3">
                        <Badge variant={p.is_active ? 'success' : 'secondary'}>
                          {p.is_active ? 'Active' : 'Inactive'}
                        </Badge>
                      </td>
                      <td class="border-b-[3px] border-black px-3 py-3 text-neutral-darkGray">{p.created_at}</td>
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

export default RbacPermissions;

