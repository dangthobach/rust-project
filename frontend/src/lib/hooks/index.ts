export * from './useFiles';
export { useMyMenus } from './useMenus';
export * from './useNotifications';
export * from './useUsers';
export * from './useClients';
export * from './useTasks';
export * from './useRbac';
export { useListState } from './useListState';

import { createQuery } from '@tanstack/solid-query';
import { api } from '~/lib/api';
import { authState } from '~/lib/auth';

export const useDashboardStats = () => createQuery(() => ({
  queryKey: ['dashboard', 'stats'],
  queryFn: () => api.get<any>('/dashboard/stats'),
}));

export const useRecentActivities = (limit: number = 10) => createQuery(() => ({
  queryKey: ['activities', 'recent', limit],
  queryFn: () => api.get<any[]>(`/activities?limit=${limit}`),
}));

export const useCurrentUser = () => createQuery(() => ({
  queryKey: ['users', 'me'],
  queryFn: () => api.get<any>('/users/me'),
  // Seed from auth store so the header renders immediately without a round-trip
  initialData: authState().user ?? undefined,
  initialDataUpdatedAt: 0,
  staleTime: 30_000,
}));

export const useAuthState = () => authState;
