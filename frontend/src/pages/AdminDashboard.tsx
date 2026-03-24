/**
 * Admin Dashboard Page
 * Displays system metrics, activity feed, and health status
 */

import { Component, Show, For } from 'solid-js';
import { useDashboard } from '../lib/hooks';
import { formatDistanceToNow } from 'date-fns';

const AdminDashboard: Component = () => {
  const { stats, activities, health, isLoading, isError } = useDashboard();

  return (
    <div class="container mx-auto p-6 space-y-6">
      {/* Page Header */}
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-4xl font-black tracking-tight">Admin Dashboard</h1>
          <p class="text-muted-foreground mt-1">System overview and activity monitoring</p>
        </div>
        
        {/* Health Status Badge */}
        <Show when={health.data}>
          <div class="flex items-center gap-2 px-4 py-2 bg-card border-4 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]">
            <div class={`w-3 h-3 rounded-full ${health.data?.status === 'ok' ? 'bg-green-500' : 'bg-red-500'}`} />
            <span class="font-bold text-sm">
              {health.data?.status === 'ok' ? 'System Healthy' : 'System Error'}
            </span>
          </div>
        </Show>
      </div>

      <Show when={isLoading}>
        <div class="flex items-center justify-center h-96">
          <div class="animate-spin rounded-full h-16 w-16 border-t-4 border-b-4 border-primary"></div>
        </div>
      </Show>

      <Show when={isError}>
        <div class="bg-destructive/10 border-4 border-destructive p-6 text-center">
          <p class="text-destructive font-bold text-lg">Failed to load dashboard data</p>
          <p class="text-sm mt-2">Please check your connection and try again</p>
        </div>
      </Show>

      <Show when={!isLoading && !isError && stats.data}>
        {/* Metrics Grid */}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {/* Users Metric */}
          <MetricCard
            title="Total Users"
            value={stats.data!.activities.total}
            subtitle={`${stats.data!.activities.today} active today`}
            trend={`${stats.data!.activities.this_week} this week`}
            icon="👥"
            color="bg-blue-400"
          />

          {/* Clients Metric */}
          <MetricCard
            title="Total Clients"
            value={stats.data!.clients.total}
            subtitle={`${stats.data!.clients.active} active`}
            trend={`+${stats.data!.clients.new_this_month} this month`}
            icon="🏢"
            color="bg-purple-400"
          />

          {/* Tasks Metric */}
          <MetricCard
            title="Total Tasks"
            value={stats.data!.tasks.total}
            subtitle={`${stats.data!.tasks.completed} completed`}
            trend={`${stats.data!.tasks.overdue} overdue`}
            icon="✅"
            color="bg-green-400"
          />

          {/* Files Metric */}
          <MetricCard
            title="Total Files"
            value={stats.data!.files.total}
            subtitle={formatBytes(stats.data!.files.total_size)}
            trend={`${stats.data!.files.uploaded_this_week} this week`}
            icon="📁"
            color="bg-orange-400"
          />
        </div>

        {/* Detailed Stats Grid */}
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Tasks Breakdown */}
          <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
            <h3 class="text-xl font-black mb-4">Tasks by Status</h3>
            <div class="space-y-3">
              <StatBar label="Pending" value={stats.data!.tasks.pending} total={stats.data!.tasks.total} color="bg-yellow-400" />
              <StatBar label="In Progress" value={stats.data!.tasks.in_progress} total={stats.data!.tasks.total} color="bg-blue-400" />
              <StatBar label="Completed" value={stats.data!.tasks.completed} total={stats.data!.tasks.total} color="bg-green-400" />
              <StatBar label="Cancelled" value={stats.data!.tasks.cancelled} total={stats.data!.tasks.total} color="bg-gray-400" />
            </div>
            
            <div class="mt-4 pt-4 border-t-2 border-black">
              <div class="flex justify-between text-sm">
                <span class="font-bold">Due Today:</span>
                <span class="text-destructive font-bold">{stats.data!.tasks.due_today}</span>
              </div>
              <div class="flex justify-between text-sm mt-1">
                <span class="font-bold">Due This Week:</span>
                <span class="font-bold">{stats.data!.tasks.due_this_week}</span>
              </div>
            </div>
          </div>

          {/* Clients Breakdown */}
          <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
            <h3 class="text-xl font-black mb-4">Client Status</h3>
            <div class="space-y-3">
              <StatBar label="Active" value={stats.data!.clients.active} total={stats.data!.clients.total} color="bg-green-400" />
              <StatBar label="Inactive" value={stats.data!.clients.inactive} total={stats.data!.clients.total} color="bg-gray-400" />
            </div>
            
            <div class="mt-4 pt-4 border-t-2 border-black">
              <div class="flex justify-between text-sm">
                <span class="font-bold">New This Week:</span>
                <span class="text-primary font-bold">{stats.data!.clients.new_this_week}</span>
              </div>
              <div class="flex justify-between text-sm mt-1">
                <span class="font-bold">New This Month:</span>
                <span class="text-primary font-bold">{stats.data!.clients.new_this_month}</span>
              </div>
            </div>
          </div>

          {/* Activity Summary */}
          <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
            <h3 class="text-xl font-black mb-4">Activity Overview</h3>
            <div class="space-y-4">
              <div class="bg-primary/10 border-2 border-primary p-3">
                <div class="text-3xl font-black">{stats.data!.activities.today}</div>
                <div class="text-sm font-bold">Activities Today</div>
              </div>
              <div class="bg-secondary/10 border-2 border-secondary p-3">
                <div class="text-3xl font-black">{stats.data!.activities.this_week}</div>
                <div class="text-sm font-bold">This Week</div>
              </div>
              <div class="bg-muted border-2 border-black p-3">
                <div class="text-3xl font-black">{stats.data!.activities.total}</div>
                <div class="text-sm font-bold">Total Activities</div>
              </div>
            </div>
          </div>
        </div>

        {/* Activity Feed */}
        <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
          <div class="flex items-center justify-between mb-6">
            <h3 class="text-2xl font-black">Recent Activity</h3>
            <span class="text-sm text-muted-foreground font-bold">Last 10 activities</span>
          </div>

          <Show when={activities.data && activities.data.data.length > 0}>
            <div class="space-y-3">
              <For each={activities.data!.data}>
                {(activity) => (
                  <div class="flex items-start gap-4 p-4 bg-muted/30 border-2 border-black hover:bg-muted/50 transition-colors">
                    {/* User Avatar */}
                    <div class="w-10 h-10 rounded-full border-2 border-black bg-primary flex items-center justify-center font-bold text-primary-foreground flex-shrink-0">
                      {activity.user_name.charAt(0).toUpperCase()}
                    </div>

                    {/* Activity Details */}
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2 mb-1">
                        <span class="font-bold">{activity.user_name}</span>
                        <span class="text-muted-foreground">•</span>
                        <span class="text-sm text-muted-foreground">
                          {formatDistanceToNow(new Date(activity.created_at), { addSuffix: true })}
                        </span>
                      </div>
                      
                      <div class="text-sm">
                        <span class="font-semibold">{activity.action}</span>
                        {activity.resource_type && (
                          <span class="text-muted-foreground"> • {activity.resource_type}</span>
                        )}
                      </div>
                      
                      <Show when={activity.details}>
                        <div class="text-xs text-muted-foreground mt-1">{activity.details}</div>
                      </Show>
                    </div>

                    {/* Action Icon */}
                    <div class="flex-shrink-0">
                      <div class="w-8 h-8 rounded border-2 border-black bg-secondary flex items-center justify-center">
                        {getActionIcon(activity.action)}
                      </div>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>

          <Show when={!activities.data || activities.data.data.length === 0}>
            <div class="text-center py-12 text-muted-foreground">
              <p class="text-lg font-bold">No recent activities</p>
              <p class="text-sm mt-1">Activity will appear here as users interact with the system</p>
            </div>
          </Show>

          {/* Pagination Info */}
          <Show when={activities.data && activities.data.pagination.total > 10}>
            <div class="mt-6 pt-4 border-t-2 border-black text-center text-sm text-muted-foreground">
              Showing {activities.data!.data.length} of {activities.data!.pagination.total} activities
            </div>
          </Show>
        </div>

        {/* Notifications Summary */}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
            <h3 class="text-xl font-black mb-4">📬 Notifications</h3>
            <div class="flex items-center justify-between">
              <div>
                <div class="text-4xl font-black">{stats.data!.notifications.unread}</div>
                <div class="text-sm text-muted-foreground font-bold">Unread Messages</div>
              </div>
              <div class="text-right">
                <div class="text-2xl font-bold text-muted-foreground">{stats.data!.notifications.total}</div>
                <div class="text-xs text-muted-foreground">Total</div>
              </div>
            </div>
          </div>

          {/* System Health Details */}
          <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
            <h3 class="text-xl font-black mb-4">🏥 System Health</h3>
            <Show when={health.data}>
              <div class="space-y-2">
                <div class="flex justify-between">
                  <span class="font-bold">Status:</span>
                  <span class={`font-bold ${health.data?.status === 'ok' ? 'text-green-600' : 'text-red-600'}`}>
                    {health.data?.status.toUpperCase()}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span class="font-bold">Database:</span>
                  <span class={`font-bold ${health.data?.database === 'healthy' ? 'text-green-600' : 'text-red-600'}`}>
                    {health.data?.database}
                  </span>
                </div>
                <div class="flex justify-between text-sm">
                  <span class="text-muted-foreground">Version:</span>
                  <span class="font-mono">{health.data?.version}</span>
                </div>
                <div class="flex justify-between text-xs text-muted-foreground">
                  <span>Last Check:</span>
                  <span>{formatDistanceToNow(new Date(health.data?.timestamp || ''), { addSuffix: true })}</span>
                </div>
              </div>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
};

// Metric Card Component
interface MetricCardProps {
  title: string;
  value: number;
  subtitle: string;
  trend: string;
  icon: string;
  color: string;
}

const MetricCard: Component<MetricCardProps> = (props) => {
  return (
    <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6 hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-[6px_6px_0px_0px_rgba(0,0,0,1)] transition-all">
      <div class="flex items-start justify-between mb-4">
        <div class={`w-12 h-12 ${props.color} border-2 border-black flex items-center justify-center text-2xl`}>
          {props.icon}
        </div>
      </div>
      <div class="text-4xl font-black mb-2">{props.value.toLocaleString()}</div>
      <div class="text-sm font-bold text-muted-foreground mb-1">{props.title}</div>
      <div class="text-sm font-bold">{props.subtitle}</div>
      <div class="text-xs text-muted-foreground mt-1">{props.trend}</div>
    </div>
  );
};

// Stat Bar Component
interface StatBarProps {
  label: string;
  value: number;
  total: number;
  color: string;
}

const StatBar: Component<StatBarProps> = (props) => {
  const percentage = () => props.total > 0 ? (props.value / props.total) * 100 : 0;

  return (
    <div>
      <div class="flex justify-between text-sm mb-1">
        <span class="font-bold">{props.label}</span>
        <span class="font-bold">{props.value} ({percentage().toFixed(0)}%)</span>
      </div>
      <div class="h-2 bg-muted border-2 border-black">
        <div class={`h-full ${props.color} border-r-2 border-black transition-all`} style={{ width: `${percentage()}%` }} />
      </div>
    </div>
  );
};

// Helper Functions
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}

function getActionIcon(action: string): string {
  const actionLower = action.toLowerCase();
  if (actionLower.includes('create')) return '➕';
  if (actionLower.includes('update') || actionLower.includes('edit')) return '✏️';
  if (actionLower.includes('delete')) return '🗑️';
  if (actionLower.includes('login') || actionLower.includes('logout')) return '🔐';
  if (actionLower.includes('upload')) return '📤';
  if (actionLower.includes('download')) return '📥';
  if (actionLower.includes('complete')) return '✅';
  return '📋';
}

export default AdminDashboard;
