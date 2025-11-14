import { Component, createSignal, For } from 'solid-js';
import { Card, Badge, Button } from '~/components/ui';

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
  
  const notifications: Notification[] = [
    {
      id: '1',
      title: 'New Task Assigned',
      message: 'You have been assigned to "Prepare quarterly report"',
      type: 'task',
      isRead: false,
      time: '5 min ago',
    },
    {
      id: '2',
      title: 'Client Added',
      message: 'New client "Acme Corporation" has been added',
      type: 'client',
      isRead: false,
      time: '1 hour ago',
    },
    {
      id: '3',
      title: 'File Uploaded',
      message: 'Document "contract.pdf" has been uploaded',
      type: 'file',
      isRead: true,
      time: '2 hours ago',
    },
    {
      id: '4',
      title: 'Task Completed',
      message: '"Update CRM database" has been completed',
      type: 'task',
      isRead: true,
      time: '1 day ago',
    },
  ];

  const displayNotifications = () =>
    showAll() ? notifications : notifications.slice(0, 3);
  const unreadCount = () => notifications.filter((n) => !n.isRead).length;

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
        {unreadCount() > 0 && (
          <Badge variant="primary">{unreadCount()} New</Badge>
        )}
      </div>

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

      {notifications.length > 3 && (
        <div class="p-4 border-t-3 border-black">
          <Button
            size="sm"
            variant="ghost"
            fullWidth
            onClick={() => setShowAll(!showAll())}
          >
            {showAll() ? 'Show Less' : `Show All (${notifications.length})`}
          </Button>
        </div>
      )}
    </Card>
  );
};
