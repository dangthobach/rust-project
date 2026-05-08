export * from './useFiles';
export * from './useNotifications';
export * from './useUsers';
export * from './useClients';
export * from './useTasks';

import { createQuery } from '@tanstack/solid-query';
import { api } from '~/lib/api';

export const useDashboardStats = () => createQuery(() => ({
  queryKey: ['dashboard', 'stats'],
  queryFn: () => api.get<any>('/dashboard/stats'),
}));

export const useRecentActivities = (limit: number = 10) => createQuery(() => ({
  queryKey: ['activities', 'recent', limit],
  queryFn: () => api.get<any[]>(`/activities?limit=${limit}`),
}));

export const useRolesPaged = (params: any) => createQuery(() => ({
  queryKey: ['rbac', 'roles', params],
  queryFn: () => api.get<any>('/admin/rbac/roles/paged'),
}));

export const usePermissionsPaged = (params: any) => createQuery(() => ({
  queryKey: ['rbac', 'permissions', params],
  queryFn: () => api.get<any>('/admin/rbac/permissions/paged'),
}));

export const useCurrentUser = () => createQuery(() => ({
  queryKey: ['users', 'me'],
  queryFn: () => api.get<any>('/users/me'),
}));
