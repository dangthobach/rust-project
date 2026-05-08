/**
 * Base API Client
 */

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

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = localStorage.getItem('auth_token');
  
  const headers = new Headers(options.headers);
  if (token) {
    headers.set('Authorization', `Bearer ${token}`);
  }
  if (!(options.body instanceof FormData) && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...options,
    headers,
  });

  if (response.status === 401) {
    localStorage.removeItem('auth_token');
    window.location.href = '/login';
    throw new ApiError(401, 'Unauthorized');
  }

  if (!response.ok) {
    const errorData = await response.json().catch(() => ({}));
    throw new ApiError(response.status, errorData.message || 'API request failed', errorData);
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

export const api = {
  get: <T>(path: string, options?: RequestInit) => request<T>(path, { ...options, method: 'GET' }),
  post: <T>(path: string, body?: any, options?: RequestInit) => 
    request<T>(path, { 
      ...options, 
      method: 'POST', 
      body: body instanceof FormData ? body : JSON.stringify(body) 
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
  getUser: (id: string) => request<any>(`/api/users/${id}`, { method: 'GET' }),
  completeTask: (id: string) => request<any>(`/tasks/${id}/complete`, { method: 'POST' }),
  updateTask: (id: string, data: any) => request<any>(`/tasks/${id}`, { method: 'PATCH', body: JSON.stringify(data) }),
  deleteTask: (id: string) => request<any>(`/tasks/${id}`, { method: 'DELETE' }),
};

export default api;
