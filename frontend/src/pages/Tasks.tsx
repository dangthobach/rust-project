import { Component, createSignal, For, Show, createMemo, createEffect, type Accessor } from 'solid-js';
import { useNavigate, useLocation } from '@solidjs/router';
import { Card, Button, Input, Spinner } from '~/components/ui';
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
  const location = useLocation();
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

  createEffect(() => {
    if (!location.pathname.startsWith('/tasks')) return;
    const q = new URLSearchParams(location.search).get('q');
    if (q !== null) setSearch(q);
  });

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

  const paginationPageButtons = createMemo(() => {
    const tp = tasks.data?.total_pages || 1;
    const cur = page();
    const nums: number[] = [];
    const start = Math.max(1, Math.min(cur, tp) - 1);
    const end = Math.min(tp, start + 2);
    for (let i = start; i <= end; i++) nums.push(i);
    return nums;
  });

  const stats = createMemo(
    () =>
      taskStats.data || {
        total: 0,
        completed: 0,
        pending: 0,
        inProgress: 0,
        overdue: 0,
        dueToday: 0,
        weekOverWeekPct: 0,
        byPriority: { high: 0, medium: 0, low: 0 },
      },
  );

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

  const clearSelection = () => setSelectedIds(new Set<string>());

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
    const rows = tasks.data?.items || [];
    const map: Record<string, unknown[]> = { todo: [], in_progress: [], done: [], cancelled: [] };
    for (const t of rows) {
      (map[(t as { status: string }).status] ??= []).push(t);
    }
    return map as Record<string, any[]>;
  });

  const statusBtn = (active: boolean) =>
    [
      'border-3 border-black px-3 py-2 font-heading text-xs font-black uppercase tracking-wide shadow-brutal-sm transition-all',
      active
        ? 'bg-ledger-lime text-black -translate-x-0.5 -translate-y-0.5'
        : 'bg-white text-black hover:-translate-x-0.5 hover:-translate-y-0.5 hover:bg-ledger-pale',
    ].join(' ');

  return (
    <div class="font-body">
      {/* Page header — Task Management (match HTML mock top bar within page) */}
      <div class="mb-6 flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between lg:gap-6">
        <div class="flex min-w-0 flex-col gap-4 lg:flex-row lg:items-center lg:gap-8">
        <h1 class="shrink-0 font-heading text-2xl font-black uppercase tracking-tight text-black sm:text-3xl">
          Task Management
        </h1>

        <div class="relative min-w-0 flex-1 lg:w-96">
          <span class="pointer-events-none absolute left-3 top-1/2 z-10 -translate-y-1/2 text-neutral-gray" aria-hidden="true">
            <span class="material-symbols-outlined">search</span>
          </span>
          <Input
            type="text"
            placeholder="SEARCH ARCHIVE / DATA..."
            value={search()}
            onInput={(e) => handleSearch(e.currentTarget.value)}
            class="!pl-10 !bg-white !border-[3px] !border-black py-2 font-heading text-sm font-bold uppercase focus:!border-secondary"
          />
        </div>
        </div>
        <div class="flex shrink-0 flex-wrap items-center gap-3">
          <button
            type="button"
            class="border-[3px] border-black bg-neutral-lightGray px-4 py-2 font-heading text-xs font-black uppercase shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5"
            onClick={() => handleExport('csv')}
          >
            <span class="material-symbols-outlined text-base mr-2">file_download</span>
            Export
          </button>
          <button
            type="button"
            class="border-[3px] border-black bg-ledger-electric px-4 py-2 font-heading text-xs font-black uppercase text-white shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5"
            onClick={() => {
              setMyOnly(true);
              setPage(1);
            }}
          >
            My tasks
          </button>
          <button
            type="button"
            class="border-[3px] border-black bg-ledger-lime px-6 py-2 font-heading text-xs font-black uppercase shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5"
            onClick={() => navigate('/tasks/new')}
          >
            <span class="material-symbols-outlined text-base mr-2">add_box</span>
            New task
          </button>
        </div>
      </div>

      {/* Stat Cards Section — match HTML mock */}
      <section class="grid grid-cols-1 gap-6 md:grid-cols-3 lg:grid-cols-6">
        <div class="bg-white border-[3px] border-black p-6 flex flex-col justify-between">
          <div class="flex justify-between items-start mb-4">
            <span class="material-symbols-outlined text-ledger-electric text-3xl" aria-hidden="true">analytics</span>
            <span class="bg-black text-white px-2 py-0.5 font-heading text-[10px] font-black uppercase">
              {stats().weekOverWeekPct >= 0 ? '+' : ''}
              {stats().weekOverWeekPct}%
            </span>
          </div>
          <div>
            <p class="font-heading text-[10px] text-neutral-gray uppercase font-black tracking-widest">Total tasks</p>
            <p class="font-heading font-black text-4xl tabular-nums">{stats().total.toLocaleString('en-US')}</p>
          </div>
        </div>

        <div class="bg-white border-[3px] border-black border-l-[12px] border-l-ledger-lime p-6">
          <div class="flex justify-between items-start mb-4">
            <span class="material-symbols-outlined text-green-700 text-3xl" aria-hidden="true">check_circle</span>
          </div>
          <p class="font-heading text-[10px] text-neutral-gray uppercase font-black tracking-widest">Completed</p>
          <p class="font-heading font-black text-4xl tabular-nums text-green-700">{stats().completed.toLocaleString('en-US')}</p>
        </div>

        <div class="bg-white border-[3px] border-black border-l-[12px] border-l-ledger-electric p-6">
          <div class="flex justify-between items-start mb-4">
            <span class="material-symbols-outlined text-ledger-electric text-3xl" aria-hidden="true">sync</span>
          </div>
          <p class="font-heading text-[10px] text-neutral-gray uppercase font-black tracking-widest">In progress</p>
          <p class="font-heading font-black text-4xl tabular-nums text-ledger-electric">{stats().inProgress.toLocaleString('en-US')}</p>
        </div>

        <div class="bg-white border-[3px] border-black border-l-[12px] border-l-neutral-lightGray p-6">
          <div class="flex justify-between items-start mb-4">
            <span class="material-symbols-outlined text-neutral-gray text-3xl" aria-hidden="true">hourglass_empty</span>
          </div>
          <p class="font-heading text-[10px] text-neutral-gray uppercase font-black tracking-widest">Pending</p>
          <p class="font-heading font-black text-4xl tabular-nums">{stats().pending.toLocaleString('en-US')}</p>
        </div>

        <div class="bg-white border-[3px] border-black border-l-[12px] border-l-red-500 p-6">
          <div class="flex justify-between items-start mb-4">
            <span class="material-symbols-outlined text-red-600 text-3xl" aria-hidden="true">error</span>
          </div>
          <p class="font-heading text-[10px] text-neutral-gray uppercase font-black tracking-widest">Overdue</p>
          <p class="font-heading font-black text-4xl tabular-nums text-red-600">{String(stats().overdue).padStart(2, '0')}</p>
        </div>

        <div class="bg-ledger-lime border-[3px] border-black p-6">
          <div class="flex justify-between items-start mb-4">
            <span class="material-symbols-outlined text-black text-3xl" aria-hidden="true">calendar_today</span>
          </div>
          <p class="font-heading text-[10px] text-black uppercase font-black tracking-widest">Due today</p>
          <p class="font-heading font-black text-4xl tabular-nums text-black">{String(stats().dueToday).padStart(2, '0')}</p>
        </div>
      </section>

      {/* Filter Bar — match HTML mock (button groups inside bordered strips) */}
      <section class="bg-white border-[3px] border-black p-4 flex flex-wrap items-center justify-between gap-6">
        <div class="flex items-center gap-8">
          <div class="flex flex-col gap-1">
            <label class="font-heading text-[10px] font-black uppercase tracking-tighter text-neutral-gray">Filter By Status</label>
            <div class="flex border-[3px] border-black bg-white">
              <button
                type="button"
                class="px-4 py-1.5 font-heading text-xs font-black uppercase bg-ledger-lime border-r-[3px] border-black"
                onClick={() => handleFilterChange('status', '')}
              >
                All
              </button>
              <button
                type="button"
                class="px-4 py-1.5 font-heading text-xs font-black uppercase hover:bg-neutral-lightGray border-r-[3px] border-black transition-colors"
                onClick={() => handleFilterChange('status', 'todo')}
              >
                Todo
              </button>
              <button
                type="button"
                class="px-4 py-1.5 font-heading text-xs font-black uppercase hover:bg-neutral-lightGray border-r-[3px] border-black transition-colors"
                onClick={() => handleFilterChange('status', 'in_progress')}
              >
                Active
              </button>
              <button
                type="button"
                class="px-4 py-1.5 font-heading text-xs font-black uppercase hover:bg-neutral-lightGray border-r-[3px] border-black transition-colors"
                onClick={() => handleFilterChange('status', 'done')}
              >
                Done
              </button>
              <button
                type="button"
                class="px-4 py-1.5 font-heading text-xs font-black uppercase hover:bg-neutral-lightGray transition-colors"
                onClick={() => handleFilterChange('status', 'cancelled')}
              >
                Cancelled
              </button>
            </div>
          </div>

          <div class="flex flex-col gap-1">
            <label class="font-heading text-[10px] font-black uppercase tracking-tighter text-neutral-gray">Priority Level</label>
            <div class="flex border-[3px] border-black bg-white">
              <button
                type="button"
                class="px-4 py-1.5 font-heading text-xs font-black uppercase bg-white border-r-[3px] border-black"
                onClick={() => handleFilterChange('priority', '')}
              >
                All
              </button>
              <button
                type="button"
                class="px-4 py-1.5 font-heading text-xs font-black uppercase hover:bg-neutral-lightGray border-r-[3px] border-black transition-colors text-red-700"
                onClick={() => handleFilterChange('priority', 'urgent')}
              >
                Urgent
              </button>
              <button
                type="button"
                class="px-4 py-1.5 font-heading text-xs font-black uppercase hover:bg-neutral-lightGray border-r-[3px] border-black transition-colors"
                onClick={() => handleFilterChange('priority', 'high')}
              >
                High
              </button>
              <button
                type="button"
                class="px-4 py-1.5 font-heading text-xs font-black uppercase hover:bg-neutral-lightGray transition-colors"
                onClick={() => handleFilterChange('priority', 'low')}
              >
                Low
              </button>
            </div>
          </div>
        </div>

        <div class="flex items-center gap-4">
          <div class="flex border-[3px] border-black bg-white">
            <button
              type="button"
              class={viewMode() === 'grid' ? 'p-2 border-r-[3px] border-black bg-neutral-lightGray' : 'p-2 border-r-[3px] border-black hover:bg-neutral-lightGray'}
              onClick={() => setViewMode('grid')}
              aria-label="Grid"
              title="Grid"
            >
              <span class="material-symbols-outlined text-xl">grid_view</span>
            </button>
            <button
              type="button"
              class={viewMode() === 'list' ? 'p-2 border-r-[3px] border-black bg-neutral-lightGray' : 'p-2 border-r-[3px] border-black hover:bg-neutral-lightGray'}
              onClick={() => setViewMode('list')}
              aria-label="List"
              title="List"
            >
              <span class="material-symbols-outlined text-xl">list</span>
            </button>
            <button
              type="button"
              class={viewMode() === 'kanban' ? 'p-2 bg-neutral-lightGray' : 'p-2 hover:bg-neutral-lightGray'}
              onClick={() => setViewMode('kanban')}
              aria-label="Kanban"
              title="Kanban"
            >
              <span class="material-symbols-outlined text-xl">view_kanban</span>
            </button>
          </div>
        </div>
      </section>

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
          <section
            class={`grid gap-8 ${viewMode() === 'grid' ? 'grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4' : 'grid-cols-1'}`}
          >
            {/* Add quick task placeholder (placed first like mock) */}
            <button
              type="button"
              class="border-[3px] border-black border-dashed h-full min-h-[320px] bg-white hover:bg-ledger-lime transition-colors group p-8 flex flex-col items-center justify-center text-center"
              onClick={() => navigate('/tasks/new')}
            >
              <div class="w-16 h-16 border-[3px] border-black bg-white flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                <span class="material-symbols-outlined text-4xl" aria-hidden="true">add</span>
              </div>
              <h3 class="font-heading font-bold text-xl uppercase tracking-tight">Add Quick Task</h3>
              <p class="font-body text-sm text-neutral-gray mt-2">Initialize new ledger entry for the workspace.</p>
            </button>

            <For each={tasks.data?.items || []}>
              {(task: any, i) => (
                <div class="relative group">
                  <Show when={viewMode() !== 'grid'}>
                    <div class="absolute left-3 top-3 z-20">
                      <input
                        type="checkbox"
                        class="checkbox h-5 w-5"
                        checked={selectedIds().has(task.id)}
                        onClick={(e) => e.stopPropagation()}
                        onChange={(e) => toggleSelected(task.id, !!e.currentTarget.checked)}
                      />
                    </div>
                  </Show>

                  <TaskCard
                    title={task.title}
                    description={task.description || ''}
                    priority={task.priority}
                    dueDate={formatTaskDue(task.due_date)}
                    status={task.status}
                    tags={tagsForRow(rowIndex(i))}
                    onClick={() => navigate(`/tasks/${task.id}`)}
                  />

                  <Show when={viewMode() !== 'grid'}>
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
                  </Show>
                </div>
              )}
            </For>
          </section>
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

        <Show when={tasks.data && viewMode() !== 'kanban'}>
          <footer class="mt-8 flex items-center justify-between border-[3px] border-black p-4 bg-white">
            <p class="font-heading text-[10px] font-black uppercase text-neutral-gray">
              {(() => {
                const meta = tasks.data;
                if (!meta) return 'Showing 0-0 of 0 technical tasks';
                const limit = meta.page_size || 1;
                const from = (meta.page - 1) * limit + 1;
                const to = Math.min(meta.page * limit, meta.total);
                const total = Number(meta.total).toLocaleString('en-US');
                return `Showing ${from}-${to} of ${total} technical tasks`;
              })()}
            </p>

            <div class="flex gap-2">
              <button
                type="button"
                class="w-10 h-10 border-[3px] border-black flex items-center justify-center hover:bg-neutral-lightGray transition-colors disabled:opacity-40"
                disabled={(tasks.data?.page ?? 1) <= 1}
                onClick={() => setPage((p) => Math.max(1, p - 1))}
                aria-label="Previous page"
              >
                <span class="material-symbols-outlined">chevron_left</span>
              </button>
              <For each={paginationPageButtons()}>
                {(n) => (
                  <button
                    type="button"
                    class={[
                      'w-10 h-10 border-[3px] border-black flex items-center justify-center font-heading font-bold transition-colors',
                      n === page() ? 'bg-ledger-lime' : 'hover:bg-neutral-lightGray',
                    ].join(' ')}
                    onClick={() => setPage(n)}
                  >
                    {n}
                  </button>
                )}
              </For>
              <button
                type="button"
                class="w-10 h-10 border-[3px] border-black flex items-center justify-center hover:bg-neutral-lightGray transition-colors disabled:opacity-40"
                disabled={(tasks.data?.page ?? 1) >= (tasks.data?.total_pages ?? 1)}
                onClick={() => setPage((p) => p + 1)}
                aria-label="Next page"
              >
                <span class="material-symbols-outlined">chevron_right</span>
              </button>
            </div>
          </footer>
        </Show>
      </Show>
    </div>
  );
};

export default Tasks;
