import { Component, createSignal, For } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent, Button, Badge, Spinner } from '~/components/ui';
import { ClientCard, TaskCard, NotificationPanel, DataChart } from '~/components/crm';

interface DashboardStats {
  totalClients: number;
  activeClients: number;
  totalTasks: number;
  completedTasks: number;
  pendingTasks: number;
  notifications: number;
}

const Dashboard: Component = () => {
  const [loading, setLoading] = createSignal(false);
  const [stats] = createSignal<DashboardStats>({
    totalClients: 42,
    activeClients: 38,
    totalTasks: 156,
    completedTasks: 98,
    pendingTasks: 58,
    notifications: 12,
  });

  const clients = () => [
    {
      name: 'Acme Corporation',
      email: 'contact@acme.com',
      phone: '+1 234 567 890',
      status: 'active' as const,
      lastContact: '2 hours ago',
    },
    {
      name: 'TechStart Inc',
      email: 'info@techstart.io',
      phone: '+1 234 567 891',
      status: 'active' as const,
      lastContact: '1 day ago',
    },
    {
      name: 'Design Studio',
      email: 'hello@designstudio.com',
      phone: '+1 234 567 892',
      status: 'inactive' as const,
      lastContact: '1 week ago',
    },
  ];

  const tasks = () => [
    {
      title: 'Follow up with Acme Corporation',
      description: 'Discuss new project requirements',
      priority: 'high' as const,
      dueDate: 'Today',
      status: 'pending' as const,
    },
    {
      title: 'Prepare quarterly report',
      description: 'Compile sales data for Q4',
      priority: 'medium' as const,
      dueDate: 'Tomorrow',
      status: 'in-progress' as const,
    },
    {
      title: 'Client meeting preparation',
      description: 'Review presentation slides',
      priority: 'high' as const,
      dueDate: 'Today',
      status: 'pending' as const,
    },
    {
      title: 'Update CRM database',
      description: 'Add new client information',
      priority: 'low' as const,
      dueDate: 'Next week',
      status: 'completed' as const,
    },
  ];

  return (
    <div>
      {/* Header */}
      <div class="mb-8">
        <div class="flex items-center justify-between flex-wrap gap-4">
          <div>
            <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">
              Dashboard
            </h1>
            <p class="text-neutral-darkGray mt-1">
              Welcome back! Here's your CRM overview
            </p>
          </div>
          <div class="flex gap-3">
            <Button variant="secondary" size="md">
              📊 Reports
            </Button>
            <Button variant="primary" size="md">
              ➕ New Client
            </Button>
          </div>
        </div>
      </div>

      {/* Stats Grid */}
      <div class="grid-brutal mb-8">
        <Card variant="primary" class="asymmetric-1">
          <CardHeader>
            <CardTitle>Total Clients</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-display-1 font-heading font-black">
              {stats().totalClients}
            </div>
            <Badge variant="success" class="mt-2">
              {stats().activeClients} Active
            </Badge>
          </CardContent>
        </Card>

        <Card variant="secondary" class="asymmetric-2">
          <CardHeader>
            <CardTitle>Total Tasks</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-display-1 font-heading font-black">
              {stats().totalTasks}
            </div>
            <div class="flex gap-2 mt-2">
              <Badge variant="success">✓ {stats().completedTasks}</Badge>
              <Badge variant="warning">⏳ {stats().pendingTasks}</Badge>
            </div>
          </CardContent>
        </Card>

        <Card variant="accent" class="asymmetric-3">
          <CardHeader>
            <CardTitle>Notifications</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-display-1 font-heading font-black">
              {stats().notifications}
            </div>
            <Badge variant="default" class="mt-2">
              New updates
            </Badge>
          </CardContent>
        </Card>

        <Card hoverable>
          <CardHeader>
            <CardTitle>Completion Rate</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-display-1 font-heading font-black">
              {Math.round((stats().completedTasks / stats().totalTasks) * 100)}%
            </div>
            <div class="w-full bg-neutral-concrete h-4 border-3 border-black mt-3">
              <div
                class="h-full bg-primary"
                style={{
                  width: `${(stats().completedTasks / stats().totalTasks) * 100}%`,
                }}
              />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Recent Activity */}
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-8">
        <div class="lg:col-span-2">
          <h2 class="text-heading-2 font-heading font-bold uppercase mb-4">
            Recent Clients
          </h2>
          <div class="grid gap-4">
            <For each={clients()}>
              {(client) => <ClientCard {...client} />}
            </For>
          </div>
        </div>

        <div>
          <h2 class="text-heading-2 font-heading font-bold uppercase mb-4">
            Notifications
          </h2>
          <NotificationPanel />
        </div>
      </div>

      {/* Tasks */}
      <div class="mb-8">
        <h2 class="text-heading-2 font-heading font-bold uppercase mb-4">
          Recent Tasks
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <For each={tasks()}>
            {(task) => <TaskCard {...task} />}
          </For>
        </div>
      </div>

      {/* Analytics */}
      <div>
        <h2 class="text-heading-2 font-heading font-bold uppercase mb-4">
          Analytics
        </h2>
        <DataChart />
      </div>
    </div>
  );
};

export default Dashboard;

