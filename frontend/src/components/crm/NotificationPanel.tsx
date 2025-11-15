import { Component, createSignal, For, Show, createResource } from 'solid-js';
import { Card, Badge, Button, Spinner } from '~/components/ui';
import { api } from '~/lib/api';

interface Notification {
  id: string;
  title: string;
  message: string;
  type: string;
  isRead: boolean;
  time: string;
}

export const NotificationPanel: Component = () => {
  const [showAll, setShowAll] = createSignal(false);
  const [notifications] = createResource<Notification[]>(() => api.getNotifications());

  const displayNotifications = () => {
    const items = notifications();
    if (!items) return [];
    return showAll() ? items : items.slice(0, 3);
  };

  const unreadCount = () => {
    const items = notifications();
    if (!items) return 0;
    return items.filter((n) => !n.isRead).length;
  };

  const getIcon = (type: string) => {
    const icons: Record<string, string> = {
      task: '✓',
      client: '👤',
      file: '📄',
    };
    return icons[type] || 'ℹ';
  };

  return (
    <Card class="w-full">
      <div class="p-4 border-b-3 border-black flex items-center justify-between">
        <h3 class="font-heading text-xl font-bold uppercase">Notifications</h3>
        <Show when={!notifications.loading && unreadCount() > 0}>
          <Badge variant="primary">{unreadCount()} New</Badge>
        </Show>
      </div>

      <Show
        when={!notifications.loading}
        fallback={
          <div class="p-8 flex justify-center">
            <Spinner />
          </div>
        }
      >
        <Show
          when={!notifications.error}
          fallback={
            <div class="p-4 text-center text-red-600">
              Failed to load notifications. Please try again.
            </div>
          }
        >
          <div class="max-h-[500px] overflow-y-auto">
            <div class="divide-y-3 divide-black">
              <For each={displayNotifications()}>
                {(notification) => (
                  <div
                    class={`p-4 cursor-pointer transition-all hover:bg-neutral-beige ${
                      !notification.isRead ? 'bg-primary/10' : ''
                    }`}
                  >
                    <div class="flex items-start gap-3">
                      <span class="text-2xl flex-shrink-0">
                        {getIcon(notification.type)}
                      </span>
                      <div class="flex-1 min-w-0">
                        <div class="flex items-start justify-between gap-2 mb-1">
                          <h4 class="font-heading font-bold text-sm">
                            {notification.title}
                          </h4>
                          {!notification.isRead && (
                            <span class="h-2 w-2 bg-primary rounded-full flex-shrink-0 mt-1" />
                          )}
                        </div>
                        <p class="text-sm text-neutral-darkGray mb-2">
                          {notification.message}
                        </p>
                        <span class="text-xs font-mono text-neutral-gray">
                          {notification.time}
                        </span>
                      </div>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </div>

          <Show when={notifications()?.length ?? 0 > 3}>
            <div class="p-4 border-t-3 border-black">
              <Button
                size="sm"
                variant="ghost"
                fullWidth
                onClick={() => setShowAll(!showAll())}
              >
                {showAll() ? 'Show Less' : `Show All (${notifications()?.length ?? 0})`}
              </Button>
            </div>
          </Show>
        </Show>
      </Show>
    </Card>
  );
};
