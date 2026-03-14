/**
 * Analytics Page
 * Advanced analytics with Chart.js visualizations and date range filtering
 */

import { Component, Show, createSignal, createEffect } from 'solid-js';
import { useAnalytics } from '../lib/hooks';
import { Line, Bar, Doughnut } from 'solid-chartjs';
import { Chart, Title, Tooltip, Legend, Colors, LineElement, BarElement, PointElement, ArcElement, CategoryScale, LinearScale } from 'chart.js';

// Register Chart.js components
Chart.register(Title, Tooltip, Legend, Colors, LineElement, BarElement, PointElement, ArcElement, CategoryScale, LinearScale);

const Analytics: Component = () => {
  // Date range state (last 30 days by default)
  const [startDate, setStartDate] = createSignal(getDefaultStartDate());
  const [endDate, setEndDate] = createSignal(getDefaultEndDate());

  // Fetch analytics data
  const { userActivity, taskCompletion, clientEngagement, storageUsage, isLoading, isError } = 
    useAnalytics(startDate(), endDate());

  // Chart options
  const commonOptions = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        position: 'bottom' as const,
        labels: {
          font: { weight: 'bold' as const },
          padding: 15,
        },
      },
      tooltip: {
        backgroundColor: '#000',
        titleFont: { weight: 'bold' as const },
        bodyFont: { weight: 'bold' as const },
        padding: 12,
        borderColor: '#000',
        borderWidth: 2,
      },
    },
  };

  // User Activity Chart Data
  const userActivityChartData = () => {
    if (!userActivity.data) return null;
    return {
      labels: userActivity.data.daily_activity.map(d => new Date(d.date).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })),
      datasets: [{
        label: 'Daily Activities',
        data: userActivity.data.daily_activity.map(d => d.count),
        borderColor: '#8b5cf6',
        backgroundColor: 'rgba(139, 92, 246, 0.1)',
        borderWidth: 3,
        fill: true,
        tension: 0.4,
      }],
    };
  };

  // Task Completion Chart Data
  const taskCompletionChartData = () => {
    if (!taskCompletion.data) return null;
    return {
      labels: taskCompletion.data.daily_completions.map(d => new Date(d.date).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })),
      datasets: [
        {
          label: 'Tasks Created',
          data: taskCompletion.data.daily_completions.map(d => d.created_count),
          backgroundColor: '#60a5fa',
          borderColor: '#000',
          borderWidth: 2,
        },
        {
          label: 'Tasks Completed',
          data: taskCompletion.data.daily_completions.map(d => d.completed_count),
          backgroundColor: '#34d399',
          borderColor: '#000',
          borderWidth: 2,
        },
      ],
    };
  };

  // Task Status Doughnut Chart
  const taskStatusChartData = () => {
    if (!taskCompletion.data) return null;
    return {
      labels: taskCompletion.data.completion_by_status.map(s => s.status),
      datasets: [{
        data: taskCompletion.data.completion_by_status.map(s => s.count),
        backgroundColor: ['#fbbf24', '#60a5fa', '#34d399', '#9ca3af'],
        borderColor: '#000',
        borderWidth: 3,
      }],
    };
  };

  // Client Engagement Chart Data
  const clientEngagementChartData = () => {
    if (!clientEngagement.data) return null;
    return {
      labels: clientEngagement.data.new_clients_trend.map(d => new Date(d.date).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })),
      datasets: [{
        label: 'New Clients',
        data: clientEngagement.data.new_clients_trend.map(d => d.new_clients),
        borderColor: '#a78bfa',
        backgroundColor: 'rgba(167, 139, 250, 0.1)',
        borderWidth: 3,
        fill: true,
        tension: 0.4,
      }],
    };
  };

  return (
    <div class="container mx-auto p-6 space-y-6">
      {/* Page Header */}
      <div class="flex items-center justify-between flex-wrap gap-4">
        <div>
          <h1 class="text-4xl font-black tracking-tight">📊 Analytics</h1>
          <p class="text-muted-foreground mt-1">Advanced insights and trends</p>
        </div>

        {/* Date Range Picker */}
        <div class="flex items-center gap-3 bg-card border-4 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] p-4">
          <div class="flex flex-col gap-1">
            <label class="text-xs font-bold text-muted-foreground">FROM</label>
            <input
              type="date"
              value={startDate()}
              onInput={(e) => setStartDate(e.currentTarget.value)}
              class="px-3 py-2 border-2 border-black font-bold focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-bold text-muted-foreground">TO</label>
            <input
              type="date"
              value={endDate()}
              onInput={(e) => setEndDate(e.currentTarget.value)}
              class="px-3 py-2 border-2 border-black font-bold focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>
          <button
            onClick={() => {
              setStartDate(getDefaultStartDate());
              setEndDate(getDefaultEndDate());
            }}
            class="px-4 py-2 bg-secondary border-2 border-black font-bold hover:translate-x-[2px] hover:translate-y-[2px] transition-all mt-5"
          >
            Reset
          </button>
        </div>
      </div>

      <Show when={isLoading}>
        <div class="flex items-center justify-center h-96">
          <div class="animate-spin rounded-full h-16 w-16 border-t-4 border-b-4 border-primary"></div>
        </div>
      </Show>

      <Show when={isError}>
        <div class="bg-destructive/10 border-4 border-destructive p-6 text-center">
          <p class="text-destructive font-bold text-lg">Failed to load analytics data</p>
          <p class="text-sm mt-2">Please check your connection and try again</p>
        </div>
      </Show>

      <Show when={!isLoading && !isError}>
        {/* Summary Cards */}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <Show when={userActivity.data}>
            <SummaryCard
              title="Total Activities"
              value={userActivity.data!.total_activities}
              subtitle={`${userActivity.data!.unique_active_users} active users`}
              icon="📈"
              color="bg-purple-400"
            />
          </Show>

          <Show when={taskCompletion.data}>
            <SummaryCard
              title="Completion Rate"
              value={`${taskCompletion.data!.completion_rate.toFixed(1)}%`}
              subtitle={`${taskCompletion.data!.completed_tasks} / ${taskCompletion.data!.total_tasks} tasks`}
              icon="✅"
              color="bg-green-400"
            />
          </Show>

          <Show when={clientEngagement.data}>
            <SummaryCard
              title="Engagement Rate"
              value={`${clientEngagement.data!.engagement_rate.toFixed(1)}%`}
              subtitle={`${clientEngagement.data!.active_clients} / ${clientEngagement.data!.total_clients} active`}
              icon="🎯"
              color="bg-blue-400"
            />
          </Show>

          <Show when={storageUsage.data}>
            <SummaryCard
              title="Storage Used"
              value={`${storageUsage.data!.total_size_gb.toFixed(2)} GB`}
              subtitle={`${storageUsage.data!.total_files} files`}
              icon="💾"
              color="bg-orange-400"
            />
          </Show>
        </div>

        {/* Charts Grid */}
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* User Activity Trend */}
          <Show when={userActivityChartData()}>
            <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
              <h3 class="text-xl font-black mb-4">👥 User Activity Trend</h3>
              <div style={{ height: '300px' }}>
                <Line data={userActivityChartData()!} options={commonOptions} />
              </div>
            </div>
          </Show>

          {/* Task Completion Trend */}
          <Show when={taskCompletionChartData()}>
            <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
              <h3 class="text-xl font-black mb-4">📋 Task Creation vs Completion</h3>
              <div style={{ height: '300px' }}>
                <Bar data={taskCompletionChartData()!} options={commonOptions} />
              </div>
            </div>
          </Show>

          {/* Task Status Distribution */}
          <Show when={taskStatusChartData()}>
            <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
              <h3 class="text-xl font-black mb-4">📊 Task Status Distribution</h3>
              <div style={{ height: '300px' }} class="flex items-center justify-center">
                <Doughnut data={taskStatusChartData()!} options={commonOptions} />
              </div>
            </div>
          </Show>

          {/* Client Growth */}
          <Show when={clientEngagementChartData()}>
            <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
              <h3 class="text-xl font-black mb-4">🏢 Client Growth</h3>
              <div style={{ height: '300px' }}>
                <Line data={clientEngagementChartData()!} options={commonOptions} />
              </div>
            </div>
          </Show>
        </div>

        {/* Detailed Statistics */}
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Most Active Users */}
          <Show when={userActivity.data}>
            <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
              <h3 class="text-xl font-black mb-4">🌟 Most Active Users</h3>
              <div class="space-y-3">
                <Show when={userActivity.data!.most_active_users.length > 0}>
                  {userActivity.data!.most_active_users.slice(0, 5).map((user, index) => (
                    <div class="flex items-center justify-between p-3 bg-muted/30 border-2 border-black">
                      <div class="flex items-center gap-3">
                        <div class="w-8 h-8 rounded-full bg-primary border-2 border-black flex items-center justify-center font-bold text-primary-foreground text-sm">
                          {index + 1}
                        </div>
                        <span class="font-bold">{user.user_name}</span>
                      </div>
                      <span class="font-black text-lg">{user.activity_count}</span>
                    </div>
                  ))}
                </Show>
              </div>
            </div>
          </Show>

          {/* Top Clients by Tasks */}
          <Show when={clientEngagement.data}>
            <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
              <h3 class="text-xl font-black mb-4">🏆 Top Clients by Tasks</h3>
              <div class="space-y-3">
                <Show when={clientEngagement.data!.top_clients_by_tasks.length > 0}>
                  {clientEngagement.data!.top_clients_by_tasks.slice(0, 5).map((client, index) => (
                    <div class="flex items-center justify-between p-3 bg-muted/30 border-2 border-black">
                      <div class="flex items-center gap-3">
                        <div class="w-8 h-8 rounded-full bg-secondary border-2 border-black flex items-center justify-center font-bold text-sm">
                          {index + 1}
                        </div>
                        <div>
                          <div class="font-bold">{client.client_name}</div>
                          <div class="text-xs text-muted-foreground">{client.completed_tasks} completed</div>
                        </div>
                      </div>
                      <span class="font-black text-lg">{client.task_count}</span>
                    </div>
                  ))}
                </Show>
              </div>
            </div>
          </Show>
        </div>

        {/* Storage Analytics */}
        <Show when={storageUsage.data}>
          <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
            <h3 class="text-xl font-black mb-4">💾 Storage by File Type</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {storageUsage.data!.files_by_type.slice(0, 6).map((fileType) => (
                <div class="p-4 bg-muted/30 border-2 border-black">
                  <div class="flex justify-between items-start mb-2">
                    <span class="font-bold text-sm">{fileType.file_type.toUpperCase()}</span>
                    <span class="text-xs font-bold text-muted-foreground">{fileType.percentage.toFixed(1)}%</span>
                  </div>
                  <div class="text-2xl font-black mb-1">{fileType.count}</div>
                  <div class="text-xs text-muted-foreground">
                    {formatBytes(fileType.total_size_bytes)}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </Show>

        {/* Task Analytics Details */}
        <Show when={taskCompletion.data}>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
              <h3 class="text-xl font-black mb-4">⏱️ Average Completion Time</h3>
              <div class="text-center">
                <div class="text-5xl font-black mb-2">
                  {taskCompletion.data!.average_completion_time_hours.toFixed(1)}
                </div>
                <div class="text-lg font-bold text-muted-foreground">Hours</div>
              </div>
            </div>

            <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
              <h3 class="text-xl font-black mb-4">📌 Tasks by Priority</h3>
              <div class="space-y-2">
                {taskCompletion.data!.completion_by_priority.map((priority) => (
                  <div class="flex items-center justify-between p-2 bg-muted/30 border-2 border-black">
                    <span class="font-bold capitalize">{priority.priority}</span>
                    <div class="text-right">
                      <div class="font-black">{priority.count}</div>
                      <div class="text-xs text-muted-foreground">{priority.percentage.toFixed(1)}%</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </Show>
      </Show>
    </div>
  );
};

// Summary Card Component
interface SummaryCardProps {
  title: string;
  value: string | number;
  subtitle: string;
  icon: string;
  color: string;
}

const SummaryCard: Component<SummaryCardProps> = (props) => {
  return (
    <div class="bg-card border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6 hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-[6px_6px_0px_0px_rgba(0,0,0,1)] transition-all">
      <div class={`w-12 h-12 ${props.color} border-2 border-black flex items-center justify-center text-2xl mb-4`}>
        {props.icon}
      </div>
      <div class="text-3xl font-black mb-2">{props.value}</div>
      <div class="text-sm font-bold text-muted-foreground">{props.title}</div>
      <div class="text-xs text-muted-foreground mt-1">{props.subtitle}</div>
    </div>
  );
};

// Helper Functions
function getDefaultStartDate(): string {
  const date = new Date();
  date.setDate(date.getDate() - 30);
  return date.toISOString().split('T')[0];
}

function getDefaultEndDate(): string {
  return new Date().toISOString().split('T')[0];
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}

export default Analytics;
