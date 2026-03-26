import { Component, Show } from 'solid-js';
import { useNavigate, useParams } from '@solidjs/router';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, Spinner } from '~/components/ui';
import { useTask, useDeleteTask, useCompleteTask } from '~/lib/hooks/useTasks';

const TaskDetail: Component = () => {
  const params = useParams();
  const navigate = useNavigate();
  const id = () => params.id;

  const task = useTask(id);
  const deleteTask = useDeleteTask();
  const completeTask = useCompleteTask();

  const onDelete = () => {
    if (!id()) return;
    if (!confirm('Delete this task?')) return;
    deleteTask.mutate(id()!, {
      onSuccess: () => navigate('/tasks', { replace: true }),
    });
  };

  const onComplete = () => {
    if (!id()) return;
    completeTask.mutate(id()!);
  };

  return (
    <div class="max-w-4xl">
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">Task</h1>
          <p class="text-neutral-darkGray break-all">{id()}</p>
        </div>
        <div class="flex gap-2">
          <Button variant="secondary" onClick={() => navigate('/tasks')}>
            ← Back
          </Button>
          <Button variant="primary" onClick={() => navigate(`/tasks/${id()}/edit`)}>
            ✏️ Edit
          </Button>
          <Button
            variant="primary"
            class="bg-green-500 hover:bg-green-600"
            onClick={onComplete}
            disabled={completeTask.isPending || task.data?.status === 'done'}
          >
            <Show when={completeTask.isPending} fallback="✅ Complete">
              <Spinner class="inline-block mr-2" />
              Completing...
            </Show>
          </Button>
          <Button
            variant="secondary"
            class="bg-red-500 hover:bg-red-600"
            onClick={onDelete}
            disabled={deleteTask.isPending}
          >
            <Show when={deleteTask.isPending} fallback="🗑️ Delete">
              <Spinner class="inline-block mr-2" />
              Deleting...
            </Show>
          </Button>
        </div>
      </div>

      <Card class="border-5">
        <CardHeader>
          <CardTitle class="flex items-center justify-between">
            <span>Details</span>
            <Show when={task.data?.status}>
              <Badge variant="primary" class="border-3">
                {task.data!.status}
              </Badge>
            </Show>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Show
            when={!task.isPending && !task.isError && !!task.data}
            fallback={
              <div class="py-8">
                <Show when={task.isPending}>
                  <div class="flex items-center gap-2">
                    <Spinner />
                    <span class="font-bold">Loading...</span>
                  </div>
                </Show>
                <Show when={task.isError}>
                  <div class="p-3 bg-red-100 border-3 border-red-500 text-red-700 text-sm font-bold">
                    {task.error?.message}
                  </div>
                </Show>
              </div>
            }
          >
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="p-4 border-3 border-black bg-white md:col-span-2">
                <div class="text-xs font-bold uppercase text-neutral-darkGray">Title</div>
                <div class="font-heading font-black text-xl">{task.data!.title}</div>
              </div>
              <div class="p-4 border-3 border-black bg-white">
                <div class="text-xs font-bold uppercase text-neutral-darkGray">Priority</div>
                <div class="font-mono">{task.data!.priority}</div>
              </div>
              <div class="p-4 border-3 border-black bg-white">
                <div class="text-xs font-bold uppercase text-neutral-darkGray">Due Date</div>
                <div class="font-mono">{task.data!.due_date ? new Date(task.data!.due_date).toLocaleDateString() : '-'}</div>
              </div>
              <div class="p-4 border-3 border-black bg-white">
                <div class="text-xs font-bold uppercase text-neutral-darkGray">Client ID</div>
                <div class="font-mono break-all">{task.data!.client_id || '-'}</div>
              </div>
              <div class="p-4 border-3 border-black bg-white">
                <div class="text-xs font-bold uppercase text-neutral-darkGray">Assigned To</div>
                <div class="font-mono break-all">{(task.data as any)?.assigned_to || '-'}</div>
              </div>
              <div class="p-4 border-3 border-black bg-white md:col-span-2">
                <div class="text-xs font-bold uppercase text-neutral-darkGray">Description</div>
                <div class="whitespace-pre-wrap">{task.data!.description || '-'}</div>
              </div>
            </div>
          </Show>
        </CardContent>
      </Card>
    </div>
  );
};

export default TaskDetail;

