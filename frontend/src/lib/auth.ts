import { createSignal } from 'solid-js';

export interface AuthUser {
  id: string;
  email: string;
  full_name: string;
  avatar_url?: string | null;
  is_active: boolean;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface AuthRole {
  id: string;
  slug: string;
  description?: string | null;
}

export interface AuthBranch {
  id: string;
  parent_id?: string | null;
  name: string;
  slug: string;
}

export interface AuthState {
  user: AuthUser | null;
  permissions: string[];
  roles: AuthRole[];
  branches: AuthBranch[];
  access_token: string | null;
  refresh_token: string | null;
}

const STORAGE_KEY = 'auth_session';

function loadFromStorage(): AuthState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw) as AuthState;
  } catch {}
  // Backward compat: legacy token-only storage
  const legacyToken = localStorage.getItem('auth_token') || localStorage.getItem('access_token');
  return {
    user: null,
    permissions: [],
    roles: [],
    branches: [],
    access_token: legacyToken,
    refresh_token: null,
  };
}

const [authState, setAuthState] = createSignal<AuthState>(loadFromStorage());

export { authState };

export function getAuthToken(): string | null {
  return authState().access_token;
}

export function isAuthenticated(): boolean {
  return !!authState().access_token;
}

export function currentUser(): AuthUser | null {
  return authState().user;
}

export function hasPermission(code: string): boolean {
  return authState().permissions.includes(code);
}

export function hasRole(slug: string): boolean {
  return authState().roles.some((r) => r.slug === slug);
}

export function setAuth(data: {
  access_token: string;
  refresh_token: string;
  user: AuthUser;
  permissions?: string[];
  roles?: AuthRole[];
  branches?: AuthBranch[];
}) {
  const next: AuthState = {
    user: data.user,
    permissions: data.permissions ?? [],
    roles: data.roles ?? [],
    branches: data.branches ?? [],
    access_token: data.access_token,
    refresh_token: data.refresh_token,
  };
  setAuthState(next);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
  // Keep legacy keys so older code that reads them still works
  localStorage.setItem('auth_token', data.access_token);
  localStorage.setItem('access_token', data.access_token);
}

export function clearAuth() {
  const empty: AuthState = {
    user: null,
    permissions: [],
    roles: [],
    branches: [],
    access_token: null,
    refresh_token: null,
  };
  setAuthState(empty);
  localStorage.removeItem(STORAGE_KEY);
  localStorage.removeItem('auth_token');
  localStorage.removeItem('access_token');
}
