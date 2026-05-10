import { createQuery } from '@tanstack/solid-query';

export function useDashboardStats() {
  return createQuery(() => ({
    queryKey: ['dashboard', 'stats'],
    queryFn: async () => {
      const res = await fetch('/api/dashboard/stats');
      if (!res.ok) throw new Error('Failed to load dashboard stats');
      return res.json();
    },
  }));
}

export function useRecentActivities(limit = 5) {
  return createQuery(() => ({
    queryKey: ['dashboard', 'activity-feed', limit],
    queryFn: async () => {
      const res = await fetch(`/api/dashboard/activity-feed?limit=${limit}`);
      if (!res.ok) return [];
      const json = await res.json();
      const items = Array.isArray(json) ? json : (json?.data ?? []);

      const relTime = (dateStr: string) => {
        const d = new Date(dateStr);
        const now = new Date();
        const diff = now.getTime() - d.getTime();
        const minutes = Math.floor(diff / 60000);
        const hours = Math.floor(diff / 3600000);
        const days = Math.floor(diff / 86400000);
        if (minutes < 1) return 'Just now';
        if (minutes < 60) return `${minutes}m ago`;
        if (hours < 24) return `${hours}h ago`;
        if (days < 7) return `${days}d ago`;
        return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
      };

      return (items as any[]).map((a) => {
        const resourceType = (a.resource_type ?? '').toLowerCase();
        const icon =
          resourceType.includes('task') ? '📋' :
          resourceType.includes('client') ? '👥' :
          resourceType.includes('file') ? '📁' :
          resourceType.includes('notification') ? '🔔' :
          '🧾';

        const color =
          resourceType.includes('task') ? 'bg-blue-50' :
          resourceType.includes('client') ? 'bg-green-50' :
          resourceType.includes('file') ? 'bg-purple-50' :
          resourceType.includes('notification') ? 'bg-yellow-50' :
          'bg-neutral-lightGray/30';

        return {
          id: a.id,
          action: a.action ?? 'Activity',
          detail: a.details ?? a.resource_id ?? a.resource_type ?? '',
          time: relTime(a.created_at ?? new Date().toISOString()),
          icon,
          color,
        };
      });
    },
  }));
}

