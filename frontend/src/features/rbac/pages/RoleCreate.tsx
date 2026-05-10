import { Component } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { PageHeader } from '~/components/ui';
import { useCreateRole } from '../hooks';
import { RoleForm } from '../components/RoleForm';
import { toast } from '~/lib/toast';

const RoleCreate: Component = () => {
  const navigate = useNavigate();
  const createRole = useCreateRole();

  return (
    <>
      <PageHeader
        title="Tạo Role mới"
        breadcrumbs={[
          { label: 'Admin' },
          { label: 'Roles', href: '/admin/rbac/roles' },
          { label: 'Tạo mới' },
        ]}
      />

      <RoleForm
        submitLabel="Tạo Role"
        loading={createRole.isPending}
        onCancel={() => navigate('/admin/rbac/roles')}
        onSubmit={(data) =>
          createRole.mutate(data, {
            onSuccess: (role) => {
              toast.success(`Đã tạo role "${role.slug}"`);
              navigate(`/admin/rbac/roles/${role.id}`);
            },
            onError: (err: any) => toast.error(err.message ?? 'Tạo role thất bại'),
          })
        }
      />
    </>
  );
};

export default RoleCreate;
