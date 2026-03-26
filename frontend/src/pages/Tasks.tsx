import { Component, createSignal, For, Show, createMemo } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { Card, CardHeader, CardTitle, CardContent, Button, Badge, Input, Spinner } from '~/components/ui';
import { TaskCard } from '~/components/crm';
import ExportButton from '~/components/ExportButton';
import { useTasks, useUpdateTask, useDeleteTask, useCompleteTask, useTaskStats, useCurrentUser } from '~/lib/hooks';
import { api } from '~/lib/api';
import { showToast } from '~/lib/toast';

const Tasks: Component = () => {
  const navigate = useNavigate();
  const [page, setPage] = createSignal(1);
  const [search, setSearch] = createSignal('');
  const [status, setStatus] = createSignal('');
  const [priority, setPriority] = createSignal('');
  const [myOnly, setMyOnly] = createSignal(false);
  const [dueTodayOnly, setDueTodayOnly] = createSignal(false);
  const [overdueOnly, setOverdueOnly] = createSignal(false);
  const [viewMode, setViewMode] = createSignal<'grid' | 'list' | 'kanban'>('grid');
  const [selectedIds, setSelectedIds] = createSignal<Set<string>>(new Set());

  const me = useCurrentUser();

  // API hooks
  const tasks = useTasks(() => ({
    page: page(),
    limit: viewMode() === 'kanban' ? 200 : 12,
    search: search() || undefined,
    status: status() || undefined,
    priority: priority() || undefined,
    assigned_to: myOnly() ? me.data?.id : undefined,
    due_today: dueTodayOnly() || undefined,
    overdue: overdueOnly() || undefined,
  }));

  const taskStats = useTaskStats();
  const updateTask = useUpdateTask();
  const deleteTask = useDeleteTask();
  const completeTask = useCompleteTask();

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
    setSearch('');
    setDueTodayOnly(false);
    setOverdueOnly(false);
    setPage(1);
  };

  const toggleSelected = (id: string, next: boolean) => {
    setSelectedIds((prev) => {
      const copy = new Set(prev);
      if (next) copy.add(id);
      else copy.delete(id);
      return copy;
    });
  };

  const clearSelection = () => setSelectedIds(new Set());

  const bulkAction = async (action: 'complete' | 'cancel' | 'delete') => {
    const ids = Array.from(selectedIds());
    if (ids.length === 0) return;

    if (action === 'delete') {
      const ok = confirm(`Delete ${ids.length} task(s)? This cannot be undone.`);
      if (!ok) return;
    }

    try {
      for (const id of ids) {
        if (action === 'complete') await api.completeTask(id);
        if (action === 'cancel') await api.updateTask(id, { status: 'cancelled' });
        if (action === 'delete') await api.deleteTask(id);
      }
      showToast('success', 'Bulk action complete', `Updated ${ids.length} task(s)`);
      clearSelection();
      await tasks.refetch();
      await taskStats.refetch();
    } catch (e: any) {
      showToast('error', 'Bulk action failed', e?.message || 'Please try again');
    }
  };

  const onDragStartTask = (ev: DragEvent, id: string) => {
    try {
      ev.dataTransfer?.setData('text/plain', id);
      ev.dataTransfer?.setData('application/x-task-id', id);
      ev.dataTransfer!.effectAllowed = 'move';
    } catch {
      // ignore
    }
  };

  const onDropToStatus = (ev: DragEvent, newStatus: string) => {
    ev.preventDefault();
    const id = ev.dataTransfer?.getData('application/x-task-id') || ev.dataTransfer?.getData('text/plain');
    if (!id) return;
    handleQuickStatusUpdate(id, newStatus);
  };

  const tasksByStatus = createMemo(() => {
    const rows = tasks.data?.data || [];
    const map: Record<string, any[]> = { todo: [], in_progress: [], done: [], cancelled: [] };
    for (const t of rows) {
      (map[t.status] ??= []).push(t);
    }
    return map;
  });

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
            variant="secondary"
            size="md"
            onClick={() => {
              setMyOnly((v) => !v);
              setPage(1);
            }}
          >
            📋 {myOnly() ? 'All Tasks' : 'My Tasks'}
          </Button>
          <Button 
            variant="primary" 
            size="lg"
            onClick={() => navigate('/tasks/new')}
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
            variant={status() === 'todo' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => handleFilterChange('status', 'todo')}
          >
            Todo
          </Button>
          <Button
            variant={status() === 'in_progress' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => handleFilterChange('status', 'in_progress')}
          >
            In Progress
          </Button>
          <Button
            variant={status() === 'done' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => handleFilterChange('status', 'done')}
          >
            Done
          </Button>
          <Button
            variant={status() === 'cancelled' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => handleFilterChange('status', 'cancelled')}
          >
            Cancelled
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
          <Button
            variant={priority() === 'urgent' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => handleFilterChange('priority', 'urgent')}
          >
            🔥 Urgent
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
            variant={viewMode() === 'kanban' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => setViewMode('kanban')}
          >
            Kanban
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

      {/* Due filters */}
      <div class="flex flex-wrap gap-2 mb-6">
        <Button
          variant={dueTodayOnly() ? 'primary' : 'secondary'}
          size="sm"
          onClick={() => {
            setDueTodayOnly((v) => !v);
            if (!dueTodayOnly()) setOverdueOnly(false);
            setPage(1);
          }}
        >
          Due today
        </Button>
        <Button
          variant={overdueOnly() ? 'primary' : 'secondary'}
          size="sm"
          onClick={() => {
            setOverdueOnly((v) => !v);
            if (!overdueOnly()) setDueTodayOnly(false);
            setPage(1);
          }}
        >
          Overdue
        </Button>
      </div>

      {/* Bulk actions */}
      <Show when={selectedIds().size > 0}>
        <Card class="p-4 mb-6">
          <div class="flex flex-wrap items-center justify-between gap-3">
            <div class="font-bold">
              Selected: {selectedIds().size}
            </div>
            <div class="flex flex-wrap gap-2">
              <Button variant="primary" size="sm" onClick={() => bulkAction('complete')}>
                Complete
              </Button>
              <Button variant="secondary" size="sm" onClick={() => bulkAction('cancel')}>
                Cancel
              </Button>
              <Button variant="secondary" size="sm" class="bg-red-500 hover:bg-red-600" onClick={() => bulkAction('delete')}>
                Delete
              </Button>
              <Button variant="secondary" size="sm" onClick={clearSelection}>
                Clear
              </Button>
            </div>
          </div>
        </Card>
      </Show>

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
        <Show when={viewMode() !== 'kanban'}>
          <div class={`grid gap-6 ${viewMode() === 'grid' ? 'grid-cols-1 md:grid-cols-2 lg:grid-cols-3' : 'grid-cols-1'}`}>
            <For each={tasks.data?.data || []}>
              {(task: any) => (
                <div class="relative group">
                  <div class="absolute top-2 left-2 z-10">
                    <input
                      type="checkbox"
                      checked={selectedIds().has(task.id)}
                      onClick={(e) => e.stopPropagation()}
                      onChange={(e: any) => toggleSelected(task.id, !!e.currentTarget.checked)}
                    />
                  </div>

                  <TaskCard
                    title={task.title}
                    description={task.description || ''}
                    priority={task.priority}
                    dueDate={task.due_date ? new Date(task.due_date).toLocaleDateString() : 'No due date'}
                    status={task.status}
                    onClick={() => navigate(`/tasks/${task.id}`)}
                  />
                  
                  {/* Quick Action Buttons */}
                  <div class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity flex gap-1">
                    <Show when={task.status !== 'done'}>
                      <Button
                        variant="primary"
                        size="sm"
                        class="bg-green-500 hover:bg-green-600"
                        onClick={() => completeTask.mutate(task.id)}
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
                      onClick={() => navigate(`/tasks/${task.id}/edit`)}
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
                        task.status === 'done' ? 'success' :
                        task.status === 'in_progress' ? 'primary' :
                        task.status === 'todo' ? 'warning' :
                        task.status === 'cancelled' ? 'destructive' : 'secondary'
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
        </Show>

        <Show when={viewMode() === 'kanban'}>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div
              class="border-2 border-neutral-black bg-neutral-white"
              onDragOver={(e) => e.preventDefault()}
              onDrop={(e) => onDropToStatus(e as any, 'todo')}
            >
              <div class="p-3 font-heading font-black uppercase border-b-2 border-neutral-black">Todo</div>
              <div class="p-3 space-y-3">
                <For each={tasksByStatus().todo}>
                  {(task: any) => (
                    <div class="relative">
                      <div class="absolute top-2 left-2 z-10">
                        <input
                          type="checkbox"
                          checked={selectedIds().has(task.id)}
                          onClick={(e) => e.stopPropagation()}
                          onChange={(e: any) => toggleSelected(task.id, !!e.currentTarget.checked)}
                        />
                      </div>
                      <div draggable onDragStart={(e) => onDragStartTask(e as any, task.id)}>
                        <TaskCard
                          title={task.title}
                          description={task.description || ''}
                          priority={task.priority}
                          dueDate={task.due_date ? new Date(task.due_date).toLocaleDateString() : 'No due date'}
                          status={task.status}
                          onClick={() => navigate(`/tasks/${task.id}`)}
                        />
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </div>

            <div
              class="border-2 border-neutral-black bg-neutral-white"
              onDragOver={(e) => e.preventDefault()}
              onDrop={(e) => onDropToStatus(e as any, 'in_progress')}
            >
              <div class="p-3 font-heading font-black uppercase border-b-2 border-neutral-black">In Progress</div>
              <div class="p-3 space-y-3">
                <For each={tasksByStatus().in_progress}>
                  {(task: any) => (
                    <div class="relative">
                      <div class="absolute top-2 left-2 z-10">
                        <input
                          type="checkbox"
                          checked={selectedIds().has(task.id)}
                          onClick={(e) => e.stopPropagation()}
                          onChange={(e: any) => toggleSelected(task.id, !!e.currentTarget.checked)}
                        />
                      </div>
                      <div draggable onDragStart={(e) => onDragStartTask(e as any, task.id)}>
                        <TaskCard
                          title={task.title}
                          description={task.description || ''}
                          priority={task.priority}
                          dueDate={task.due_date ? new Date(task.due_date).toLocaleDateString() : 'No due date'}
                          status={task.status}
                          onClick={() => navigate(`/tasks/${task.id}`)}
                        />
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </div>

            <div
              class="border-2 border-neutral-black bg-neutral-white"
              onDragOver={(e) => e.preventDefault()}
              onDrop={(e) => onDropToStatus(e as any, 'done')}
            >
              <div class="p-3 font-heading font-black uppercase border-b-2 border-neutral-black">Done</div>
              <div class="p-3 space-y-3">
                <For each={tasksByStatus().done}>
                  {(task: any) => (
                    <div class="relative">
                      <div class="absolute top-2 left-2 z-10">
                        <input
                          type="checkbox"
                          checked={selectedIds().has(task.id)}
                          onClick={(e) => e.stopPropagation()}
                          onChange={(e: any) => toggleSelected(task.id, !!e.currentTarget.checked)}
                        />
                      </div>
                      <div draggable onDragStart={(e) => onDragStartTask(e as any, task.id)}>
                        <TaskCard
                          title={task.title}
                          description={task.description || ''}
                          priority={task.priority}
                          dueDate={task.due_date ? new Date(task.due_date).toLocaleDateString() : 'No due date'}
                          status={task.status}
                          onClick={() => navigate(`/tasks/${task.id}`)}
                        />
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </div>
          </div>
        </Show>

        {/* Pagination */}
        <Show when={tasks.data?.pagination && viewMode() !== 'kanban'}>
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

    </div>
  );
};

export default Tasks;