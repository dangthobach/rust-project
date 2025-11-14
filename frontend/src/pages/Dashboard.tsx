import { Component, createSignal, For, Show } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent, Button, Badge, Spinner } from '~/components/ui';
import { ClientCard, TaskCard, NotificationPanel, DataChart } from '~/components/crm';

interface DashboardStats {
  totalClients: number;
  activeClients: number;
  totalTasks: number;
  completedTasks: number;
  pendingTasks: number;
  notifications: number;
  revenue: number;
  growth: number;
  activeUsers: number;
  filesUploaded: number;
}

const Dashboard: Component = () => {
  const [loading, setLoading] = createSignal(false);
  const [selectedPeriod, setSelectedPeriod] = createSignal('week');
  const [stats] = createSignal<DashboardStats>({
    totalClients: 42,
    activeClients: 38,
    totalTasks: 156,
    completedTasks: 98,
    pendingTasks: 58,
    notifications: 12,
    revenue: 127500,
    growth: 23.5,
    activeUsers: 8,
    filesUploaded: 234,
  });

  const quickActions = [
    { icon: '➕', label: 'New Client', color: 'bg-primary', action: () => console.log('New Client') },
    { icon: '📋', label: 'New Task', color: 'bg-accent-yellow', action: () => console.log('New Task') },
    { icon: '📊', label: 'Generate Report', color: 'bg-secondary', action: () => console.log('Report') },
    { icon: '📧', label: 'Send Email', color: 'bg-green-400', action: () => console.log('Email') },
  ];

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

  const recentActivity = [
    { icon: '👤', action: 'New client added', detail: 'Acme Corporation', time: '2 hours ago', color: 'bg-primary' },
    { icon: '✅', action: 'Task completed', detail: 'Quarterly report finished', time: '4 hours ago', color: 'bg-green-400' },
    { icon: '📧', action: 'Email sent', detail: 'To 15 clients', time: '6 hours ago', color: 'bg-secondary' },
    { icon: '📁', action: 'File uploaded', detail: 'contract.pdf', time: '1 day ago', color: 'bg-accent-yellow' },
  ];

  return (
    <div>
      {/* Enhanced Header with More Info */}
      <div class="mb-8">
        <div class="flex items-center justify-between flex-wrap gap-6">
          <div>
            <div class="flex items-center gap-4 mb-2">
              <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">
                Dashboard
              </h1>
              <Badge variant="success" class="text-lg px-4 py-2 border-4">
                <span class="flex items-center gap-2">
                  <span class="w-3 h-3 bg-green-500 rounded-full animate-pulse"></span>
                  Live
                </span>
              </Badge>
            </div>
            <p class="text-neutral-darkGray text-lg">
              Welcome back! Here's your CRM overview for {selectedPeriod() === 'week' ? 'this week' : selectedPeriod() === 'month' ? 'this month' : 'today'}
            </p>
            <div class="flex items-center gap-3 mt-3">
              <button
                class={`px-4 py-2 text-sm font-bold uppercase border-3 border-black transition-all ${selectedPeriod() === 'today' ? 'bg-primary shadow-brutal' : 'bg-white hover:shadow-brutal-sm'}`}
                onClick={() => setSelectedPeriod('today')}
              >
                Today
              </button>
              <button
                class={`px-4 py-2 text-sm font-bold uppercase border-3 border-black transition-all ${selectedPeriod() === 'week' ? 'bg-primary shadow-brutal' : 'bg-white hover:shadow-brutal-sm'}`}
                onClick={() => setSelectedPeriod('week')}
              >
                Week
              </button>
              <button
                class={`px-4 py-2 text-sm font-bold uppercase border-3 border-black transition-all ${selectedPeriod() === 'month' ? 'bg-primary shadow-brutal' : 'bg-white hover:shadow-brutal-sm'}`}
                onClick={() => setSelectedPeriod('month')}
              >
                Month
              </button>
            </div>
          </div>
          
          {/* Quick Actions */}
          <div class="flex flex-wrap gap-3">
            <For each={quickActions}>
              {(action) => (
                <Button
                  variant="primary"
                  size="md"
                  class={`${action.color} border-5 border-black shadow-brutal-lg hover:shadow-brutal-xl group`}
                  onClick={action.action}
                >
                  <span class="text-2xl group-hover:scale-125 transition-transform">{action.icon}</span>
                  <span class="font-black">{action.label}</span>
                </Button>
              )}
            </For>
          </div>
        </div>
      </div>

      {/* Enhanced Stats Grid with More Metrics */}
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <Card variant="primary" class="transform hover:-translate-y-2 hover:shadow-brutal-xl transition-all duration-200">
          <CardHeader>
            <div class="flex items-center justify-between">
              <CardTitle>Total Clients</CardTitle>
              <span class="text-4xl">👥</span>
            </div>
          </CardHeader>
          <CardContent>
            <div class="text-display-1 font-heading font-black">
              {stats().totalClients}
            </div>
            <div class="flex items-center gap-2 mt-3">
              <Badge variant="success" class="border-3">
                {stats().activeClients} Active
              </Badge>
              <span class="text-sm text-neutral-darkGray">
                +{stats().growth}% ↗
              </span>
            </div>
            <div class="mt-4 flex gap-2">
              <Button variant="secondary" size="sm" class="flex-1 text-xs">
                View All
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card variant="secondary" class="transform hover:-translate-y-2 hover:shadow-brutal-xl transition-all duration-200">
          <CardHeader>
            <div class="flex items-center justify-between">
              <CardTitle>Total Tasks</CardTitle>
              <span class="text-4xl">📋</span>
            </div>
          </CardHeader>
          <CardContent>
            <div class="text-display-1 font-heading font-black">
              {stats().totalTasks}
            </div>
            <div class="flex gap-2 mt-3">
              <Badge variant="success" class="border-3">✓ {stats().completedTasks}</Badge>
              <Badge variant="warning" class="border-3">⏳ {stats().pendingTasks}</Badge>
            </div>
            <div class="mt-4">
              <Button variant="primary" size="sm" class="w-full text-xs bg-accent-yellow">
                + New Task
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card variant="accent" class="transform hover:-translate-y-2 hover:shadow-brutal-xl transition-all duration-200">
          <CardHeader>
            <div class="flex items-center justify-between">
              <CardTitle>Revenue</CardTitle>
              <span class="text-4xl">💰</span>
            </div>
          </CardHeader>
          <CardContent>
            <div class="text-display-1 font-heading font-black">
              ${(stats().revenue / 1000).toFixed(1)}K
            </div>
            <div class="flex items-center gap-2 mt-3">
              <Badge variant="success" class="border-3">
                +{stats().growth}%
              </Badge>
              <span class="text-sm text-neutral-darkGray">
                vs last {selectedPeriod()}
              </span>
            </div>
            <div class="mt-4">
              <Button variant="secondary" size="sm" class="w-full text-xs">
                View Report
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card hoverable class="transform hover:-translate-y-2 hover:shadow-brutal-xl transition-all duration-200 bg-gradient-to-br from-white to-neutral-beige">
          <CardHeader>
            <div class="flex items-center justify-between">
              <CardTitle>Completion</CardTitle>
              <span class="text-4xl">📊</span>
            </div>
          </CardHeader>
          <CardContent>
            <div class="text-display-1 font-heading font-black">
              {Math.round((stats().completedTasks / stats().totalTasks) * 100)}%
            </div>
            <div class="w-full bg-neutral-concrete h-5 border-4 border-black mt-3 overflow-hidden">
              <div
                class="h-full bg-primary transition-all duration-500 ease-out"
                style={{
                  width: `${(stats().completedTasks / stats().totalTasks) * 100}%`,
                }}
              />
            </div>
            <div class="mt-3 text-xs text-neutral-darkGray font-bold">
              {stats().completedTasks} of {stats().totalTasks} tasks completed
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Additional Stats Row */}
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
        <div class="bg-white border-5 border-black p-4 shadow-brutal hover:shadow-brutal-lg transition-all">
          <div class="flex items-center gap-3">
            <span class="text-3xl">🔔</span>
            <div>
              <div class="text-2xl font-heading font-black">{stats().notifications}</div>
              <div class="text-xs font-bold uppercase text-neutral-darkGray">Notifications</div>
            </div>
          </div>
        </div>
        
        <div class="bg-white border-5 border-black p-4 shadow-brutal hover:shadow-brutal-lg transition-all">
          <div class="flex items-center gap-3">
            <span class="text-3xl">👨‍💼</span>
            <div>
              <div class="text-2xl font-heading font-black">{stats().activeUsers}</div>
              <div class="text-xs font-bold uppercase text-neutral-darkGray">Active Users</div>
            </div>
          </div>
        </div>
        
        <div class="bg-white border-5 border-black p-4 shadow-brutal hover:shadow-brutal-lg transition-all">
          <div class="flex items-center gap-3">
            <span class="text-3xl">📁</span>
            <div>
              <div class="text-2xl font-heading font-black">{stats().filesUploaded}</div>
              <div class="text-xs font-bold uppercase text-neutral-darkGray">Files</div>
            </div>
          </div>
        </div>
        
        <div class="bg-gradient-to-r from-primary to-accent-yellow border-5 border-black p-4 shadow-brutal hover:shadow-brutal-lg transition-all">
          <div class="flex items-center gap-3">
            <span class="text-3xl">⚡</span>
            <div>
              <div class="text-2xl font-heading font-black">98%</div>
              <div class="text-xs font-bold uppercase text-black">Uptime</div>
            </div>
          </div>
        </div>
      </div>

      {/* Recent Activity */}
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-8">
        <div class="lg:col-span-2">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-heading-2 font-heading font-bold uppercase">
              Recent Clients
            </h2>
            <Button variant="secondary" size="sm" class="border-4">
              View All →
            </Button>
          </div>
          <div class="grid gap-4">
            <For each={clients()}>
              {(client) => <ClientCard {...client} />}
            </For>
          </div>
        </div>

        <div>
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-heading-2 font-heading font-bold uppercase">
              Notifications
            </h2>
            <Badge variant="error" class="border-3">
              {stats().notifications}
            </Badge>
          </div>
          <NotificationPanel />
        </div>
      </div>

      {/* Recent Activity Timeline */}
      <div class="mb-8">
        <h2 class="text-heading-2 font-heading font-bold uppercase mb-4">
          Recent Activity
        </h2>
        <Card class="border-5">
          <CardContent class="p-6">
            <div class="space-y-4">
              <For each={recentActivity}>
                {(activity) => (
                  <div class="flex items-start gap-4 pb-4 border-b-3 border-black last:border-b-0 last:pb-0">
                    <div class={`w-12 h-12 ${activity.color} border-4 border-black shadow-brutal flex items-center justify-center text-2xl flex-shrink-0`}>
                      {activity.icon}
                    </div>
                    <div class="flex-1">
                      <div class="font-heading font-bold text-lg">{activity.action}</div>
                      <div class="text-neutral-darkGray">{activity.detail}</div>
                      <div class="text-xs text-neutral-darkGray mt-1">{activity.time}</div>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Tasks */}
      <div class="mb-8">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-heading-2 font-heading font-bold uppercase">
            Recent Tasks
          </h2>
          <div class="flex gap-2">
            <Badge variant="success" class="border-3">
              {stats().completedTasks} Done
            </Badge>
            <Badge variant="warning" class="border-3">
              {stats().pendingTasks} Pending
            </Badge>
          </div>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <For each={tasks()}>
            {(task) => <TaskCard {...task} />}
          </For>
        </div>
      </div>

      {/* Analytics */}
      <div>
        <h2 class="text-heading-2 font-heading font-bold uppercase mb-4">
          Analytics & Performance
        </h2>
        <DataChart />
      </div>
    </div>
  );
};

export default Dashboard;

