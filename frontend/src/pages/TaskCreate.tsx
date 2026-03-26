import { Component, createSignal, Show, createResource, For, createEffect, onCleanup } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { Card, CardContent, CardHeader, CardTitle, Button, Input, Spinner, Badge } from '~/components/ui';
import { useCreateTask } from '~/lib/hooks/useTasks';
import { api, type Client, type User } from '~/lib/api';

const TaskCreate: Component = () => {
  const navigate = useNavigate();
  const createTask = useCreateTask();

  const [form, setForm] = createSignal({
    title: '',
    description: '',
    status: 'todo' as const,
    priority: 'medium' as const,
    due_date: '',
    client_id: '',
    assigned_to: '',
  });

  const [clientTerm, setClientTerm] = createSignal('');
  const [userTerm, setUserTerm] = createSignal('');
  const [clientTermDebounced, setClientTermDebounced] = createSignal('');
  const [userTermDebounced, setUserTermDebounced] = createSignal('');
  const [clientOpen, setClientOpen] = createSignal(false);
  const [userOpen, setUserOpen] = createSignal(false);
  const [selectedClient, setSelectedClient] = createSignal<Client | null>(null);
  const [selectedUser, setSelectedUser] = createSignal<User | null>(null);

  let clientBoxRef: HTMLDivElement | undefined;
  let userBoxRef: HTMLDivElement | undefined;

  createEffect(() => {
    const t = clientTerm();
    const handle = window.setTimeout(() => setClientTermDebounced(t), 250);
    onCleanup(() => window.clearTimeout(handle));
  });

  createEffect(() => {
    const t = userTerm();
    const handle = window.setTimeout(() => setUserTermDebounced(t), 250);
    onCleanup(() => window.clearTimeout(handle));
  });

  createEffect(() => {
    if (!clientOpen() && !userOpen()) return;
    const onDown = (e: PointerEvent) => {
      const target = e.target as Node | null;
      if (!target) return;
      if (clientOpen() && clientBoxRef && !clientBoxRef.contains(target)) setClientOpen(false);
      if (userOpen() && userBoxRef && !userBoxRef.contains(target)) setUserOpen(false);
    };
    document.addEventListener('pointerdown', onDown, { capture: true });
    onCleanup(() => document.removeEventListener('pointerdown', onDown, { capture: true } as any));
  });

  const [clientMatches] = createResource<Client[], string>(
    clientTermDebounced,
    async (term) => {
      const t = term.trim();
      if (t.length < 2) return [];
      return api.searchClients({ search_term: t, limit: 8 });
    },
    { initialValue: [] },
  );

  const [userMatches] = createResource<User[], string>(
    userTermDebounced,
    async (term) => {
      const t = term.trim();
      if (t.length < 2) return [];
      return api.searchUsersAdmin({ search: t, limit: 8, page: 1 });
    },
    { initialValue: [] },
  );

  const canSubmit = () => form().title.trim().length >= 3;

  const onSubmit = () => {
    if (!canSubmit()) return;
    const f = form();
    createTask.mutate(
      {
        title: f.title.trim(),
        description: f.description.trim() || undefined,
        status: f.status,
        priority: f.priority,
        due_date: f.due_date || undefined,
        client_id: f.client_id.trim() || undefined,
        assigned_to: f.assigned_to.trim() || undefined,
      },
      {
        onSuccess: () => {
          navigate('/tasks', { replace: true });
        },
      },
    );
  };

  return (
    <div class="max-w-3xl">
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">New Task</h1>
          <p class="text-neutral-darkGray">Create a task</p>
        </div>
        <div class="flex gap-2">
          <Button variant="secondary" onClick={() => navigate('/tasks')}>
            ← Back
          </Button>
          <Button variant="primary" onClick={onSubmit} disabled={createTask.isPending || !canSubmit()}>
            <Show when={createTask.isPending} fallback="Create">
              <Spinner class="inline-block mr-2" />
              Creating...
            </Show>
          </Button>
        </div>
      </div>

      <Card class="border-5">
        <CardHeader>
          <CardTitle class="flex items-center justify-between">
            <span>Task Details</span>
            <Badge variant="info" class="border-3">
              page-based form
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <div>
            <label class="block font-bold uppercase text-sm mb-2">Title *</label>
            <Input
              type="text"
              value={form().title}
              onInput={(e: any) => setForm((p) => ({ ...p, title: e.currentTarget.value }))}
              placeholder="Task title"
            />
          </div>

          <div>
            <label class="block font-bold uppercase text-sm mb-2">Description</label>
            <textarea
              class="w-full p-3 border-3 border-black font-mono"
              rows="4"
              value={form().description}
              onInput={(e: any) => setForm((p) => ({ ...p, description: e.currentTarget.value }))}
              placeholder="Task description..."
            />
          </div>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label class="block font-bold uppercase text-sm mb-2">Status</label>
              <select
                class="w-full px-3 py-2 border-3 border-black font-mono bg-white"
                value={form().status}
                onChange={(e: any) => setForm((p) => ({ ...p, status: e.currentTarget.value }))}
              >
                <option value="todo">Todo</option>
                <option value="in_progress">In Progress</option>
                <option value="done">Done</option>
                <option value="cancelled">Cancelled</option>
              </select>
            </div>
            <div>
              <label class="block font-bold uppercase text-sm mb-2">Priority</label>
              <select
                class="w-full px-3 py-2 border-3 border-black font-mono bg-white"
                value={form().priority}
                onChange={(e: any) => setForm((p) => ({ ...p, priority: e.currentTarget.value }))}
              >
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
                <option value="urgent">Urgent</option>
              </select>
            </div>
            <div>
              <label class="block font-bold uppercase text-sm mb-2">Due Date</label>
              <Input
                type="date"
                value={form().due_date}
                onInput={(e: any) => setForm((p) => ({ ...p, due_date: e.currentTarget.value }))}
              />
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label class="block font-bold uppercase text-sm mb-2">Client</label>
              <div class="relative" ref={(el) => (clientBoxRef = el)}>
                <Input
                  type="text"
                  value={selectedClient()?.name ? `${selectedClient()!.name}` : clientTerm()}
                  onFocus={() => setClientOpen(true)}
                  onInput={(e: any) => {
                    const v = e.currentTarget.value;
                    setSelectedClient(null);
                    setForm((p) => ({ ...p, client_id: '' }));
                    setClientTerm(v);
                    setClientOpen(true);
                  }}
                  placeholder="Type to search clients (name/email)..."
                />
                <Show when={selectedClient()}>
                  <button
                    type="button"
                    class="absolute right-2 top-1/2 -translate-y-1/2 text-sm font-bold"
                    onClick={() => {
                      setSelectedClient(null);
                      setForm((p) => ({ ...p, client_id: '' }));
                      setClientTerm('');
                      setClientOpen(false);
                    }}
                    aria-label="Clear client"
                  >
                    ✕
                  </button>
                </Show>

                <Show when={clientOpen() && !selectedClient() && clientMatches.loading && clientTerm().trim().length >= 2}>
                  <div class="absolute z-20 mt-2 w-full bg-white border-3 border-black shadow-brutal p-3 text-sm">
                    Loading...
                  </div>
                </Show>

                <Show when={clientOpen() && !selectedClient() && !clientMatches.loading && clientMatches().length > 0}>
                  <div class="absolute z-20 mt-2 w-full bg-white border-3 border-black shadow-brutal max-h-64 overflow-auto">
                    <For each={clientMatches()}>
                      {(c) => (
                        <button
                          type="button"
                          class="w-full text-left px-3 py-2 hover:bg-neutral-lightGray border-b-2 border-black last:border-b-0"
                          onClick={() => {
                            setSelectedClient(c);
                            setForm((p) => ({ ...p, client_id: c.id }));
                            setClientTerm('');
                            setClientOpen(false);
                          }}
                        >
                          <div class="font-bold">{c.name}</div>
                          <div class="text-xs text-neutral-darkGray">{c.email || c.company || c.id}</div>
                        </button>
                      )}
                    </For>
                  </div>
                </Show>

                <Show
                  when={
                    clientOpen() &&
                    !selectedClient() &&
                    !clientMatches.loading &&
                    clientTerm().trim().length >= 2 &&
                    clientMatches().length === 0
                  }
                >
                  <div class="absolute z-20 mt-2 w-full bg-white border-3 border-black shadow-brutal p-3 text-sm">
                    No matches
                  </div>
                </Show>
              </div>
            </div>
            <div>
              <label class="block font-bold uppercase text-sm mb-2">Assigned user</label>
              <div class="relative" ref={(el) => (userBoxRef = el)}>
                <Input
                  type="text"
                  value={selectedUser()?.full_name ? `${selectedUser()!.full_name} (${selectedUser()!.email})` : userTerm()}
                  onFocus={() => setUserOpen(true)}
                  onInput={(e: any) => {
                    const v = e.currentTarget.value;
                    setSelectedUser(null);
                    setForm((p) => ({ ...p, assigned_to: '' }));
                    setUserTerm(v);
                    setUserOpen(true);
                  }}
                  placeholder="Type to search users (name/email)..."
                />
                <Show when={selectedUser()}>
                  <button
                    type="button"
                    class="absolute right-2 top-1/2 -translate-y-1/2 text-sm font-bold"
                    onClick={() => {
                      setSelectedUser(null);
                      setForm((p) => ({ ...p, assigned_to: '' }));
                      setUserTerm('');
                      setUserOpen(false);
                    }}
                    aria-label="Clear user"
                  >
                    ✕
                  </button>
                </Show>

                <Show when={userOpen() && !selectedUser() && userMatches.loading && userTerm().trim().length >= 2}>
                  <div class="absolute z-20 mt-2 w-full bg-white border-3 border-black shadow-brutal p-3 text-sm">
                    Loading...
                  </div>
                </Show>

                <Show when={userOpen() && !selectedUser() && !userMatches.loading && userMatches().length > 0}>
                  <div class="absolute z-20 mt-2 w-full bg-white border-3 border-black shadow-brutal max-h-64 overflow-auto">
                    <For each={userMatches()}>
                      {(u) => (
                        <button
                          type="button"
                          class="w-full text-left px-3 py-2 hover:bg-neutral-lightGray border-b-2 border-black last:border-b-0"
                          onClick={() => {
                            setSelectedUser(u);
                            setForm((p) => ({ ...p, assigned_to: u.id }));
                            setUserTerm('');
                            setUserOpen(false);
                          }}
                        >
                          <div class="font-bold">{u.full_name || u.email}</div>
                          <div class="text-xs text-neutral-darkGray">{u.email} • {u.role}</div>
                        </button>
                      )}
                    </For>
                  </div>
                </Show>

                <Show
                  when={
                    userOpen() &&
                    !selectedUser() &&
                    !userMatches.loading &&
                    userTerm().trim().length >= 2 &&
                    userMatches().length === 0
                  }
                >
                  <div class="absolute z-20 mt-2 w-full bg-white border-3 border-black shadow-brutal p-3 text-sm">
                    No matches
                  </div>
                </Show>
              </div>
            </div>
          </div>

          <Show when={createTask.isError}>
            <div class="p-3 bg-red-100 border-3 border-red-500 text-red-700 text-sm font-bold">
              {createTask.error?.message}
            </div>
          </Show>
        </CardContent>
      </Card>
    </div>
  );
};

export default TaskCreate;

