import { createQuery, createMutation, useQueryClient } from '@tanstack/solid-query';
import { api } from '~/lib/api';

export const useClients = (params: any = {}) => createQuery(() => ({
  queryKey: ['clients', params],
  queryFn: async () => {
    const res = await api.get<any>(`/clients?page=${params.page || 1}&limit=${params.limit || 10}`);
    const items = res.data || (Array.isArray(res) ? res : []);
    return {
      items,
      data: items,
      total: res.pagination?.total || items.length,
      page: res.pagination?.page || 1,
      total_pages: res.pagination?.total_pages || 1,
      pagination: res.pagination
    };
  },
}));

export const useClient = (id: () => string, enabled: () => boolean = () => true) => createQuery(() => ({
  queryKey: ['clients', id()],
  enabled: enabled(),
  queryFn: () => api.get<any>(`/clients/${id()}`),
}));

export const useCreateClient = () => {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: (data: any) => api.post('/clients', data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['clients'] }),
  }));
};

export const useUpdateClient = () => {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ id, ...data }: any) => api.patch(`/clients/${id}`, data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['clients'] }),
  }));
};

export const useDeleteClient = () => {
  const qc = useQueryClient();
  return createMutation(() => ({
    mutationFn: (id: string) => api.delete(`/clients/${id}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['clients'] }),
  }));
};
