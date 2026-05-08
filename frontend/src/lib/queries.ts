export const queryKeys = {
  notifications: {
    all: ['notifications'] as const,
    list: (params: any) => ['notifications', 'list', params] as const,
    unreadCount: ['notifications', 'unread-count'] as const,
    stats: ['notifications', 'stats'] as const,
  },
  files: {
    all: ['files'] as const,
    list: (params: any) => ['files', 'list', params] as const,
    detail: (id: string) => ['files', 'detail', id] as const,
    search: (q: string) => ['files', 'search', q] as const,
    tree: (id: string) => ['files', 'tree', id] as const,
  },
  users: {
    all: ['users'] as const,
    list: (params: any) => ['users', 'list', params] as const,
    detail: (id: string) => ['users', 'detail', id] as const,
    me: ['users', 'me'] as const,
    stats: ['users', 'stats'] as const,
  },
};
