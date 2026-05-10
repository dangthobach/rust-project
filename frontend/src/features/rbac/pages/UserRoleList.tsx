import { Component, Show, createSignal, createMemo, For } from 'solid-js';
import {
  Badge, Button, Card, CardContent, CardHeader, CardTitle,
  DataTable, Pagination, PageHeader, BulkActionBar, ConfirmDialog,
  Input,
} from '~/components/ui';
import type { ColumnDef } from '~/components/ui';
import { useListState } from '~/lib/hooks/useListState';
import { useUsersWithRoles, useAssignUserRole, useBulkAssignUserRole } from '../hooks';
import { useRolesPaged } from '../hooks';
import type { UserWithRoleDto } from '~/lib/api';
import { toast } from '~/lib/toast';

/**
 * User-Role Assignment page.
 * - Search user list (URL state preserved on back navigation)
 * - Per-row role selector (dropdown)
 * - Multi-select users → bulk assign role
 */
const UserRoleList: Component = () => {
  const { state, update, reset, page, limit } = useListState({
    page: '1',
    limit: '20',
    search: '',
  });

  const [selectedIds, setSelectedIds] = createSignal<Set<string>>(new Set());
  const [bulkRoleSlug, setBulkRoleSlug] = createSignal('');
  const [confirmBulk, setConfirmBulk] = createSignal(false);

  const users = useUsersWithRoles(() => ({
    page: page(),
    limit: limit(),
    search: state().search.trim() || undefined,
  }));

  const allRoles = useRolesPaged(() => ({ page: 1, limit: 100 }));
  const assignRole = useAssignUserRole();
  const bulkAssign = useBulkAssignUserRole();

  const meta = createMemo(() => users.data);
  const items = createMemo(() => meta()?.items ?? []);
  const roleOptions = createMemo(() => allRoles.data?.items ?? []);

  function handleSingleAssign(userId: string, roleSlug: string) {
    assignRole.mutate(
      { userId, roleSlug },
      {
        onSuccess: () => toast.success('Đã cập nhật role'),
        onError: (err: any) => toast.error(err.message ?? 'Cập nhật thất bại'),
      },
    );
  }

  function handleBulkAssign() {
    if (!bulkRoleSlug()) return;
    bulkAssign.mutate(
      { userIds: [...selectedIds()], roleSlug: bulkRoleSlug() },
      {
        onSuccess: () => {
          toast.success(`Đã gán role "${bulkRoleSlug()}" cho ${selectedIds().size} người dùng`);
          setSelectedIds(new Set());
          setBulkRoleSlug('');
          setConfirmBulk(false);
        },
        onError: (err: any) => toast.error(err.message ?? 'Gán role thất bại'),
      },
    );
  }

  const columns: ColumnDef<UserWithRoleDto>[] = [
    {
      key: 'email',
      header: 'Email',
      render: (u) => <span class="font-bold">{u.email}</span>,
    },
    {
      key: 'full_name',
      header: 'Họ tên',
      width: 'w-40',
      render: (u) => <span>{u.full_name}</span>,
    },
    {
      key: 'current_roles',
      header: 'Role hiện tại',
      width: 'w-48',
      render: (u) => (
        <div class="flex flex-wrap gap-1">
          <Show
            when={u.roles.length > 0}
            fallback={<span class="text-neutral-darkGray">—</span>}
          >
            <For each={u.roles}>
              {(r) => <Badge variant="secondary">{r.slug}</Badge>}
            </For>
          </Show>
        </div>
      ),
    },
    {
      key: 'status',
      header: 'Trạng thái',
      width: 'w-24',
      render: (u) => (
        <Badge variant={u.is_active ? 'success' : 'secondary'}>
          {u.is_active ? 'Active' : 'Inactive'}
        </Badge>
      ),
    },
    {
      key: 'assign',
      header: 'Gán role',
      width: 'w-44',
      render: (u) => (
        <select
          class="border-[2px] border-black bg-white px-2 py-1 font-mono text-xs focus:outline-none"
          value={u.roles[0]?.slug ?? ''}
          onChange={(e: any) => {
            const val = e.currentTarget.value;
            if (val) handleSingleAssign(u.id, val);
          }}
        >
          <option value="">-- Chọn role --</option>
          <For each={roleOptions()}>
            {(r) => (
              <option value={r.slug} selected={u.roles.some((ur) => ur.slug === r.slug)}>
                {r.slug}
              </option>
            )}
          </For>
        </select>
      ),
    },
  ];

  return (
    <>
      <PageHeader
        title="Gán Role cho User"
        description="Tìm kiếm người dùng và cấu hình role"
        breadcrumbs={[
          { label: 'Admin' },
          { label: 'RBAC' },
          { label: 'User-Role' },
        ]}
        actions={
          <Button variant="secondary" size="sm" onClick={() => window.history.back()}>
            ← Quay lại
          </Button>
        }
      />

      {/* Search */}
      <Card class="mb-6 border-[3px] border-black bg-white shadow-brutal-sm">
        <CardHeader>
          <CardTitle class="font-mono text-xs uppercase tracking-widest">Tìm kiếm người dùng</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="flex flex-wrap items-center gap-3">
            <div class="min-w-[260px] flex-1">
              <Input
                type="text"
                placeholder="Tìm theo email hoặc họ tên..."
                value={state().search}
                onInput={(e: any) => update({ search: e.currentTarget.value })}
              />
            </div>
            <Button variant="secondary" size="sm" onClick={reset}>
              Xoá bộ lọc
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Error */}
      <Show when={users.isError}>
        <Card class="mb-4 border-[3px] border-red-600 bg-red-50 p-6">
          <div class="font-heading font-bold uppercase text-red-800">Lỗi tải dữ liệu</div>
          <Button variant="secondary" size="sm" class="mt-4" onClick={() => users.refetch()}>
            Thử lại
          </Button>
        </Card>
      </Show>

      {/* Table */}
      <DataTable<UserWithRoleDto>
        columns={columns}
        data={items()}
        loading={users.isPending}
        emptyMessage="Không tìm thấy người dùng"
        selectedIds={selectedIds()}
        onSelectionChange={setSelectedIds}
      />

      {/* Pagination */}
      <Show when={meta()}>
        <div class="mt-5">
          <Pagination
            page={meta()!.page}
            totalPages={meta()!.total_pages}
            total={meta()!.total}
            limit={meta()!.limit}
            onPageChange={(p) => update({ page: String(p) })}
          />
        </div>
      </Show>

      {/* Bulk action bar — appears when rows are selected */}
      <BulkActionBar
        selectedCount={selectedIds().size}
        onClearSelection={() => setSelectedIds(new Set())}
        actions={
          <div class="flex items-center gap-2">
            <select
              class="border-[2px] border-black bg-white px-2 py-1 font-mono text-xs focus:outline-none"
              value={bulkRoleSlug()}
              onChange={(e: any) => setBulkRoleSlug(e.currentTarget.value)}
            >
              <option value="">-- Chọn role --</option>
              <For each={roleOptions()}>
                {(r) => <option value={r.slug}>{r.slug}</option>}
              </For>
            </select>
            <Button
              variant="primary"
              size="xs"
              disabled={!bulkRoleSlug()}
              onClick={() => setConfirmBulk(true)}
            >
              Gán role
            </Button>
          </div>
        }
      />

      {/* Confirm bulk assign */}
      <ConfirmDialog
        open={confirmBulk()}
        title="Xác nhận gán role"
        message={`Gán role "${bulkRoleSlug()}" cho ${selectedIds().size} người dùng đã chọn?`}
        confirmLabel="Gán"
        loading={bulkAssign.isPending}
        onConfirm={handleBulkAssign}
        onCancel={() => setConfirmBulk(false)}
      />
    </>
  );
};

export default UserRoleList;
