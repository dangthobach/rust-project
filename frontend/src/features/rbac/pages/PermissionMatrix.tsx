import { Component, Show, createSignal, createMemo, For, batch } from 'solid-js';
import { Button, PageHeader, Spinner, Card } from '~/components/ui';
import { usePermissionMatrix, useUpdateRolePermissions } from '../hooks';
import { toast } from '~/lib/toast';
import type { PermissionMatrixDto } from '~/lib/api';

/**
 * Permission Matrix: role (rows) × permission (columns) grid.
 * Changes are staged locally and submitted as a batch PUT per role.
 *
 *         | read | write | delete |
 * admin   |  ✓   |   ✓   |   ✓   |
 * manager |  ✓   |   ✓   |   ✗   |
 * viewer  |  ✓   |   ✗   |   ✗   |
 */
const PermissionMatrix: Component = () => {
  const matrix = usePermissionMatrix();
  const updateRolePerms = useUpdateRolePermissions();

  // Local staged assignments: Map<roleId, Set<permId>>
  const [staged, setStaged] = createSignal<Map<string, Set<string>> | null>(null);
  const [isDirty, setIsDirty] = createSignal(false);

  /** Initialise staged state from server data */
  const effective = createMemo(() => {
    if (staged() !== null) return staged()!;
    const data = matrix.data;
    if (!data) return new Map<string, Set<string>>();
    const map = new Map<string, Set<string>>();
    for (const role of data.roles) {
      map.set(role.id, new Set());
    }
    for (const a of data.assignments) {
      map.get(a.role_id)?.add(a.permission_id);
    }
    return map;
  });

  function toggle(roleId: string, permId: string) {
    const current = new Map(effective());
    const perms = new Set(current.get(roleId) ?? []);
    perms.has(permId) ? perms.delete(permId) : perms.add(permId);
    current.set(roleId, perms);
    batch(() => {
      setStaged(current);
      setIsDirty(true);
    });
  }

  function discardChanges() {
    batch(() => {
      setStaged(null);
      setIsDirty(false);
    });
  }

  async function saveAll() {
    const data = matrix.data;
    if (!data) return;
    const map = effective();
    const promises = data.roles.map((role) =>
      updateRolePerms.mutateAsync({ roleId: role.id, permissionIds: [...(map.get(role.id) ?? [])] }),
    );
    try {
      await Promise.all(promises);
      batch(() => {
        setStaged(null);
        setIsDirty(false);
      });
      toast.success('Đã lưu ma trận phân quyền');
    } catch (err: any) {
      toast.error(err.message ?? 'Lưu thất bại');
    }
  }

  return (
    <>
      <PageHeader
        title="Ma trận phân quyền"
        description="Cấu hình quyền cho từng role"
        breadcrumbs={[
          { label: 'Admin' },
          { label: 'Roles', href: '/admin/rbac/roles' },
          { label: 'Ma trận' },
        ]}
        actions={
          <Show when={isDirty()}>
            <Button variant="secondary" size="sm" onClick={discardChanges}>
              Huỷ thay đổi
            </Button>
            <Button
              variant="primary"
              size="sm"
              disabled={updateRolePerms.isPending}
              onClick={saveAll}
            >
              {updateRolePerms.isPending ? '...' : 'Lưu tất cả'}
            </Button>
          </Show>
        }
      />

      <Show when={matrix.isPending}>
        <div class="flex justify-center p-16">
          <Spinner />
        </div>
      </Show>

      <Show when={matrix.isError}>
        <Card class="border-[3px] border-red-600 bg-red-50 p-6">
          <div class="font-heading font-bold uppercase text-red-800">Lỗi tải dữ liệu</div>
        </Card>
      </Show>

      <Show when={matrix.data}>
        {(data) => <MatrixGrid data={data()} effective={effective()} onToggle={toggle} />}
      </Show>
    </>
  );
};

// ── Grid sub-component ────────────────────────────────────────────────────────

interface MatrixGridProps {
  data: PermissionMatrixDto;
  effective: Map<string, Set<string>>;
  onToggle: (roleId: string, permId: string) => void;
}

const MatrixGrid: Component<MatrixGridProps> = (props) => {
  return (
    <div class="overflow-x-auto border-[3px] border-black bg-white shadow-brutal">
      <table class="border-collapse font-mono text-xs">
        <thead>
          <tr class="bg-neutral-lightGray">
            <th class="sticky left-0 z-10 min-w-[140px] border-b-[3px] border-r-[3px] border-black bg-neutral-lightGray px-4 py-2 text-left font-heading text-[10px] font-black uppercase">
              Role \ Permission
            </th>
            <For each={props.data.permissions}>
              {(perm) => (
                <th class="min-w-[80px] border-b-[3px] border-black px-3 py-2 text-center font-heading text-[10px] font-black uppercase">
                  <div class="max-w-[80px] truncate" title={perm.code}>
                    {perm.code}
                  </div>
                  <Show when={perm.description}>
                    <div class="mt-0.5 font-mono text-[9px] font-normal normal-case tracking-normal text-neutral-darkGray">
                      {perm.description}
                    </div>
                  </Show>
                </th>
              )}
            </For>
          </tr>
        </thead>
        <tbody>
          <For each={props.data.roles}>
            {(role, ri) => (
              <tr class={ri() % 2 === 0 ? 'bg-white' : 'bg-neutral-lightGray/30'}>
                <td class="sticky left-0 z-10 border-b-[3px] border-r-[3px] border-black bg-inherit px-4 py-3">
                  <div class="font-heading font-bold">{role.slug}</div>
                  <Show when={role.description}>
                    <div class="font-mono text-[9px] text-neutral-darkGray">{role.description}</div>
                  </Show>
                </td>
                <For each={props.data.permissions}>
                  {(perm) => {
                    const checked = () => props.effective.get(role.id)?.has(perm.id) ?? false;
                    return (
                      <td class="border-b-[3px] border-black px-3 py-3 text-center">
                        <input
                          type="checkbox"
                          checked={checked()}
                          onChange={() => props.onToggle(role.id, perm.id)}
                          class="h-4 w-4 cursor-pointer accent-black"
                          aria-label={`${role.slug} — ${perm.code}`}
                        />
                      </td>
                    );
                  }}
                </For>
              </tr>
            )}
          </For>
        </tbody>
      </table>
    </div>
  );
};

export default PermissionMatrix;
