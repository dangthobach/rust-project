import { createMutation, createQuery, useQueryClient } from '@tanstack/solid-query';
import { api } from '~/lib/api';

// ── Query keys ────────────────────────────────────────────────────────────────

export const rbacKeys = {
  all: ['rbac'] as const,
  roles: () => [...rbacKeys.all, 'roles'] as const,
  rolesPaged: (p: object) => [...rbacKeys.roles(), 'paged', p] as const,
  role: (id: string) => [...rbacKeys.roles(), id] as const,
  permissions: () => [...rbacKeys.all, 'permissions'] as const,
  permissionsPaged: (p: object) => [...rbacKeys.permissions(), 'paged', p] as const,
  matrix: () => [...rbacKeys.all, 'matrix'] as const,
  usersWithRoles: (p: object) => [...rbacKeys.all, 'users-roles', p] as const,
};

// ── Roles — queries ───────────────────────────────────────────────────────────

export function useRolesPaged(params: () => { page?: number; limit?: number; search?: string }) {
  return createQuery(() => ({
    queryKey: rbacKeys.rolesPaged(params()),
    queryFn: () => api.listRolesPaged(params()),
  }));
}

export function useRole(id: () => string) {
  return createQuery(() => ({
    queryKey: rbacKeys.role(id()),
    queryFn: () => api.getRole(id()),
    enabled: !!id(),
  }));
}

export function usePermissionsPaged(params: () => { page?: number; limit?: number; search?: string }) {
  return createQuery(() => ({
    queryKey: rbacKeys.permissionsPaged(params()),
    queryFn: () => api.listPermissionsPaged(params()),
  }));
}

export function usePermissionMatrix() {
  return createQuery(() => ({
    queryKey: rbacKeys.matrix(),
    queryFn: () => api.getPermissionMatrix(),
  }));
}

export function useUsersWithRoles(params: () => { page?: number; limit?: number; search?: string }) {
  return createQuery(() => ({
    queryKey: rbacKeys.usersWithRoles(params()),
    queryFn: () => api.listUsersWithRoles(params()),
  }));
}

// ── Roles — mutations ────────────────────────────────────────────────────────

export function useCreateRole() {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: api.createRole,
    onSuccess: () => qc.invalidateQueries({ queryKey: rbacKeys.roles() }),
  }));
}

export function useUpdateRole() {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ id, data }: { id: string; data: Parameters<typeof api.updateRole>[1] }) =>
      api.updateRole(id, data),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: rbacKeys.roles() });
      qc.invalidateQueries({ queryKey: rbacKeys.role(vars.id) });
    },
  }));
}

export function useDeleteRole() {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: (id: string) => api.deleteRole(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: rbacKeys.roles() }),
  }));
}

export function useDeleteRolesBulk() {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: (ids: string[]) => api.deleteRolesBulk(ids),
    onSuccess: () => qc.invalidateQueries({ queryKey: rbacKeys.roles() }),
  }));
}

export function useUpdateRolePermissions() {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ roleId, permissionIds }: { roleId: string; permissionIds: string[] }) =>
      api.updateRolePermissions(roleId, permissionIds),
    onSuccess: () => qc.invalidateQueries({ queryKey: rbacKeys.matrix() }),
  }));
}

export function useAssignUserRole() {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ userId, roleSlug }: { userId: string; roleSlug: string }) =>
      api.assignUserRole(userId, roleSlug),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: rbacKeys.usersWithRoles({}) });
    },
  }));
}

export function useBulkAssignUserRole() {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ userIds, roleSlug }: { userIds: string[]; roleSlug: string }) =>
      api.bulkAssignUserRole(userIds, roleSlug),
    onSuccess: () => qc.invalidateQueries({ queryKey: rbacKeys.usersWithRoles({}) }),
  }));
}
