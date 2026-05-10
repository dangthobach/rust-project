/**
 * Base API Client
 */

import { getAuthToken, clearAuth, setAuth } from './auth';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000/api';

export interface ApiResponse<T> {
  data: T;
  pagination?: {
    page: number;
    limit: number;
    total: number;
    total_pages: number;
  };
}

export class ApiError extends Error {
  constructor(public status: number, message: string, public data?: any) {
    super(message);
    this.name = 'ApiError';
  }
}

async function request<T>(
  path: string,
  options: RequestInit = {},
  extra: { skipAuthRedirect?: boolean } = {},
): Promise<T> {
  const token = getAuthToken();

  const headers = new Headers(options.headers);
  if (token) {
    headers.set('Authorization', `Bearer ${token}`);
  }
  if (!(options.body instanceof FormData) && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(`${API_BASE_URL}${path}`, { ...options, headers });
  const errorData = response.ok ? null : await response.json().catch(() => ({}));
  const errorMsg = (errorData as any)?.error || (errorData as any)?.message || 'Request failed';

  if (response.status === 401) {
    if (!extra.skipAuthRedirect) {
      clearAuth();
      window.location.href = '/login';
    }
    throw new ApiError(401, errorMsg, errorData);
  }

  if (!response.ok) {
    throw new ApiError(response.status, errorMsg, errorData);
  }

  if (response.status === 204) {
    return {} as T;
  }

  return response.json();
}

export interface Notification {
  id: string;
  title: string;
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
  read: boolean;
  is_read?: boolean; // Backward compatibility
  created_at: string;
}

export interface FileMetadata {
  id: string;
  name: string;
  original_name: string;
  file_type: string;
  mime_type: string; // Required now
  file_size: number;
  size: number; // Required now
  uploaded_by: string;
  created_at: string;
  updated_at?: string;
  thumbnail_path?: string;
}

export interface User {
  id: string;
  email: string;
  full_name: string;
  role: string;
  is_active: boolean;
  created_at: string;
}

export interface Client {
  id: string;
  full_name: string;
  name?: string; // Add this for compatibility
  company?: string; // Add this for compatibility
  email: string;
  phone?: string;
  created_at: string;
}

export type ReportFormat = 'pdf' | 'csv' | 'xlsx' | 'json';
export type ReportExportStatus = {
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'ready' | 'queued';
  download_url?: string;
  error_message?: string;
  job_id?: string;
};
export type ReportType = 'users' | 'clients' | 'tasks' | 'inventory';

// ── RBAC ──────────────────────────────────────────────────────────────────────

export interface CorePaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  limit: number;
  total_pages: number;
}

export interface RoleDto {
  id: string;
  slug: string;
  description?: string;
  is_active: boolean;
  created_at: string;
}

export interface PermissionDto {
  id: string;
  code: string;
  description?: string;
  is_active: boolean;
  created_at: string;
}

export interface UserWithRoleDto extends User {
  roles: RoleDto[];
}

export interface PermissionMatrixDto {
  roles: RoleDto[];
  permissions: PermissionDto[];
  assignments: Array<{ role_id: string; permission_id: string }>;
}

export interface MenuNode {
  key: string;
  label: string;
  path: string | null;
  icon: string | null;
  sort_order: number;
  children: MenuNode[];
}

export interface CreateRoleInput {
  slug: string;
  description?: string;
  is_active?: boolean;
}

export type UpdateRoleInput = Partial<CreateRoleInput>;

export const api = {
  get: <T>(path: string, options?: RequestInit) => request<T>(path, { ...options, method: 'GET' }),
  post: <T>(path: string, body?: any, options?: RequestInit) =>
    request<T>(path, {
      ...options,
      method: 'POST',
      body: body instanceof FormData ? body : JSON.stringify(body),
    }),
  patch: <T>(path: string, body?: any, options?: RequestInit) => 
    request<T>(path, { 
      ...options, 
      method: 'PATCH', 
      body: body instanceof FormData ? body : JSON.stringify(body) 
    }),
  put: <T>(path: string, body?: any, options?: RequestInit) => 
    request<T>(path, { 
      ...options, 
      method: 'PUT', 
      body: body instanceof FormData ? body : JSON.stringify(body) 
    }),
  delete: <T>(path: string, options?: RequestInit) => request<T>(path, { ...options, method: 'DELETE' }),

  // RBAC — Roles
  listRolesPaged: (params: { page?: number; limit?: number; search?: string }) => {
    const q = new URLSearchParams();
    if (params.page) q.set('page', String(params.page));
    if (params.limit) q.set('limit', String(params.limit));
    if (params.search) q.set('search', params.search);
    return request<CorePaginatedResponse<RoleDto>>(`/admin/rbac/roles/paged?${q}`);
  },
  getRole: (id: string) => request<RoleDto>(`/admin/rbac/roles/${id}`),
  createRole: (body: CreateRoleInput) =>
    request<RoleDto>('/admin/rbac/roles', { method: 'POST', body: JSON.stringify(body) }),
  updateRole: (id: string, body: UpdateRoleInput) =>
    request<RoleDto>(`/admin/rbac/roles/${id}`, { method: 'PATCH', body: JSON.stringify(body) }),
  deleteRole: (id: string) => request<void>(`/admin/rbac/roles/${id}`, { method: 'DELETE' }),
  deleteRolesBulk: (ids: string[]) =>
    request<void>('/admin/rbac/roles/bulk-delete', { method: 'POST', body: JSON.stringify({ ids }) }),

  // RBAC — Permissions
  listPermissionsPaged: (params: { page?: number; limit?: number; search?: string }) => {
    const q = new URLSearchParams();
    if (params.page) q.set('page', String(params.page));
    if (params.limit) q.set('limit', String(params.limit));
    if (params.search) q.set('search', params.search);
    return request<CorePaginatedResponse<PermissionDto>>(`/admin/rbac/permissions/paged?${q}`);
  },

  // RBAC — Permission Matrix
  getPermissionMatrix: () => request<PermissionMatrixDto>('/admin/rbac/matrix'),
  updateRolePermissions: (roleId: string, permissionIds: string[]) =>
    request<void>(`/admin/rbac/roles/${roleId}/permissions`, {
      method: 'PUT',
      body: JSON.stringify({ permission_ids: permissionIds }),
    }),

  // RBAC — User-Role assignment
  listUsersWithRoles: (params: { page?: number; limit?: number; search?: string }) => {
    const q = new URLSearchParams();
    if (params.page) q.set('page', String(params.page));
    if (params.limit) q.set('limit', String(params.limit));
    if (params.search) q.set('search', params.search);
    return request<CorePaginatedResponse<UserWithRoleDto>>(`/admin/users/with-roles?${q}`);
  },
  assignUserRole: (userId: string, roleSlug: string) =>
    request<void>(`/admin/users/${userId}/role`, {
      method: 'PUT',
      body: JSON.stringify({ role: roleSlug }),
    }),
  bulkAssignUserRole: (userIds: string[], roleSlug: string) =>
    request<void>('/admin/users/bulk-assign-role', {
      method: 'POST',
      body: JSON.stringify({ user_ids: userIds, role: roleSlug }),
    }),

  // Auth
  login: async (email: string, password: string) => {
    const data = await request<any>(
      '/auth/login',
      { method: 'POST', body: JSON.stringify({ email, password }) },
      { skipAuthRedirect: true },
    );
    setAuth(data);
    return data;
  },
  logout: async (refreshToken?: string) => {
    try {
      if (refreshToken) {
        await request<void>(
          '/auth/logout',
          { method: 'POST', body: JSON.stringify({ refresh_token: refreshToken }) },
          { skipAuthRedirect: true },
        );
      }
    } finally {
      clearAuth();
    }
  },
  refreshToken: async (refreshToken: string) => {
    const data = await request<any>(
      '/auth/refresh',
      { method: 'POST', body: JSON.stringify({ refresh_token: refreshToken }) },
      { skipAuthRedirect: true },
    );
    setAuth(data);
    return data;
  },

  // Dynamic menus
  getMyMenus: () => request<MenuNode[]>('/menus/my-menus'),

  // Backward compatibility methods
  uploadFile: (formData: FormData) => request<FileMetadata>('/fs/files/upload', { method: 'POST', body: formData }),
  getNotifications: () => request<Notification[]>('/notifications', { method: 'GET' }),
  exportClients: (format: string, params: any) => request<Blob>(`/export/clients?format=${format}`, { method: 'GET' }),
  exportTasks: (format: string, params: any) => request<Blob>(`/export/tasks?format=${format}`, { method: 'GET' }),
  listReportExports: (params: any) => request<any>('/reports/exports', { method: 'GET' }),
  getReportExport: (id: string) => request<any>(`/reports/exports/${id}`, { method: 'GET' }),
  startReportExport: (data: any) => request<any>('/reports/exports', { method: 'POST', body: JSON.stringify(data) }),
  unifiedSearch: (q: string, limit: number) => request<any>(`/search?q=${q}&limit=${limit}`, { method: 'GET' }),
  searchClients: (params: any) => request<any>(`/clients/search?q=${params.search_term}`, { method: 'GET' }),
  searchUsersAdmin: (params: any) => request<any>(`/admin/users/search?q=${params.search}`, { method: 'GET' }),
  getClient: (id: string) => request<any>(`/clients/${id}`, { method: 'GET' }),
  getUser: (id: string) => request<any>(`/users/${id}`, { method: 'GET' }),
  completeTask: (id: string) => request<any>(`/tasks/${id}/complete`, { method: 'POST' }),
  updateTask: (id: string, data: any) => request<any>(`/tasks/${id}`, { method: 'PATCH', body: JSON.stringify(data) }),
  deleteTask: (id: string) => request<any>(`/tasks/${id}`, { method: 'DELETE' }),
};

export default api;
