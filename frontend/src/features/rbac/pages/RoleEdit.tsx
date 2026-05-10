import { Component, Show } from 'solid-js';
import { useParams, useNavigate } from '@solidjs/router';
import { PageHeader, Spinner, Card } from '~/components/ui';
import { useRole, useUpdateRole } from '../hooks';
import { RoleForm } from '../components/RoleForm';
import { toast } from '~/lib/toast';

const RoleEdit: Component = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const role = useRole(() => id);
  const updateRole = useUpdateRole();

  return (
    <>
      <PageHeader
        title="Chỉnh sửa Role"
        breadcrumbs={[
          { label: 'Admin' },
          { label: 'Roles', href: '/admin/rbac/roles' },
          { label: role.data?.slug ?? id, href: `/admin/rbac/roles/${id}` },
          { label: 'Sửa' },
        ]}
      />

      <Show when={role.isPending}>
        <div class="flex justify-center p-16">
          <Spinner />
        </div>
      </Show>

      <Show when={role.isError}>
        <Card class="border-[3px] border-red-600 bg-red-50 p-6">
          <div class="font-heading font-bold uppercase text-red-800">Không tìm thấy role</div>
        </Card>
      </Show>

      <Show when={role.data}>
        {(r) => (
          <RoleForm
            initial={r()}
            submitLabel="Lưu thay đổi"
            loading={updateRole.isPending}
            onCancel={() => navigate(`/admin/rbac/roles/${id}`)}
            onSubmit={(data) =>
              updateRole.mutate(
                { id, data },
                {
                  onSuccess: () => {
                    toast.success('Đã cập nhật role');
                    navigate(`/admin/rbac/roles/${id}`);
                  },
                  onError: (err: any) => toast.error(err.message ?? 'Cập nhật thất bại'),
                },
              )
            }
          />
        )}
      </Show>
    </>
  );
};

export default RoleEdit;
