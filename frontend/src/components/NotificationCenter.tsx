/**
 * Notification Center Component
 * Displays notifications dropdown with real-time WebSocket updates
 */

import { Component, createSignal, For, Show, onMount, onCleanup } from 'solid-js';
import { Button, Badge } from '~/components/ui';
import { useNotifications, useUnreadCount, useMarkAsRead } from '~/lib/hooks/useNotifications';
import { useWebSocket } from '~/lib/websocket';
import { useQueryClient } from '@tanstack/solid-query';
import { queryKeys } from '~/lib/queries';
import type { Notification } from '~/lib/api';
import { A } from '@solidjs/router';

export const NotificationCenter: Component = () => {
  const [isOpen, setIsOpen] = createSignal(false);
  const queryClient = useQueryClient();

  // Get notifications (limited to recent 10)
  const notifications = useNotifications(() => ({ page: 1, limit: 10, read: false }));
  const unreadCount = useUnreadCount();
  const markAsRead = useMarkAsRead();
  const ws = useWebSocket();

  // WebSocket real-time updates
  onMount(() => {
    // Subscribe to notification events
    const unsubscribe = ws.on('notification', (message) => {
      console.log('[NotificationCenter] New notification:', message);
      
      // Invalidate notifications cache to refetch
      queryClient.invalidateQueries({ queryKey: queryKeys.notifications.all });
      queryClient.invalidateQueries({ queryKey: ['notifications', 'unread-count'] });
    });

    onCleanup(() => {
      unsubscribe();
    });
  });

  const handleNotificationClick = (notification: Notification) => {
    if (!notification.read) {
      markAsRead.mutate([notification.id]);
    }
    setIsOpen(false);
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

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    return `${days}d ago`;
  };

  const getConnectionStatusColor = () => {
    const status = ws.status();
    switch (status) {
      case 'connected': return 'bg-green-500';
      case 'connecting': return 'bg-yellow-500 animate-pulse';
      case 'error': return 'bg-red-500';
      default: return 'bg-gray-500';
    }
  };

  return (
    <div class="relative">
      {/* Notification Bell Button */}
      <button
        onClick={() => setIsOpen(!isOpen())}
        class="relative p-2 hover:bg-neutral-beige rounded-lg transition-colors"
        aria-label="Notifications"
      >
        <div class="flex items-center gap-2">
          {/* Connection Status Indicator */}
          <div 
            class={`w-2 h-2 rounded-full ${getConnectionStatusColor()}`}
            title={`WebSocket: ${ws.status()}`}
          />
          
          {/* Bell Icon */}
          <span class="text-2xl">🔔</span>
          
          {/* Unread Badge */}
          <Show when={unreadCount.data && unreadCount.data > 0}>
            <Badge variant="danger" class="absolute -top-1 -right-1 min-w-[20px] h-5 flex items-center justify-center">
              {unreadCount.data}
            </Badge>
          </Show>
        </div>
      </button>

      {/* Dropdown Panel */}
      <Show when={isOpen()}>
        <div 
          class="absolute right-0 mt-2 w-96 max-h-[500px] overflow-hidden bg-white border-4 border-black shadow-brutal z-50"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div class="p-4 border-b-3 border-black bg-neutral-lightGray">
            <div class="flex items-center justify-between">
              <h3 class="font-heading font-bold text-lg">Notifications</h3>
              <div class="flex items-center gap-2">
                <Show when={unreadCount.data && unreadCount.data > 0}>
                  <span class="text-sm text-neutral-darkGray">
                    {unreadCount.data} unread
                  </span>
                </Show>
                <A href="/notifications" class="text-sm font-bold text-primary-blue hover:underline">
                  View All
                </A>
              </div>
            </div>
          </div>

          {/* Notifications List */}
          <div class="max-h-[400px] overflow-y-auto">
            <Show
              when={!notifications.isLoading}
              fallback={
                <div class="p-8 text-center">
                  <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-black" />
                </div>
              }
            >
              <Show
                when={notifications.data?.items && notifications.data.items.length > 0}
                fallback={
                  <div class="p-8 text-center text-neutral-darkGray">
                    <div class="text-4xl mb-2">🔔</div>
                    <p>No unread notifications</p>
                  </div>
                }
              >
                <For each={notifications.data?.items}>
                  {(notification) => (
                    <button
                      onClick={() => handleNotificationClick(notification)}
                      class={`w-full p-4 border-b-2 border-neutral-lightGray hover:bg-neutral-beige transition-colors text-left ${
                        getNotificationColor(notification.type)
                      }`}
                    >
                      <div class="flex items-start gap-3">
                        <span class="text-2xl flex-shrink-0">
                          {getNotificationIcon(notification.type)}
                        </span>
                        <div class="flex-1 min-w-0">
                          <h4 class="font-heading font-bold text-sm mb-1 truncate">
                            {notification.title}
                          </h4>
                          <p class="text-sm text-neutral-darkGray line-clamp-2 mb-1">
                            {notification.message}
                          </p>
                          <p class="text-xs text-neutral-darkGray">
                            {formatTimestamp(notification.created_at)}
                          </p>
                        </div>
                      </div>
                    </button>
                  )}
                </For>
              </Show>
            </Show>
          </div>

          {/* Footer */}
          <div class="p-3 border-t-3 border-black bg-neutral-lightGray text-center">
            <A 
              href="/notifications" 
              class="font-bold text-sm text-primary-blue hover:underline"
              onClick={() => setIsOpen(false)}
            >
              View All Notifications →
            </A>
          </div>
        </div>
      </Show>

      {/* Click Outside to Close */}
      <Show when={isOpen()}>
        <div 
          class="fixed inset-0 z-40" 
          onClick={() => setIsOpen(false)}
        />
      </Show>
    </div>
  );
};

export default NotificationCenter;
