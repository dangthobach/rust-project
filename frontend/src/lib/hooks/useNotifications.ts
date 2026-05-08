import { createQuery, createMutation, useQueryClient } from '@tanstack/solid-query';
import { api } from '~/lib/api';
import { queryKeys } from '~/lib/queries';

export interface Notification {
  id: string;
  title: string;
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
  read: boolean;
  created_at: string;
}

export const useNotifications = (params: () => { page: number; limit: number; read?: boolean }) => {
  return createQuery(() => ({
    queryKey: queryKeys.notifications.list(params()),
    queryFn: async () => {
      const res = await api.get<any>(`/notifications?page=${params().page}&limit=${params().limit}${params().read !== undefined ? `&read=${params().read}` : ''}`);
      const items = Array.isArray(res) ? res : (res.data || []);
      return {
        items,
        data: items,
        total: res.total || items.length,
        pagination: res.pagination
      };
    },
  }));
};

export const useMarkAllAsRead = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: () => api.post('/notifications/mark-read/all', {}),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.notifications.all });
    },
  }));
};

export const useDeleteNotification = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (id: string) => api.delete(`/notifications/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.notifications.all });
    },
  }));
};

export const useBulkDeleteNotifications = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (ids: string[]) => api.post('/notifications/bulk-delete', { ids }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.notifications.all });
    },
  }));
};

export const useDeleteAllRead = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: () => api.delete('/notifications/read'),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.notifications.all });
    },
  }));
};

export const useNotificationStats = () => {
  return createQuery(() => ({
    queryKey: queryKeys.notifications.stats,
    queryFn: () => api.get<any>('/notifications/stats'),
    refetchInterval: 30000,
  }));
};

export const useUnreadCount = () => {
  return createQuery(() => ({
    queryKey: queryKeys.notifications.unreadCount,
    queryFn: async () => {
      const res = await api.get<{ count: number }>('/notifications/unread-count');
      return res.count;
    },
    refetchInterval: 30000, // Poll every 30s as fallback for WS
  }));
};

export const useMarkAsRead = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (ids: string[]) => api.post('/notifications/mark-read', { ids }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.notifications.all });
    },
  }));
};
