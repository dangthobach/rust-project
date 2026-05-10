import { Component, Show } from 'solid-js';
import { useParams, useNavigate, useSearchParams } from '@solidjs/router';
import { Badge, Button, Card, CardContent, CardHeader, CardTitle, PageHeader, Spinner } from '~/components/ui';
import { useRole } from '../hooks';

/**
 * Role Detail page.
 *
 * The "← Quay lại" button uses navigate(-1) so the browser restores the
 * exact previous URL (including all search params in the list page).
 */
const RoleDetail: Component = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const role = useRole(() => id);

  return (
    <>
      <PageHeader
        title={role.data?.slug ?? '...'}
        breadcrumbs={[
          { label: 'Admin' },
          { label: 'Roles', href: '/admin/rbac/roles' },
          { label: role.data?.slug ?? id },
        ]}
        actions={
          <Show when={role.data}>
            <Button variant="secondary" size="sm" onClick={() => navigate(-1)}>
              ← Quay lại
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={() => navigate(`/admin/rbac/roles/${id}/edit`)}
            >
              Chỉnh sửa
            </Button>
          </Show>
        }
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
          <div class="grid gap-4 sm:grid-cols-2">
            <Card class="border-[3px] border-black shadow-brutal-sm">
              <CardHeader>
                <CardTitle class="font-mono text-xs uppercase tracking-widest">Thông tin cơ bản</CardTitle>
              </CardHeader>
              <CardContent class="flex flex-col gap-3">
                <Row label="Slug" value={r().slug} />
                <Row label="Mô tả" value={r().description ?? '—'} />
                <Row
                  label="Trạng thái"
                  value={
                    <Badge variant={r().is_active ? 'success' : 'secondary'}>
                      {r().is_active ? 'Active' : 'Inactive'}
                    </Badge>
                  }
                />
                <Row label="Ngày tạo" value={r().created_at} />
              </CardContent>
            </Card>
          </div>
        )}
      </Show>
    </>
  );
};

const Row: Component<{ label: string; value: any }> = (props) => (
  <div class="flex flex-col gap-0.5">
    <dt class="font-heading text-[10px] font-black uppercase tracking-widest text-neutral-darkGray">
      {props.label}
    </dt>
    <dd class="font-mono text-sm">{props.value}</dd>
  </div>
);

export default RoleDetail;
