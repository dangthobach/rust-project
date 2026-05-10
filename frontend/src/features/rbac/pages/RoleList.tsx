import { Component, createSignal, createMemo, For, Show } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import {
  Badge, Button, Card, CardContent, CardHeader, CardTitle,
  DataTable, Pagination, PageHeader, BulkActionBar, ConfirmDialog,
  Input,
} from '~/components/ui';
import type { ColumnDef } from '~/components/ui';
import { useListState } from '~/lib/hooks/useListState';
import { useRolesPaged, useDeleteRolesBulk } from '../hooks';
import type { RoleDto } from '~/lib/api';

/**
 * RBAC Role List
 *
 * Pattern:
 * - Search/filter state stored in URL → state persists when user presses Back from Detail
 * - Multi-select rows → bulk delete via BulkActionBar
 * - Row click → navigate to detail (Back button on Detail returns here with same state)
 */
const RoleList: Component = () => {
  const navigate = useNavigate();
  const { state, update, reset, page, limit } = useListState({
    page: '1',
    limit: '20',
    search: '',
  });

  const [selectedIds, setSelectedIds] = createSignal<Set<string>>(new Set());
  const [confirmDelete, setConfirmDelete] = createSignal(false);

  const roles = useRolesPaged(() => ({
    page: page(),
    limit: limit(),
    search: state().search.trim() || undefined,
  }));

  const bulkDelete = useDeleteRolesBulk();

  const meta = createMemo(() => roles.data);
  const items = createMemo(() => meta()?.items ?? []);

  const columns: ColumnDef<RoleDto>[] = [
    {
      key: 'slug',
      header: 'Slug',
      width: 'w-40',
      render: (r) => <span class="font-bold">{r.slug}</span>,
    },
    {
      key: 'description',
      header: 'Mô tả',
      render: (r) => <span class="text-neutral-darkGray">{r.description ?? '—'}</span>,
    },
    {
      key: 'is_active',
      header: 'Trạng thái',
      width: 'w-24',
      render: (r) => (
        <Badge variant={r.is_active ? 'success' : 'secondary'}>
          {r.is_active ? 'Active' : 'Inactive'}
        </Badge>
      ),
    },
    {
      key: 'created_at',
      header: 'Ngày tạo',
      width: 'w-36',
      render: (r) => <span class="text-neutral-darkGray">{r.created_at.slice(0, 10)}</span>,
    },
  ];

  function handleBulkDelete() {
    const ids = [...selectedIds()];
    bulkDelete.mutate(ids, {
      onSuccess: () => {
        setSelectedIds(new Set());
        setConfirmDelete(false);
      },
    });
  }

  return (
    <>
      <PageHeader
        title="Roles"
        description="Quản lý danh sách role trong hệ thống RBAC"
        breadcrumbs={[
          { label: 'Admin' },
          { label: 'RBAC' },
          { label: 'Roles' },
        ]}
        actions={
          <>
            <Button variant="secondary" size="sm" onClick={() => navigate('/admin/rbac/permissions')}>
              Permissions →
            </Button>
            <Button variant="secondary" size="sm" onClick={() => navigate('/admin/rbac/matrix')}>
              Ma trận →
            </Button>
            <Button variant="primary" size="sm" onClick={() => navigate('/admin/rbac/roles/new')}>
              + Tạo Role
            </Button>
          </>
        }
      />

      {/* Search form */}
      <Card class="mb-6 border-[3px] border-black bg-white shadow-brutal-sm">
        <CardHeader>
          <CardTitle class="font-mono text-xs uppercase tracking-widest">Tìm kiếm</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="flex flex-wrap items-center gap-3">
            <div class="min-w-[240px] flex-1">
              <Input
                type="text"
                placeholder="Tìm theo slug..."
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
      <Show when={roles.isError}>
        <Card class="mb-4 border-[3px] border-red-600 bg-red-50 p-6">
          <div class="font-heading font-bold uppercase text-red-800">Lỗi tải dữ liệu</div>
          <div class="mt-1 font-mono text-sm text-neutral-darkGray">
            {String(roles.error?.message ?? '')}
          </div>
          <Button variant="secondary" size="sm" class="mt-4" onClick={() => roles.refetch()}>
            Thử lại
          </Button>
        </Card>
      </Show>

      {/* Table */}
      <DataTable<RoleDto>
        columns={columns}
        data={items()}
        loading={roles.isPending}
        emptyMessage="Chưa có role nào"
        selectedIds={selectedIds()}
        onSelectionChange={setSelectedIds}
        onRowClick={(r) => navigate(`/admin/rbac/roles/${r.id}`)}
        rowActions={(r) => (
          <div class="flex items-center justify-end gap-1">
            <Button
              variant="ghost"
              size="xs"
              onClick={(e: MouseEvent) => {
                e.stopPropagation();
                navigate(`/admin/rbac/roles/${r.id}`);
              }}
            >
              Chi tiết
            </Button>
            <Button
              variant="ghost"
              size="xs"
              onClick={(e: MouseEvent) => {
                e.stopPropagation();
                navigate(`/admin/rbac/roles/${r.id}/edit`);
              }}
            >
              Sửa
            </Button>
          </div>
        )}
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

      {/* Bulk action bar */}
      <BulkActionBar
        selectedCount={selectedIds().size}
        onClearSelection={() => setSelectedIds(new Set())}
        actions={
          <Button
            variant="danger"
            size="xs"
            onClick={() => setConfirmDelete(true)}
          >
            Xoá {selectedIds().size} role
          </Button>
        }
      />

      {/* Confirm delete dialog */}
      <ConfirmDialog
        open={confirmDelete()}
        title="Xác nhận xoá"
        message={`Bạn có chắc muốn xoá ${selectedIds().size} role đã chọn? Hành động này không thể hoàn tác.`}
        confirmLabel="Xoá"
        variant="danger"
        loading={bulkDelete.isPending}
        onConfirm={handleBulkDelete}
        onCancel={() => setConfirmDelete(false)}
      />
    </>
  );
};

export default RoleList;
