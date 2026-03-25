import { Component, createSignal, Show, For, createMemo } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent, Button, Spinner } from '~/components/ui';
import { 
  useNotifications, 
  useMarkAsRead, 
  useMarkAllAsRead,
  useDeleteNotification,
  useBulkDeleteNotifications,
  useDeleteAllRead,
  useNotificationStats
} from '~/lib/hooks/useNotifications';
import type { Notification } from '~/lib/api';

const Notifications: Component = () => {
  const [page, setPage] = createSignal(1);
  const [limit] = createSignal(20);
  const [readFilter, setReadFilter] = createSignal<boolean | undefined>(undefined);
  const [typeFilter, setTypeFilter] = createSignal<'info' | 'success' | 'warning' | 'error' | ''>('');
  const [selectedIds, setSelectedIds] = createSignal<string[]>([]);
  const [showDeleteConfirm, setShowDeleteConfirm] = createSignal<string | null>(null);

  // Queries and mutations
  const notifications = useNotifications(() => ({ 
    page: page(), 
    limit: limit(),
    read: readFilter(),
    type: typeFilter() || undefined
  }));
  
  const stats = useNotificationStats();
  const markAsRead = useMarkAsRead();
  const markAllAsRead = useMarkAllAsRead();
  const deleteNotification = useDeleteNotification();
  const bulkDelete = useBulkDeleteNotifications();
  const deleteAllRead = useDeleteAllRead();

  const displayNotifications = createMemo(() => notifications.data?.data || []);
  const pagination = createMemo(() => notifications.data?.pagination);

  // Handlers
  const handleMarkAsRead = (id: string) => {
    markAsRead.mutate([id]);
  };

  const handleMarkAllAsRead = () => {
    markAllAsRead.mutate();
  };

  const handleDeleteClick = (id: string) => {
    setShowDeleteConfirm(id);
  };

  const handleDeleteConfirm = () => {
    const id = showDeleteConfirm();
    if (id) {
      deleteNotification.mutate(id);
      setShowDeleteConfirm(null);
    }
  };

  const handleDeleteCancel = () => {
    setShowDeleteConfirm(null);
  };

  const handleDeleteSelected = () => {
    if (selectedIds().length === 0) return;
    
    if (confirm(`Delete ${selectedIds().length} selected notifications?`)) {
      bulkDelete.mutate(selectedIds());
      setSelectedIds([]);
    }
  };

  const handleDeleteAllRead = () => {
    if (confirm('Delete all read notifications?')) {
      deleteAllRead.mutate();
    }
  };

  const toggleSelection = (id: string) => {
    setSelectedIds(prev => 
      prev.includes(id) 
        ? prev.filter(nid => nid !== id)
        : [...prev, id]
    );
  };

  const toggleSelectAll = () => {
    const allIds = displayNotifications().map(n => n.id);
    if (selectedIds().length === allIds.length) {
      setSelectedIds([]);
    } else {
      setSelectedIds(allIds);
    }
  };

  // Utility functions
  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);
    
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined,
    });
  };

  const getNotificationIcon = (type: string) => {
    switch (type) {
      case 'success': return '✅';
      case 'warning': return '⚠️';
      case 'error': return '❌';
      default: return 'ℹ️';
    }
  };

  const getNotificationColor = (type: string) => {
    switch (type) {
      case 'success': return 'bg-green-50 border-green-500';
      case 'warning': return 'bg-yellow-50 border-yellow-500';
      case 'error': return 'bg-red-50 border-red-500';
      default: return 'bg-blue-50 border-blue-500';
    }
  };

  return (
    <div>
      {/* Header */}
      <div class="mb-8">
        <div class="flex items-center justify-between flex-wrap gap-4">
          <div>
            <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">
              Notifications
            </h1>
            <p class="text-neutral-darkGray mt-1">
              Manage your notifications and alerts
            </p>
          </div>
          
          <div class="flex gap-3 flex-wrap">
            <Show when={selectedIds().length > 0}>
              <Button 
                variant="danger" 
                size="md"
                onClick={handleDeleteSelected}
              >
                🗑️ Delete Selected ({selectedIds().length})
              </Button>
            </Show>
            
            <Button 
              variant="ghost" 
              size="md"
              onClick={handleDeleteAllRead}
              disabled={deleteAllRead.isPending}
            >
              🗑️ Delete All Read
            </Button>
            
            <Button 
              variant="primary" 
              size="md"
              onClick={handleMarkAllAsRead}
              disabled={markAllAsRead.isPending}
            >
              ✓ Mark All Read
            </Button>
          </div>
        </div>
      </div>

      {/* Stats Cards */}
      <Show when={stats.data}>
        {(s) => (
          <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
            <div class="p-4 border-3 border-black bg-white">
              <div class="text-2xl font-bold">{s().total}</div>
              <div class="text-sm text-neutral-darkGray">Total</div>
            </div>
            <div class="p-4 border-3 border-black bg-blue-50">
              <div class="text-2xl font-bold">{s().unread}</div>
              <div class="text-sm text-neutral-darkGray">Unread</div>
            </div>
            <div class="p-4 border-3 border-black bg-green-50">
              <div class="text-2xl font-bold">{s().read}</div>
              <div class="text-sm text-neutral-darkGray">Read</div>
            </div>
            <div class="p-4 border-3 border-black bg-yellow-50">
              <div class="text-2xl font-bold">{s().byType.warning + s().byType.error}</div>
              <div class="text-sm text-neutral-darkGray">Warnings & Errors</div>
            </div>
          </div>
        )}
      </Show>

      {/* Filter Bar */}
      <div class="mb-6 flex gap-4 flex-wrap">
        <div class="flex gap-2">
          <button
            class={`px-4 py-2 border-3 border-black font-bold ${
              readFilter() === undefined ? 'bg-primary-yellow' : 'bg-white'
            }`}
            onClick={() => setReadFilter(undefined)}
          >
            All
          </button>
          <button
            class={`px-4 py-2 border-3 border-black font-bold ${
              readFilter() === false ? 'bg-primary-yellow' : 'bg-white'
            }`}
            onClick={() => setReadFilter(false)}
          >
            Unread
          </button>
          <button
            class={`px-4 py-2 border-3 border-black font-bold ${
              readFilter() === true ? 'bg-primary-yellow' : 'bg-white'
            }`}
            onClick={() => setReadFilter(true)}
          >
            Read
          </button>
        </div>
        
        <select
          class="px-4 py-2 border-3 border-black font-bold bg-white cursor-pointer"
          value={typeFilter()}
          onChange={(e) => setTypeFilter(e.currentTarget.value as any)}
        >
          <option value="">All Types</option>
          <option value="info">Info</option>
          <option value="success">Success</option>
          <option value="warning">Warning</option>
          <option value="error">Error</option>
        </select>
      </div>

      {/* Notifications List */}
      <Card>
        <CardHeader>
          <div class="flex items-center justify-between">
            <CardTitle>
              Notification Center
              <Show when={pagination()}>
                {(p) => (
                  <span class="ml-2 text-sm font-normal text-neutral-darkGray">
                    ({p().total} total)
                  </span>
                )}
              </Show>
            </CardTitle>
            
            <Show when={displayNotifications().length > 0}>
              <Button
                variant="ghost"
                size="sm"
                onClick={toggleSelectAll}
              >
                {selectedIds().length === displayNotifications().length ? '☑️ Deselect All' : '☐ Select All'}
              </Button>
            </Show>
          </div>
        </CardHeader>
        <CardContent>
          <Show
            when={!notifications.isLoading}
            fallback={
              <div class="py-12 flex justify-center">
                <Spinner />
              </div>
            }
          >
            <Show
              when={displayNotifications().length > 0}
              fallback={
                <div class="text-center py-12">
                  <div class="text-6xl mb-4">🔔</div>
                  <p class="text-neutral-darkGray">
                    {readFilter() === false 
                      ? 'No unread notifications'
                      : readFilter() === true
                      ? 'No read notifications'
                      : 'No notifications yet'
                    }
                  </p>
                </div>
              }
            >
              <div class="space-y-3">
                <For each={displayNotifications()}>
                  {(notification) => (
                    <div 
                      class={`p-4 border-3 border-black transition-all group ${
                        notification.read 
                          ? 'bg-neutral-lightGray/30 opacity-70' 
                          : getNotificationColor(notification.type)
                      }`}
                    >
                      <div class="flex items-start gap-3">
                        {/* Checkbox */}
                        <input
                          type="checkbox"
                          class="mt-1 w-5 h-5 cursor-pointer"
                          checked={selectedIds().includes(notification.id)}
                          onChange={() => toggleSelection(notification.id)}
                        />
                        
                        {/* Icon */}
                        <span class="text-2xl mt-0.5">
                          {getNotificationIcon(notification.type)}
                        </span>
                        
                        {/* Content */}
                        <div class="flex-1">
                          <div class="flex items-start justify-between gap-2">
                            <div class="flex-1">
                              <h4 class="font-heading font-bold text-sm mb-1">
                                {notification.title}
                              </h4>
                              <p class="text-sm text-neutral-darkGray mb-2">
                                {notification.message}
                              </p>
                              <p class="text-xs text-neutral-darkGray">
                                {formatDate(notification.created_at)}
                              </p>
                            </div>
                            
                            {/* Action Buttons */}
                            <div class="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                              <Show when={!notification.read}>
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => handleMarkAsRead(notification.id)}
                                  disabled={markAsRead.isPending}
                                >
                                  ✓ Mark Read
                                </Button>
                              </Show>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => handleDeleteClick(notification.id)}
                                class="text-red-600 hover:bg-red-50"
                              >
                                🗑️
                              </Button>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  )}
                </For>
              </div>

              {/* Pagination */}
              <Show when={pagination() && (pagination() as any)!.total_pages > 1}>
                {(p) => (
                  <div class="mt-6 flex items-center justify-center gap-2">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setPage(p => Math.max(1, p - 1))}
                      disabled={!(p() as any).has_prev}
                    >
                      ← Previous
                    </Button>
                    
                    <span class="px-4 py-2 font-bold">
                      Page {(p() as any).page} of {(p() as any).total_pages}
                    </span>
                    
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setPage(p => p + 1)}
                      disabled={!(p() as any).has_next}
                    >
                      Next →
                    </Button>
                  </div>
                )}
              </Show>
            </Show>
          </Show>
        </CardContent>
      </Card>

      {/* Delete Confirmation Modal */}
      <Show when={showDeleteConfirm()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div class="bg-white border-4 border-black p-6 max-w-md w-full mx-4">
            <h3 class="text-xl font-heading font-black mb-4">Delete Notification?</h3>
            <p class="text-neutral-darkGray mb-6">
              Are you sure you want to delete this notification? This action cannot be undone.
            </p>
            <div class="flex gap-3 justify-end">
              <Button
                variant="ghost"
                size="md"
                onClick={handleDeleteCancel}
              >
                Cancel
              </Button>
              <Button
                variant="danger"
                size="md"
                onClick={handleDeleteConfirm}
              >
                Delete
              </Button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default Notifications;

