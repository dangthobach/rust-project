import { createQuery, createMutation, useQueryClient } from '@tanstack/solid-query';
import { api } from '~/lib/api';

export const useTasks = (params: any = {}) => createQuery(() => ({
  queryKey: ['tasks', params],
  queryFn: async () => {
    const res = await api.get<any>(`/tasks?page=${params.page || 1}&limit=${params.limit || 10}`);
    const items = res.data || (Array.isArray(res) ? res : []);
    return {
      items,
      data: items,
      total: res.pagination?.total || items.length,
      page: res.pagination?.page || 1,
      page_size: res.pagination?.limit || 10,
      total_pages: res.pagination?.total_pages || 1,
      pagination: res.pagination
    };
  },
}));

export const useTask = (id: () => string, enabled: () => boolean = () => true) => createQuery(() => ({
  queryKey: ['tasks', id()],
  enabled: enabled(),
  queryFn: () => api.get<any>(`/tasks/${id()}`),
}));

export const useCreateTask = () => {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: (data: any) => api.post('/tasks', data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['tasks'] }),
  }));
};

export const useUpdateTask = () => {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ id, ...data }: any) => api.patch(`/tasks/${id}`, data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['tasks'] }),
  }));
};

export const useDeleteTask = () => {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: (id: string) => api.delete(`/tasks/${id}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['tasks'] }),
  }));
};

export const useCompleteTask = () => {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: (id: string) => api.post(`/tasks/${id}/complete`, {}),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['tasks'] }),
  }));
};

export const useTaskStats = () => createQuery(() => ({
  queryKey: ['tasks', 'stats'],
  queryFn: () => api.get<any>('/tasks/stats'),
}));
