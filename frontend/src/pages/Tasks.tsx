import { Component, createSignal, For, Show, createMemo, type Accessor } from 'solid-js';
import { A, useNavigate } from '@solidjs/router';
import { Card, Button, Badge, Input, Spinner } from '~/components/ui';
import { TaskCard, type TaskTagTone } from '~/components/crm';
import ExportButton from '~/components/ExportButton';
import { useTasks, useUpdateTask, useDeleteTask, useCompleteTask, useTaskStats, useCurrentUser } from '~/lib/hooks';
import { api } from '~/lib/api';
import { showToast } from '~/lib/toast';

function formatTaskDue(d: string | null | undefined): string {
  if (!d) return 'No due date';
  const date = new Date(d);
  if (Number.isNaN(date.getTime())) return d;
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' }).toUpperCase();
}

function rowIndex(i: Accessor<number> | number | undefined): number {
  if (i === undefined) return 0;
  return typeof i === 'function' ? i() : i;
}

function tagsForRow(index: number): Array<{ label: string; tone: TaskTagTone }> {
  const pools: Array<Array<{ label: string; tone: TaskTagTone }>> = [
    [
      { label: 'Development', tone: 'sky' },
      { label: 'V2.0', tone: 'lime' },
    ],
    [{ label: 'Design', tone: 'pale' }],
    [
      { label: 'Ops', tone: 'mint' },
      { label: 'V2.0', tone: 'lime' },
    ],
  ];
  return pools[index % pools.length];
}

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
    } catch (_error) {
      showToast('error', 'Export Failed', 'Failed to export tasks');
    } finally {
      setIsExporting(false);
    }
  };

  const stats = createMemo(
    () =>
      taskStats.data || {
        total: 0,
        completed: 0,
        pending: 0,
        inProgress: 0,
        overdue: 0,
        dueToday: 0,
        byPriority: { high: 0, medium: 0, low: 0 },
      },
  );

  const displayName = createMemo(() => {
    const u = me.data as { full_name?: string; username?: string; email?: string } | undefined;
    return u?.full_name || u?.username || u?.email || 'Alex Rivera';
  });

  const handleUpdateTask = (taskId: string, updates: Record<string, unknown>) => {
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
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : 'Please try again';
      showToast('error', 'Bulk action failed', msg);
    }
  };

  const onDragStartTask = (ev: DragEvent, id: string) => {
    try {
      ev.dataTransfer?.setData('text/plain', id);
      ev.dataTransfer?.setData('application/x-task-id', id);
      if (ev.dataTransfer) ev.dataTransfer.effectAllowed = 'move';
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
    const map: Record<string, unknown[]> = { todo: [], in_progress: [], done: [], cancelled: [] };
    for (const t of rows) {
      (map[(t as { status: string }).status] ??= []).push(t);
    }
    return map as Record<string, any[]>;
  });

  const statusBtn = (active: boolean) =>
    [
      'border-3 border-black px-3 py-2 font-heading text-xs font-black uppercase tracking-wide shadow-brutal-sm transition-all',
      active ? 'bg-black text-white -translate-x-0.5 -translate-y-0.5' : 'bg-white text-black hover:-translate-x-0.5 hover:-translate-y-0.5 hover:bg-ledger-pale',
    ].join(' ');

  return (
    <div class="font-body">
      {/* Page header — Task Management */}
      <div class="mb-8 flex flex-col gap-4 lg:flex-row lg:items-center lg:gap-6">
        <h1 class="shrink-0 font-heading text-2xl font-black uppercase tracking-tight text-black sm:text-3xl">
          Task Management
        </h1>

        <div class="relative min-w-0 flex-1">
          <span class="pointer-events-none absolute left-3 top-1/2 z-10 -translate-y-1/2 text-lg" aria-hidden="true">
            🔍
          </span>
          <Input
            type="text"
            placeholder="Search tasks..."
            value={search()}
            onInput={(e) => handleSearch(e.currentTarget.value)}
            class="!pl-10 font-body"
          />
        </div>

        <div class="flex shrink-0 flex-wrap items-center justify-end gap-3">
          <A
            href="/notifications"
            class="relative inline-flex h-11 w-11 items-center justify-center border-3 border-black bg-white text-xl shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5"
            aria-label="Notifications"
          >
            🔔
            <span class="absolute right-1 top-1 h-2 w-2 rounded-full bg-red-500 ring-2 ring-black" />
          </A>
          <A
            href="/profile"
            class="inline-flex h-11 w-11 items-center justify-center border-3 border-black bg-white text-xl shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5"
            aria-label="Settings"
          >
            ⚙️
          </A>
          <div class="hidden h-8 w-[3px] bg-black sm:block" />
          <div class="flex items-center gap-3 border-3 border-black bg-white px-3 py-2 shadow-brutal-sm">
            <div class="hidden text-right sm:block">
              <div class="font-heading text-xs font-black uppercase leading-tight text-black">
                {displayName()}
              </div>
              <div class="font-heading text-[10px] font-bold uppercase text-neutral-darkGray">Admin access</div>
            </div>
            <div
              class="flex h-10 w-10 shrink-0 items-center justify-center border-2 border-black bg-ledger-pale font-heading text-xs font-black shadow-brutal-sm"
              aria-hidden="true"
            >
              AR
            </div>
          </div>
        </div>
      </div>

      {/* Stat cards — compact neon accents */}
      <div class="mb-6 grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
        <div class="border-3 border-black bg-white p-3 shadow-brutal-sm">
          <div class="mb-1 flex items-center justify-between text-ledger-sky">
            <span class="text-xl" aria-hidden="true">
              📊
            </span>
          </div>
          <div class="font-heading text-2xl font-black tabular-nums">{stats().total}</div>
          <div class="font-heading text-[10px] font-bold uppercase tracking-wide text-neutral-darkGray">Total tasks</div>
        </div>
        <div class="border-3 border-black bg-ledger-lime p-3 shadow-brutal-sm">
          <div class="mb-1 text-xl" aria-hidden="true">
            ✓
          </div>
          <div class="font-heading text-2xl font-black tabular-nums">{stats().completed}</div>
          <div class="font-heading text-[10px] font-bold uppercase tracking-wide text-black/80">Completed</div>
        </div>
        <div class="border-3 border-black bg-ledger-sky p-3 shadow-brutal-sm">
          <div class="mb-1 text-xl" aria-hidden="true">
            💼
          </div>
          <div class="font-heading text-2xl font-black tabular-nums">{stats().pending}</div>
          <div class="font-heading text-[10px] font-bold uppercase tracking-wide text-black/80">Pending</div>
        </div>
        <div class="border-3 border-black bg-ledger-mint p-3 shadow-brutal-sm">
          <div class="mb-1 text-xl" aria-hidden="true">
            ↻
          </div>
          <div class="font-heading text-2xl font-black tabular-nums">{stats().inProgress}</div>
          <div class="font-heading text-[10px] font-bold uppercase tracking-wide text-black/80">In progress</div>
        </div>
        <div class="border-3 border-black bg-ledger-orange p-3 text-white shadow-brutal-sm">
          <div class="mb-1 text-xl" aria-hidden="true">
            !
          </div>
          <div class="font-heading text-2xl font-black tabular-nums">{String(stats().overdue).padStart(2, '0')}</div>
          <div class="font-heading text-[10px] font-bold uppercase tracking-wide text-white/90">Overdue</div>
        </div>
        <div class="border-3 border-black bg-ledger-pale p-3 shadow-brutal-sm">
          <div class="mb-1 text-xl" aria-hidden="true">
            📅
          </div>
          <div class="font-heading text-2xl font-black tabular-nums">{String(stats().dueToday).padStart(2, '0')}</div>
          <div class="font-heading text-[10px] font-bold uppercase tracking-wide text-black/80">Due today</div>
        </div>
      </div>

      {/* Toolbar: status + priority + CTA */}
      <div class="mb-4 flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
        <div class="flex flex-wrap items-center gap-2">
          <button type="button" class={statusBtn(status() === '')} onClick={() => handleFilterChange('status', '')}>
            All status
          </button>
          <button type="button" class={statusBtn(status() === 'todo')} onClick={() => handleFilterChange('status', 'todo')}>
            Todo
          </button>
          <button
            type="button"
            class={statusBtn(status() === 'in_progress')}
            onClick={() => handleFilterChange('status', 'in_progress')}
          >
            In progress
          </button>
          <button type="button" class={statusBtn(status() === 'done')} onClick={() => handleFilterChange('status', 'done')}>
            Done
          </button>
          <button
            type="button"
            class={statusBtn(status() === 'cancelled')}
            onClick={() => handleFilterChange('status', 'cancelled')}
          >
            Cancelled
          </button>
        </div>

        <div class="flex flex-wrap items-center gap-3">
          <label class="flex items-center gap-2 font-heading text-xs font-black uppercase">
            <span class="hidden sm:inline">Priority</span>
            <select
              class="select max-w-[200px] border-3 border-black bg-white py-2 font-heading text-xs font-bold uppercase shadow-brutal-sm"
              value={priority()}
              onChange={(e) => handleFilterChange('priority', e.currentTarget.value)}
            >
              <option value="">All</option>
              <option value="low">Low</option>
              <option value="medium">Medium</option>
              <option value="high">High</option>
              <option value="urgent">Urgent</option>
            </select>
          </label>

          <div class="flex gap-1 border-3 border-black bg-white p-1 shadow-brutal-sm">
            <button
              type="button"
              class={viewMode() === 'grid' ? 'bg-black px-2 py-1 text-xs font-black uppercase text-white' : 'px-2 py-1 text-xs font-bold uppercase'}
              onClick={() => setViewMode('grid')}
            >
              Grid
            </button>
            <button
              type="button"
              class={viewMode() === 'list' ? 'bg-black px-2 py-1 text-xs font-black uppercase text-white' : 'px-2 py-1 text-xs font-bold uppercase'}
              onClick={() => setViewMode('list')}
            >
              List
            </button>
            <button
              type="button"
              class={viewMode() === 'kanban' ? 'bg-black px-2 py-1 text-xs font-black uppercase text-white' : 'px-2 py-1 text-xs font-bold uppercase'}
              onClick={() => setViewMode('kanban')}
            >
              Kanban
            </button>
          </div>

          <ExportButton onExport={handleExport} isExporting={isExporting()} label="Export" />

          <Button
            variant="secondary"
            size="sm"
            onClick={() => {
              setMyOnly((v) => !v);
              setPage(1);
            }}
          >
            {myOnly() ? 'All tasks' : 'My tasks'}
          </Button>

          <button
            type="button"
            class="border-3 border-black bg-ledger-lime px-4 py-2 font-heading text-xs font-black uppercase shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5"
            onClick={() => navigate('/tasks/new')}
          >
            + New task
          </button>
        </div>
      </div>

      {/* Due filters + reset */}
      <div class="mb-6 flex flex-wrap items-center gap-2">
        <button
          type="button"
          class={statusBtn(dueTodayOnly())}
          onClick={() => {
            setDueTodayOnly((v) => !v);
            if (!dueTodayOnly()) setOverdueOnly(false);
            setPage(1);
          }}
        >
          Due today
        </button>
        <button
          type="button"
          class={statusBtn(overdueOnly())}
          onClick={() => {
            setOverdueOnly((v) => !v);
            if (!overdueOnly()) setDueTodayOnly(false);
            setPage(1);
          }}
        >
          Overdue
        </button>
        <button type="button" class={statusBtn(false)} onClick={resetFilters}>
          Reset filters
        </button>
      </div>

      <Show when={selectedIds().size > 0}>
        <Card class="mb-6 p-4">
          <div class="flex flex-wrap items-center justify-between gap-3">
            <div class="font-heading font-bold uppercase">Selected: {selectedIds().size}</div>
            <div class="flex flex-wrap gap-2">
              <Button variant="primary" size="sm" onClick={() => bulkAction('complete')}>
                Complete
              </Button>
              <Button variant="secondary" size="sm" onClick={() => bulkAction('cancel')}>
                Cancel
              </Button>
              <Button variant="secondary" size="sm" class="!bg-red-500 hover:!bg-red-600" onClick={() => bulkAction('delete')}>
                Delete
              </Button>
              <Button variant="secondary" size="sm" onClick={clearSelection}>
                Clear
              </Button>
            </div>
          </div>
        </Card>
      </Show>

      <Show when={tasks.isPending}>
        <div class="flex justify-center p-12">
          <Spinner />
        </div>
      </Show>

      <Show when={tasks.isError}>
        <Card class="border-3 border-red-500 bg-red-50 p-6">
          <p class="font-heading font-bold text-red-700">Error loading tasks: {tasks.error?.message}</p>
          <Button variant="secondary" size="sm" class="mt-4" onClick={() => tasks.refetch()}>
            Retry
          </Button>
        </Card>
      </Show>

      <Show when={tasks.data}>
        <Show when={viewMode() !== 'kanban'}>
          <div
            class={`grid gap-6 ${viewMode() === 'grid' ? 'grid-cols-1 md:grid-cols-2 xl:grid-cols-3' : 'grid-cols-1'}`}
          >
            <For each={tasks.data?.data || []}>
              {(task: any, i) => (
                <div class="relative group">
                  <div class="absolute left-3 top-3 z-20">
                    <input
                      type="checkbox"
                      class="checkbox h-5 w-5"
                      checked={selectedIds().has(task.id)}
                      onClick={(e) => e.stopPropagation()}
                      onChange={(e) => toggleSelected(task.id, !!e.currentTarget.checked)}
                    />
                  </div>

                  <TaskCard
                    title={task.title}
                    description={task.description || ''}
                    priority={task.priority}
                    dueDate={formatTaskDue(task.due_date)}
                    status={task.status}
                    tags={tagsForRow(rowIndex(i))}
                    onClick={() => navigate(`/tasks/${task.id}`)}
                  />

                  <div class="absolute right-3 top-12 z-20 flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
                    <Show when={task.status !== 'done'}>
                      <Button
                        variant="primary"
                        size="sm"
                        class="!min-w-0 !px-2 !bg-green-600"
                        onClick={(e) => {
                          e.stopPropagation();
                          completeTask.mutate(task.id);
                        }}
                        title="Mark as completed"
                      >
                        ✓
                      </Button>
                    </Show>

                    <Show when={task.status !== 'in_progress'}>
                      <Button
                        variant="primary"
                        size="sm"
                        class="!min-w-0 !px-2 !bg-blue-600"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleQuickStatusUpdate(task.id, 'in_progress');
                        }}
                        title="In progress"
                      >
                        ▶
                      </Button>
                    </Show>

                    <Button
                      variant="primary"
                      size="sm"
                      class="!min-w-0 !px-2"
                      onClick={(e) => {
                        e.stopPropagation();
                        navigate(`/tasks/${task.id}/edit`);
                      }}
                      title="Edit"
                    >
                      ✎
                    </Button>

                    <Button
                      variant="secondary"
                      size="sm"
                      class="!min-w-0 !px-2 !bg-red-500 hover:!bg-red-600"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteTask(task.id);
                      }}
                      disabled={deleteTask.isPending}
                      title="Delete"
                    >
                      ×
                    </Button>
                  </div>
                </div>
              )}
            </For>

            {/* Add quick task placeholder */}
            <button
              type="button"
              class="flex min-h-[220px] flex-col items-center justify-center gap-3 border-3 border-dashed border-neutral-gray bg-background p-6 text-center shadow-none transition-all hover:border-black hover:bg-ledger-pale/50"
              onClick={() => navigate('/tasks/new')}
            >
              <span class="text-4xl font-light leading-none text-neutral-gray">+</span>
              <span class="font-heading text-sm font-black uppercase text-neutral-darkGray">Add quick task</span>
            </button>
          </div>
        </Show>

        <Show when={viewMode() === 'kanban'}>
          <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
            <div
              class="border-3 border-black bg-white"
              onDragOver={(e) => e.preventDefault()}
              onDrop={(e) => onDropToStatus(e as DragEvent, 'todo')}
            >
              <div class="border-b-3 border-black bg-ledger-pale p-3 font-heading text-sm font-black uppercase">Todo</div>
              <div class="space-y-3 p-3">
                <For each={tasksByStatus().todo}>
                  {(task: any, i) => (
                    <div class="relative">
                      <div class="absolute left-2 top-2 z-20">
                        <input
                          type="checkbox"
                          class="checkbox h-5 w-5"
                          checked={selectedIds().has(task.id)}
                          onClick={(e) => e.stopPropagation()}
                          onChange={(e) => toggleSelected(task.id, !!e.currentTarget.checked)}
                        />
                      </div>
                      <div draggable onDragStart={(e) => onDragStartTask(e as DragEvent, task.id)}>
                        <TaskCard
                          title={task.title}
                          description={task.description || ''}
                          priority={task.priority}
                          dueDate={formatTaskDue(task.due_date)}
                          status={task.status}
                          tags={tagsForRow(rowIndex(i))}
                          onClick={() => navigate(`/tasks/${task.id}`)}
                        />
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </div>

            <div
              class="border-3 border-black bg-white"
              onDragOver={(e) => e.preventDefault()}
              onDrop={(e) => onDropToStatus(e as DragEvent, 'in_progress')}
            >
              <div class="border-b-3 border-black bg-ledger-sky p-3 font-heading text-sm font-black uppercase">In progress</div>
              <div class="space-y-3 p-3">
                <For each={tasksByStatus().in_progress}>
                  {(task: any, i) => (
                    <div class="relative">
                      <div class="absolute left-2 top-2 z-20">
                        <input
                          type="checkbox"
                          class="checkbox h-5 w-5"
                          checked={selectedIds().has(task.id)}
                          onClick={(e) => e.stopPropagation()}
                          onChange={(e) => toggleSelected(task.id, !!e.currentTarget.checked)}
                        />
                      </div>
                      <div draggable onDragStart={(e) => onDragStartTask(e as DragEvent, task.id)}>
                        <TaskCard
                          title={task.title}
                          description={task.description || ''}
                          priority={task.priority}
                          dueDate={formatTaskDue(task.due_date)}
                          status={task.status}
                          tags={tagsForRow(rowIndex(i))}
                          onClick={() => navigate(`/tasks/${task.id}`)}
                        />
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </div>

            <div
              class="border-3 border-black bg-white"
              onDragOver={(e) => e.preventDefault()}
              onDrop={(e) => onDropToStatus(e as DragEvent, 'done')}
            >
              <div class="border-b-3 border-black bg-ledger-lime p-3 font-heading text-sm font-black uppercase">Done</div>
              <div class="space-y-3 p-3">
                <For each={tasksByStatus().done}>
                  {(task: any, i) => (
                    <div class="relative">
                      <div class="absolute left-2 top-2 z-20">
                        <input
                          type="checkbox"
                          class="checkbox h-5 w-5"
                          checked={selectedIds().has(task.id)}
                          onClick={(e) => e.stopPropagation()}
                          onChange={(e) => toggleSelected(task.id, !!e.currentTarget.checked)}
                        />
                      </div>
                      <div draggable onDragStart={(e) => onDragStartTask(e as DragEvent, task.id)}>
                        <TaskCard
                          title={task.title}
                          description={task.description || ''}
                          priority={task.priority}
                          dueDate={formatTaskDue(task.due_date)}
                          status={task.status}
                          tags={tagsForRow(rowIndex(i))}
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

        <Show when={tasks.data?.pagination && viewMode() !== 'kanban'}>
          <div class="mt-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <p class="font-body text-sm text-neutral-darkGray">
              Showing {tasks.data?.data?.length || 0} of {tasks.data?.pagination?.total || 0} tasks
            </p>

            <div class="flex flex-wrap items-center gap-2">
              <Button variant="secondary" disabled={!tasks.data?.pagination?.has_prev} onClick={() => setPage((p) => Math.max(1, p - 1))}>
                ← Previous
              </Button>

              <Badge variant="primary" class="border-3 border-black px-4 py-2 font-heading">
                Page {page()} / {tasks.data?.pagination?.total_pages || 1}
              </Badge>

              <Button variant="secondary" disabled={!tasks.data?.pagination?.has_next} onClick={() => setPage((p) => p + 1)}>
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
