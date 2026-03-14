import { Component, createSignal, For, Show, createMemo } from 'solid-js';
import { useParams, useNavigate } from '@solidjs/router';
import { Card, CardHeader, CardTitle, CardContent, Button, Badge, Input, Spinner } from '~/components/ui';
import { TaskCard } from '~/components/crm';
import { useClient, useUpdateClient, useClientTasks, useTasks } from '~/lib/hooks';

const ClientDetail: Component = () => {
  const params = useParams();
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = createSignal<'overview' | 'tasks' | 'files' | 'notes'>('overview');
  const [editMode, setEditMode] = createSignal(false);
  const [editForm, setEditForm] = createSignal({
    name: '',
    email: '',
    phone: '',
    company: '',
    status: 'active' as const,
    notes: '',
  });

  // API hooks
  const client = useClient(() => params.id);
  const clientTasks = useClientTasks(() => params.id);
  const updateClient = useUpdateClient();

  // Initialize edit form when client data loads
  createMemo(() => {
    if (client.data) {
      setEditForm({
        name: client.data.name || '',
        email: client.data.email || '',
        phone: client.data.phone || '',
        company: client.data.company || '',
        status: client.data.status || 'active',
        notes: client.data.notes || '',
      });
    }
  });

  const handleSaveEdit = () => {
    if (!client.data?.id) return;
    
    updateClient.mutate({
      id: client.data.id,
      updates: editForm(),
    }, {
      onSuccess: () => {
        setEditMode(false);
      },
    });
  };

  const handleCancelEdit = () => {
    if (client.data) {
      setEditForm({
        name: client.data.name || '',
        email: client.data.email || '',
        phone: client.data.phone || '',
        company: client.data.company || '',
        status: client.data.status || 'active',
        notes: client.data.notes || '',
      });
    }
    setEditMode(false);
  };

  // Client statistics
  const clientStats = createMemo(() => {
    const tasks = clientTasks.data?.data || [];
    return {
      totalTasks: tasks.length,
      completedTasks: tasks.filter((t: any) => t.status === 'completed').length,
      pendingTasks: tasks.filter((t: any) => t.status === 'pending').length,
      inProgressTasks: tasks.filter((t: any) => t.status === 'in_progress').length,
      overdueTasks: tasks.filter((t: any) => {
        if (!t.due_date) return false;
        return new Date(t.due_date) < new Date() && t.status !== 'completed';
      }).length,
    };
  });

  return (
    <div>
      {/* Header */}
      <div class="flex items-start justify-between mb-8">
        <div class="flex items-center gap-4">
          <Button
            variant="secondary"
            size="md"
            onClick={() => navigate('/clients')}
          >
            ← Back to Clients
          </Button>
          
          <Show when={client.data} fallback={<Spinner />}>
            <div>
              <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">
                {client.data?.name}
              </h1>
              <p class="text-neutral-darkGray text-lg">
                {client.data?.company}
              </p>
              <Badge 
                variant={client.data?.status === 'active' ? 'success' : 'secondary'}
                class="mt-2"
              >
                {client.data?.status?.toUpperCase()}
              </Badge>
            </div>
          </Show>
        </div>

        <Show when={client.data}>
          <div class="flex gap-3">
            <Show when={!editMode()}>
              <Button
                variant="primary"
                size="lg"
                onClick={() => setEditMode(true)}
              >
                ✏️ Edit Client
              </Button>
            </Show>
            <Show when={editMode()}>
              <Button
                variant="secondary"
                size="lg"
                onClick={handleCancelEdit}
                disabled={updateClient.isPending}
              >
                Cancel
              </Button>
              <Button
                variant="primary"
                size="lg"
                onClick={handleSaveEdit}
                disabled={updateClient.isPending}
              >
                <Show when={updateClient.isPending} fallback="Save Changes">
                  <Spinner class="inline-block mr-2" />
                  Saving...
                </Show>
              </Button>
            </Show>
          </div>
        </Show>
      </div>

      {/* Error State */}
      <Show when={client.isError}>
        <Card class="p-6 bg-red-100 border-red-500">
          <p class="text-red-700 font-bold">
            Error loading client: {client.error?.message}
          </p>
          <Button 
            variant="secondary" 
            size="sm" 
            class="mt-4"
            onClick={() => client.refetch()}
          >
            Retry
          </Button>
        </Card>
      </Show>

      <Show when={client.data}>
        {/* Tab Navigation */}
        <div class="flex gap-2 mb-6 border-b-4 border-black">
          <button
            class={`px-6 py-3 font-heading font-bold uppercase border-t-4 border-l-4 border-r-4 border-black transition-all ${
              activeTab() === 'overview'
                ? 'bg-primary shadow-brutal -mb-1'
                : 'bg-white hover:bg-neutral-beige'
            }`}
            onClick={() => setActiveTab('overview')}
          >
            📋 Overview
          </button>
          <button
            class={`px-6 py-3 font-heading font-bold uppercase border-t-4 border-l-4 border-r-4 border-black transition-all ${
              activeTab() === 'tasks'
                ? 'bg-accent-yellow shadow-brutal -mb-1'
                : 'bg-white hover:bg-neutral-beige'
            }`}
            onClick={() => setActiveTab('tasks')}
          >
            📊 Tasks ({clientStats().totalTasks})
          </button>
          <button
            class={`px-6 py-3 font-heading font-bold uppercase border-t-4 border-l-4 border-r-4 border-black transition-all ${
              activeTab() === 'files'
                ? 'bg-secondary text-white shadow-brutal -mb-1'
                : 'bg-white hover:bg-neutral-beige'
            }`}
            onClick={() => setActiveTab('files')}
          >
            📁 Files
          </button>
          <button
            class={`px-6 py-3 font-heading font-bold uppercase border-t-4 border-l-4 border-r-4 border-black transition-all ${
              activeTab() === 'notes'
                ? 'bg-green-400 shadow-brutal -mb-1'
                : 'bg-white hover:bg-neutral-beige'
            }`}
            onClick={() => setActiveTab('notes')}
          >
            📝 Notes
          </button>
        </div>

        {/* Tab Content */}
        <div class="space-y-6">
          
          {/* Overview Tab */}
          <Show when={activeTab() === 'overview'}>
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* Client Information Card */}
              <Card>
                <CardHeader>
                  <CardTitle>Client Information</CardTitle>
                </CardHeader>
                <CardContent>
                  <Show when={!editMode()}>
                    <div class="space-y-4">
                      <div>
                        <label class="block font-bold text-sm text-neutral-darkGray mb-1">NAME</label>
                        <p class="font-heading font-bold text-lg">{client.data?.name}</p>
                      </div>
                      <div>
                        <label class="block font-bold text-sm text-neutral-darkGray mb-1">EMAIL</label>
                        <p class="font-mono">{client.data?.email || 'Not provided'}</p>
                      </div>
                      <div>
                        <label class="block font-bold text-sm text-neutral-darkGray mb-1">PHONE</label>
                        <p class="font-mono">{client.data?.phone || 'Not provided'}</p>
                      </div>
                      <div>
                        <label class="block font-bold text-sm text-neutral-darkGray mb-1">COMPANY</label>
                        <p class="font-heading font-bold">{client.data?.company || 'Not provided'}</p>
                      </div>
                      <div>
                        <label class="block font-bold text-sm text-neutral-darkGray mb-1">STATUS</label>
                        <Badge variant={client.data?.status === 'active' ? 'success' : 'secondary'}>
                          {client.data?.status?.toUpperCase()}
                        </Badge>
                      </div>
                      <div>
                        <label class="block font-bold text-sm text-neutral-darkGray mb-1">CREATED</label>
                        <p class="text-sm">{new Date(client.data?.created_at).toLocaleString()}</p>
                      </div>
                    </div>
                  </Show>

                  <Show when={editMode()}>
                    <div class="space-y-4">
                      <div>
                        <label class="block font-bold uppercase text-sm mb-2">Name *</label>
                        <Input
                          type="text"
                          value={editForm().name}
                          onInput={(e: any) => setEditForm(f => ({ ...f, name: e.currentTarget.value }))}
                          required
                        />
                      </div>
                      <div>
                        <label class="block font-bold uppercase text-sm mb-2">Email</label>
                        <Input
                          type="email"
                          value={editForm().email}
                          onInput={(e: any) => setEditForm(f => ({ ...f, email: e.currentTarget.value }))}
                        />
                      </div>
                      <div>
                        <label class="block font-bold uppercase text-sm mb-2">Phone</label>
                        <Input
                          type="tel"
                          value={editForm().phone}
                          onInput={(e: any) => setEditForm(f => ({ ...f, phone: e.currentTarget.value }))}
                        />
                      </div>
                      <div>
                        <label class="block font-bold uppercase text-sm mb-2">Company</label>
                        <Input
                          type="text"
                          value={editForm().company}
                          onInput={(e: any) => setEditForm(f => ({ ...f, company: e.currentTarget.value }))}
                        />
                      </div>
                      <div>
                        <label class="block font-bold uppercase text-sm mb-2">Status</label>
                        <select
                          class="w-full p-3 border-3 border-black font-mono"
                          value={editForm().status}
                          onChange={(e: any) => setEditForm(f => ({ ...f, status: e.currentTarget.value }))}
                        >
                          <option value="active">Active</option>
                          <option value="inactive">Inactive</option>
                        </select>
                      </div>
                    </div>
                  </Show>
                </CardContent>
              </Card>

              {/* Statistics Card */}
              <Card>
                <CardHeader>
                  <CardTitle>Activity Statistics</CardTitle>
                </CardHeader>
                <CardContent>
                  <div class="grid grid-cols-2 gap-4">
                    <div class="text-center p-4 bg-blue-100 border-3 border-blue-500">
                      <div class="text-2xl font-heading font-black text-blue-700">
                        {clientStats().totalTasks}
                      </div>
                      <div class="text-xs font-bold uppercase text-blue-600">Total Tasks</div>
                    </div>
                    <div class="text-center p-4 bg-green-100 border-3 border-green-500">
                      <div class="text-2xl font-heading font-black text-green-700">
                        {clientStats().completedTasks}
                      </div>
                      <div class="text-xs font-bold uppercase text-green-600">Completed</div>
                    </div>
                    <div class="text-center p-4 bg-yellow-100 border-3 border-yellow-500">
                      <div class="text-2xl font-heading font-black text-yellow-700">
                        {clientStats().pendingTasks}
                      </div>
                      <div class="text-xs font-bold uppercase text-yellow-600">Pending</div>
                    </div>
                    <div class="text-center p-4 bg-red-100 border-3 border-red-500">
                      <div class="text-2xl font-heading font-black text-red-700">
                        {clientStats().overdueTasks}
                      </div>
                      <div class="text-xs font-bold uppercase text-red-600">Overdue</div>
                    </div>
                  </div>

                  <div class="mt-6 pt-4 border-t-3 border-black">
                    <div class="flex justify-between items-center text-sm">
                      <span class="font-bold">Completion Rate</span>
                      <span class="font-black">
                        {clientStats().totalTasks > 0 
                          ? Math.round((clientStats().completedTasks / clientStats().totalTasks) * 100)
                          : 0}%
                      </span>
                    </div>
                    <div class="w-full bg-neutral-concrete h-4 border-3 border-black mt-2 overflow-hidden">
                      <div
                        class="h-full bg-green-500 transition-all duration-500"
                        style={{
                          width: `${clientStats().totalTasks > 0 
                            ? (clientStats().completedTasks / clientStats().totalTasks) * 100
                            : 0}%`,
                        }}
                      />
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          </Show>

          {/* Tasks Tab */}
          <Show when={activeTab() === 'tasks'}>
            <div>
              <div class="flex items-center justify-between mb-6">
                <h2 class="text-heading-2 font-heading font-bold uppercase">
                  Client Tasks
                </h2>
                <Button variant="primary" size="md">
                  ➕ New Task for This Client
                </Button>
              </div>

              <Show when={clientTasks.isPending}>
                <div class="flex justify-center p-8">
                  <Spinner />
                </div>
              </Show>

              <Show when={clientTasks.isError}>
                <Card class="p-6 bg-red-100 border-red-500">
                  <p class="text-red-700 font-bold">
                    Error loading tasks: {clientTasks.error?.message}
                  </p>
                </Card>
              </Show>

              <Show when={clientTasks.data}>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  <For each={clientTasks.data?.data || []}>
                    {(task: any) => (
                      <TaskCard
                        title={task.title}
                        description={task.description || ''}
                        priority={task.priority}
                        dueDate={task.due_date ? new Date(task.due_date).toLocaleDateString() : 'No due date'}
                        status={task.status}
                      />
                    )}
                  </For>
                </div>

                <Show when={(clientTasks.data?.data || []).length === 0}>
                  <Card class="text-center p-8">
                    <p class="text-neutral-darkGray font-bold text-lg mb-4">
                      No tasks found for this client
                    </p>
                    <Button variant="primary" size="lg">
                      Create First Task
                    </Button>
                  </Card>
                </Show>
              </Show>
            </div>
          </Show>

          {/* Files Tab */}
          <Show when={activeTab() === 'files'}>
            <Card class="text-center p-8">
              <div class="text-6xl mb-4">📁</div>
              <p class="text-neutral-darkGray font-bold text-lg mb-4">
                File management coming soon
              </p>
              <p class="text-sm text-neutral-darkGray">
                Upload and manage files related to this client
              </p>
            </Card>
          </Show>

          {/* Notes Tab */}
          <Show when={activeTab() === 'notes'}>
            <Card>
              <CardHeader>
                <CardTitle>Client Notes</CardTitle>
              </CardHeader>
              <CardContent>
                <Show when={!editMode()}>
                  <div class="whitespace-pre-wrap p-4 bg-neutral-beige border-3 border-black min-h-32">
                    {client.data?.notes || 'No notes added yet.'}
                  </div>
                  <Button
                    variant="primary"
                    size="md"
                    class="mt-4"
                    onClick={() => setEditMode(true)}
                  >
                    ✏️ Edit Notes
                  </Button>
                </Show>

                <Show when={editMode()}>
                  <textarea
                    class="w-full p-4 border-3 border-black font-mono min-h-32"
                    placeholder="Add notes about this client..."
                    value={editForm().notes}
                    onInput={(e: any) => setEditForm(f => ({ ...f, notes: e.currentTarget.value }))}
                  />
                </Show>
              </CardContent>
            </Card>
          </Show>
        </div>
      </Show>
    </div>
  );
};

export default ClientDetail;