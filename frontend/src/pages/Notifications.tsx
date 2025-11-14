import { Component } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent } from '~/components/ui';
import { NotificationPanel } from '~/components/crm';

const Notifications: Component = () => {
  return (
    <div>
      <div class="mb-8">
        <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">
          Notifications
        </h1>
        <p class="text-neutral-darkGray mt-1">
          Stay updated with your CRM activities
        </p>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div>
          <h2 class="text-heading-2 font-heading font-bold uppercase mb-4">
            Recent Notifications
          </h2>
          <NotificationPanel />
        </div>

        <div>
          <h2 class="text-heading-2 font-heading font-bold uppercase mb-4">
            Settings
          </h2>
          <Card>
            <CardHeader>
              <CardTitle>Notification Preferences</CardTitle>
            </CardHeader>
            <CardContent>
              <p class="text-neutral-darkGray">
                Configure your notification settings here.
              </p>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
};

export default Notifications;

