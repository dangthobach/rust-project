import { Component, createSignal, For, Show, createMemo } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent, Button, Badge, Input, Spinner } from '~/components/ui';
import { TaskCard } from '~/components/crm';
import ExportButton from '~/components/ExportButton';
import { useTasks, useCreateTask, useUpdateTask, useDeleteTask, useTaskStats, useMyTasks } from '~/lib/hooks';
import { api } from '~/lib/api';
import { showToast } from '~/lib/toast';

const Tasks: Component = () => {
  const [page, setPage] = createSignal(1);
  const [search, setSearch] = createSignal('');
  const [status, setStatus] = createSignal('');
  const [priority, setPriority] = createSignal('');
  const [assignedToMe, setAssignedToMe] = createSignal(false);
  const [showCreateForm, setShowCreateForm] = createSignal(false);
  const [viewMode, setViewMode] = createSignal<'grid' | 'list'>('grid');

  // Form state for new task
  const [newTask, setNewTask] = createSignal({
    title: '',
    description: '',
    status: 'pending' as const,
    priority: 'medium' as const,
    due_date: '',
    client_id: '',
    assigned_to: '',
    created_by: '',
  });

  // API hooks
  const tasks = useTasks(() => ({
    page: page(),
    limit: 12,
    search: search() || undefined,
    status: status() || undefined,
    priority: priority() || undefined,
    assigned_to: assignedToMe() ? 'me' : undefined,
  }));

  const myTasks = useMyTasks(() => ({
    page: 1,
    limit: 5,
    status: 'pending',
  }));

  const taskStats = useTaskStats();
  const createTask = useCreateTask();
  const updateTask = useUpdateTask();
  const deleteTask = useDeleteTask();

  const [isExporting, setIsExporting] = createSignal(false);

  const handleExport = async (format: 'csv' | 'json' | 'pdf') => {
    setIsExporting(true);
    try {
      const blob = await api.exportTasks(format, {
        status: status() || undefined,
      });
      
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `tasks_export.${format}`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
      
      showToast('success', 'Export Successful', `Tasks exported as ${format.toUpperCase()}`);
    } catch (error) {
      showToast('error', 'Export Failed', 'Failed to export tasks');
    } finally {
      setIsExporting(false);
    }
  };

  // Quick stats from API
  const stats = createMemo(() => taskStats.data || {
    total: 0,
    completed: 0,
    pending: 0,
    inProgress: 0,
    overdue: 0,
    dueToday: 0,
    byPriority: { high: 0, medium: 0, low: 0 }
  });

  const handleCreateTask = () => {
    const task = newTask();
    if (!task.title) return;

    createTask.mutate({
      ...task,
      due_date: task.due_date || undefined,
    }, {
      onSuccess: () => {
        setNewTask({
          title: '',
          description: '',
          status: 'pending',
          priority: 'medium',
          due_date: '',
          client_id: '',
          assigned_to: '',
          created_by: '',
        });
        setShowCreateForm(false);
      },
    });
  };

  const handleUpdateTask = (taskId: string, updates: any) => {
    updateTask.mutate({ id: taskId, updates });
  };

  const handleDeleteTask = (taskId: string) => {
    if (confirm('Are you sure you want to delete this task?')) {
      deleteTask.mutate(taskId);
    }
  };

  const handleQuickStatusUpdate = (taskId: string, newStatus: string) => {
    handleUpdateTask(taskId, { status: newStatus });
  };

  const handleSearch = (query: string) => {
    setSearch(query);
    setPage(1);
  };

  const handleFilterChange = (filterType: string, value: string) => {
    if (filterType === 'status') setStatus(value);
    if (filterType === 'priority') setPriority(value);
    setPage(1);
  };

  const resetFilters = () => {
    setStatus('');
    setPriority('');
    setAssignedToMe(false);
    setSearch('');
    setPage(1);
  };

  return (
    <div>
      {/* Header */}
      <div class="flex items-center justify-between mb-8">
        <div>
          <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">
            Task Management
          </h1>
          <p class="text-neutral-darkGray text-lg">
            Organize and track all your tasks efficiently
          </p>
        </div>
        
        <div class="flex gap-3">
          <ExportButton 
            onExport={handleExport}
            isExporting={isExporting()}
            label="Export"
          />
          <Button 
            variant={assignedToMe() ? 'primary' : 'secondary'}
            size="md"
            onClick={() => setAssignedToMe(!assignedToMe())}
          >
            📋 My Tasks
          </Button>
          <Button 
            variant="primary" 
            size="lg"
            onClick={() => setShowCreateForm(true)}
          >
            ➕ New Task
          </Button>
        </div>
      </div>

      {/* Stats Cards */}
      <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4 mb-8">
        <Card class="text-center p-4">
          <div class="text-2xl font-heading font-black">{stats().total}</div>
          <div class="text-xs font-bold uppercase text-neutral-darkGray">Total</div>
        </Card>
        <Card class="text-center p-4 bg-green-100 border-green-500">
          <div class="text-2xl font-heading font-black text-green-700">{stats().completed}</div>
          <div class="text-xs font-bold uppercase text-green-600">Completed</div>
        </Card>
        <Card class="text-center p-4 bg-yellow-100 border-yellow-500">
          <div class="text-2xl font-heading font-black text-yellow-700">{stats().pending}</div>
          <div class="text-xs font-bold uppercase text-yellow-600">Pending</div>
        </Card>
        <Card class="text-center p-4 bg-blue-100 border-blue-500">
          <div class="text-2xl font-heading font-black text-blue-700">{stats().inProgress}</div>
          <div class="text-xs font-bold uppercase text-blue-600">In Progress</div>
        </Card>
        <Card class="text-center p-4 bg-red-100 border-red-500">
          <div class="text-2xl font-heading font-black text-red-700">{stats().overdue}</div>
          <div class="text-xs font-bold uppercase text-red-600">Overdue</div>
        </Card>
        <Card class="text-center p-4 bg-purple-100 border-purple-500">
          <div class="text-2xl font-heading font-black text-purple-700">{stats().dueToday}</div>
          <div class="text-xs font-bold uppercase text-purple-600">Due Today</div>
        </Card>
      </div>

      {/* Filters and View Controls */}
      <div class="flex flex-col lg:flex-row gap-4 mb-6">
        {/* Search */}
        <div class="flex-1">
          <Input
            type="text"
            placeholder="Search tasks..."
            value={search()}
            onInput={(e: any) => handleSearch(e.currentTarget.value)}
          />
        </div>
        
        {/* Status Filter */}
        <div class="flex gap-2">
          <Button
            variant={status() === '' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => handleFilterChange('status', '')}
          >
            All Status
          </Button>
          <Button
            variant={status() === 'pending' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => handleFilterChange('status', 'pending')}
          >
            Pending
          </Button>
          <Button
            variant={status() === 'in_progress' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => handleFilterChange('status', 'in_progress')}
          >
            In Progress
          </Button>
          <Button
            variant={status() === 'completed' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => handleFilterChange('status', 'completed')}
          >
            Completed
          </Button>
        </div>

        {/* Priority Filter */}
        <div class="flex gap-2">
          <Button
            variant={priority() === '' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => handleFilterChange('priority', '')}
          >
            All Priority
          </Button>
          <Button
            variant={priority() === 'high' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => handleFilterChange('priority', 'high')}
          >
            🔴 High
          </Button>
          <Button
            variant={priority() === 'medium' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => handleFilterChange('priority', 'medium')}
          >
            🟡 Medium
          </Button>
          <Button
            variant={priority() === 'low' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => handleFilterChange('priority', 'low')}
          >
            🟢 Low
          </Button>
        </div>

        {/* View Mode & Reset */}
        <div class="flex gap-2">
          <Button
            variant={viewMode() === 'grid' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => setViewMode('grid')}
          >
            📊 Grid
          </Button>
          <Button
            variant={viewMode() === 'list' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => setViewMode('list')}
          >
            📋 List
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={resetFilters}
          >
            🔄 Reset
          </Button>
        </div>
      </div>

      {/* Task List */}
      <Show when={tasks.isPending}>
        <div class="flex justify-center p-8">
          <Spinner />
        </div>
      </Show>

      <Show when={tasks.isError}>
        <Card class="p-6 bg-red-100 border-red-500">
          <p class="text-red-700 font-bold">
            Error loading tasks: {tasks.error?.message}
          </p>
          <Button 
            variant="secondary" 
            size="sm" 
            class="mt-4"
            onClick={() => tasks.refetch()}
          >
            Retry
          </Button>
        </Card>
      </Show>

      <Show when={tasks.data}>
        <div class={`grid gap-6 ${viewMode() === 'grid' ? 'grid-cols-1 md:grid-cols-2 lg:grid-cols-3' : 'grid-cols-1'}`}>
          <For each={tasks.data?.data || []}>
            {(task: any) => (
              <div class="relative group">
                <TaskCard
                  title={task.title}
                  description={task.description || ''}
                  priority={task.priority}
                  dueDate={task.due_date ? new Date(task.due_date).toLocaleDateString() : 'No due date'}
                  status={task.status}
                />
                
                {/* Quick Action Buttons */}
                <div class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity flex gap-1">
                  <Show when={task.status !== 'completed'}>
                    <Button
                      variant="primary"
                      size="sm"
                      class="bg-green-500 hover:bg-green-600"
                      onClick={() => handleQuickStatusUpdate(task.id, 'completed')}
                      title="Mark as completed"
                    >
                      ✅
                    </Button>
                  </Show>
                  
                  <Show when={task.status !== 'in_progress'}>
                    <Button
                      variant="primary"
                      size="sm"
                      class="bg-blue-500 hover:bg-blue-600"
                      onClick={() => handleQuickStatusUpdate(task.id, 'in_progress')}
                      title="Mark as in progress"
                    >
                      ▶️
                    </Button>
                  </Show>

                  <Button
                    variant="primary"
                    size="sm"
                    onClick={() => console.log('Edit task:', task.id)}
                    title="Edit task"
                  >
                    ✏️
                  </Button>
                  
                  <Button
                    variant="secondary"
                    size="sm"
                    class="bg-red-500 hover:bg-red-600"
                    onClick={() => handleDeleteTask(task.id)}
                    disabled={deleteTask.isPending}
                    title="Delete task"
                  >
                    🗑️
                  </Button>
                </div>

                {/* Status Badge */}
                <div class="absolute bottom-2 left-2">
                  <Badge 
                    variant={
                      task.status === 'completed' ? 'success' :
                      task.status === 'in_progress' ? 'primary' :
                      task.status === 'pending' ? 'warning' : 'secondary'
                    }
                    class="text-xs"
                  >
                    {task.status.replace('_', ' ').toUpperCase()}
                  </Badge>
                </div>
              </div>
            )}
          </For>
        </div>

        {/* Pagination */}
        <Show when={tasks.data?.pagination}>
          <div class="flex items-center justify-between mt-8">
            <p class="text-sm text-neutral-darkGray">
              Showing {tasks.data?.data?.length || 0} of {tasks.data?.pagination?.total || 0} tasks
            </p>
            
            <div class="flex gap-2">
              <Button
                variant="secondary"
                disabled={!tasks.data?.pagination?.has_prev}
                onClick={() => setPage(p => Math.max(1, p - 1))}
              >
                ← Previous
              </Button>
              
              <Badge variant="primary" class="px-4 py-2">
                Page {page()} of {tasks.data?.pagination?.total_pages || 1}
              </Badge>
              
              <Button
                variant="secondary"
                disabled={!tasks.data?.pagination?.has_next}
                onClick={() => setPage(p => p + 1)}
              >
                Next →
              </Button>
            </div>
          </div>
        </Show>
      </Show>

      {/* Create Task Modal */}
      <Show when={showCreateForm()}>
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <Card class="w-full max-w-2xl mx-4 max-h-[90vh] overflow-y-auto">
            <CardHeader>
              <CardTitle>Create New Task</CardTitle>
            </CardHeader>
            <CardContent>
              <div class="space-y-4">
                <div>
                  <label class="block font-bold uppercase text-sm mb-2">
                    Title *
                  </label>
                  <Input
                    type="text"
                    placeholder="Task title"
                    value={newTask().title}
                    onInput={(e: any) => setNewTask(t => ({ ...t, title: e.currentTarget.value }))}
                    required
                  />
                </div>

                <div>
                  <label class="block font-bold uppercase text-sm mb-2">
                    Description
                  </label>
                  <textarea
                    class="w-full p-3 border-3 border-black font-mono"
                    rows="3"
                    placeholder="Task description..."
                    value={newTask().description}
                    onInput={(e: any) => setNewTask(t => ({ ...t, description: e.currentTarget.value }))}
                  />
                </div>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div>
                    <label class="block font-bold uppercase text-sm mb-2">
                      Status
                    </label>
                    <select
                      class="w-full p-3 border-3 border-black font-mono"
                      value={newTask().status}
                      onChange={(e: any) => setNewTask(t => ({ ...t, status: e.currentTarget.value }))}
                    >
                      <option value="pending">Pending</option>
                      <option value="in_progress">In Progress</option>
                      <option value="completed">Completed</option>
                    </select>
                  </div>

                  <div>
                    <label class="block font-bold uppercase text-sm mb-2">
                      Priority
                    </label>
                    <select
                      class="w-full p-3 border-3 border-black font-mono"
                      value={newTask().priority}
                      onChange={(e: any) => setNewTask(t => ({ ...t, priority: e.currentTarget.value }))}
                    >
                      <option value="low">Low</option>
                      <option value="medium">Medium</option>
                      <option value="high">High</option>
                    </select>
                  </div>

                  <div>
                    <label class="block font-bold uppercase text-sm mb-2">
                      Due Date
                    </label>
                    <Input
                      type="date"
                      value={newTask().due_date}
                      onInput={(e: any) => setNewTask(t => ({ ...t, due_date: e.currentTarget.value }))}
                    />
                  </div>
                </div>

                <Show when={createTask.isError}>
                  <div class="p-3 bg-red-100 border-3 border-red-500 text-red-700 text-sm font-bold">
                    {createTask.error?.message}
                  </div>
                </Show>

                <div class="flex gap-3 pt-4">
                  <Button
                    variant="secondary"
                    fullWidth
                    onClick={() => setShowCreateForm(false)}
                    disabled={createTask.isPending}
                  >
                    Cancel
                  </Button>
                  <Button
                    variant="primary"
                    fullWidth
                    onClick={handleCreateTask}
                    disabled={createTask.isPending}
                  >
                    <Show when={createTask.isPending} fallback="Create Task">
                      <Spinner class="inline-block mr-2" />
                      Creating...
                    </Show>
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </Show>
    </div>
  );
};

export default Tasks;