import { createQuery, createMutation, useQueryClient } from '@tanstack/solid-query';
import { api } from '~/lib/api';
import { queryKeys } from '~/lib/queries';

export interface FileMetadata {
  id: string;
  name: string;
  original_name: string;
  file_type: string;
  file_size: number;
  uploaded_by: string;
  created_at: string;
  thumbnail_path?: string;
}

export const useFiles = (params: () => { page: number; limit: number; tab?: string }) => {
  return createQuery(() => ({
    queryKey: queryKeys.files.list(params()),
    queryFn: async () => {
      const res = await api.get<any>(`/fs/files?page=${params().page}&limit=${params().limit}&tab=${params().tab || 'recent'}`);
      // Ensure backward compatibility with different UI expectations
      return {
        data: res.data || res,
        items: res.data || res,
        pagination: res.pagination
      };
    },
  }));
};

export const useSearchFiles = (query: () => string, params: () => any) => {
  return createQuery(() => ({
    queryKey: queryKeys.files.search(query()),
    enabled: query().length > 0,
    queryFn: async () => {
      const res = await api.get<any>(`/fs/files/search?q=${query()}&page=${params().page}&limit=${params().limit}`);
      return {
        data: res.data || res,
        items: res.data || res,
        pagination: res.pagination
      };
    },
  }));
};

export const useThumbnailUrl = (id: () => string, enabled: () => boolean) => {
  return createQuery(() => ({
    queryKey: ['files', 'thumbnail', id()],
    enabled: enabled(),
    queryFn: () => api.get<any>(`/fs/files/${id()}/thumbnail-url`),
  }));
};

export const useUploadFile = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ file, parent_id }: { file: File; parent_id?: string }) => {
      const formData = new FormData();
      formData.append('file', file);
      if (parent_id) formData.append('parent_id', parent_id);
      return api.post<FileMetadata>('/fs/files/upload', formData);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.files.all });
    },
  }));
};

export const useDeleteFile = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (id: string) => api.delete(`/fs/files/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.files.all });
    },
  }));
};

export const useFile = (id: () => string, enabled: () => boolean) => {
  return createQuery(() => ({
    queryKey: queryKeys.files.detail(id()),
    enabled: enabled(),
    queryFn: () => api.get<FileMetadata>(`/fs/files/${id()}`),
  }));
};

export const useFileVersions = (id: () => string, enabled: () => boolean) => {
  return createQuery(() => ({
    queryKey: ['files', 'versions', id()],
    enabled: enabled(),
    queryFn: () => api.get<any[]>(`/fs/files/${id()}/versions`),
  }));
};

export const useRollbackVersion = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ fileId, versionId }: { fileId: string; versionId: string }) => 
      api.post(`/fs/files/${fileId}/rollback`, { version_id: versionId }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.files.detail(variables.fileId) });
      queryClient.invalidateQueries({ queryKey: ['files', 'versions', variables.fileId] });
      queryClient.invalidateQueries({ queryKey: ['files', 'activity', variables.fileId] });
      queryClient.invalidateQueries({ queryKey: ['files', 'download-url', variables.fileId] });
    },
  }));
};

export const useFileDownloadUrl = (id: () => string, enabled: () => boolean) => {
  return createQuery(() => ({
    queryKey: ['files', 'download-url', id()],
    enabled: enabled(),
    queryFn: () => api.get<any>(`/fs/files/${id()}/download-url`),
    retry: false, // Don't retry auth/presign failures endlessly
  }));
};

export const useFileActivity = (id: () => string, enabled: () => boolean) => {
  return createQuery(() => ({
    queryKey: ['files', 'activity', id()],
    enabled: enabled(),
    queryFn: () => api.get<any[]>(`/fs/files/${id()}/activity`),
  }));
};

export const useDownloadFile = () => {
  return createMutation(() => ({
    mutationFn: async ({ id, filename }: { id: string; filename?: string }) => {
      // First try to get a presigned URL
      try {
        const { download_url } = await api.get<{ download_url: string }>(`/fs/files/${id}/download-url`);
        window.open(download_url, '_blank');
      } catch (e) {
        // Fallback to direct download
        const token = localStorage.getItem('auth_token');
        const url = `http://localhost:3000/api/fs/files/${id}/download?token=${token}`;
        
        // Create a temporary link to trigger download with filename if possible
        const link = document.createElement('a');
        link.href = url;
        if (filename) link.download = filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
      }
    },
  }));
};
