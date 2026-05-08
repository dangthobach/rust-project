import { createQuery, createMutation, useQueryClient } from '@tanstack/solid-query';
import { api } from '~/lib/api';
import { queryKeys } from '~/lib/queries';

export interface User {
  id: string;
  email: string;
  full_name: string;
  role: string;
  is_active: boolean;
  created_at: string;
}

export const useUsers = (params: () => { page: number; limit: number }) => {
  return createQuery(() => ({
    queryKey: queryKeys.users.list(params()),
    queryFn: () => api.get<{ data: User[]; pagination: any }>(`/admin/users?page=${params().page}&limit=${params().limit}`),
  }));
};

export const useCreateUser = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (user: Partial<User> & { password?: string }) => api.post<User>('/admin/users', user),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.users.all });
    },
  }));
};

export const useUpdateUser = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ id, ...user }: Partial<User> & { id: string }) => api.patch<User>(`/admin/users/${id}`, user),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.users.all });
    },
  }));
};

export const useDeleteUser = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (id: string) => api.delete(`/admin/users/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.users.all });
    },
  }));
};
