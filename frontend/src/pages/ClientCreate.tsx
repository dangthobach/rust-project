import { Component, createSignal, Show } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { Card, CardContent, CardHeader, CardTitle, Button, Input, Spinner, Badge } from '~/components/ui';
import { useCreateClient } from '~/lib/hooks/useClients';

const ClientCreate: Component = () => {
  const navigate = useNavigate();
  const createClient = useCreateClient();

  const [form, setForm] = createSignal({
    name: '',
    email: '',
    phone: '',
    company: '',
    position: '',
    address: '',
    status: 'active' as const,
    notes: '',
    assigned_to: '',
  });

  const canSubmit = () => form().name.trim().length >= 2;

  const onSubmit = () => {
    if (!canSubmit()) return;
    const f = form();
    createClient.mutate(
      {
        name: f.name.trim(),
        email: f.email.trim() || undefined,
        phone: f.phone.trim() || undefined,
        company: f.company.trim() || undefined,
        position: f.position.trim() || undefined,
        address: f.address.trim() || undefined,
        status: f.status,
        notes: f.notes.trim() || undefined,
        assigned_to: f.assigned_to.trim() || undefined,
      },
      {
        onSuccess: (client) => {
          navigate(`/clients/${client.id}`, { replace: true });
        },
      },
    );
  };

  return (
    <div class="max-w-3xl">
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">New Client</h1>
          <p class="text-neutral-darkGray">Create a client record</p>
        </div>
        <div class="flex gap-2">
          <Button variant="secondary" onClick={() => navigate('/clients')}>
            ← Back
          </Button>
          <Button variant="primary" onClick={onSubmit} disabled={createClient.isPending || !canSubmit()}>
            <Show when={createClient.isPending} fallback="Create">
              <Spinner class="inline-block mr-2" />
              Creating...
            </Show>
          </Button>
        </div>
      </div>

      <Card class="border-5">
        <CardHeader>
          <CardTitle class="flex items-center justify-between">
            <span>Client Details</span>
            <Badge variant="info" class="border-3">
              page-based form
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label class="block font-bold uppercase text-sm mb-2">Name *</label>
              <Input
                type="text"
                value={form().name}
                onInput={(e: any) => setForm((p) => ({ ...p, name: e.currentTarget.value }))}
                placeholder="Client name"
              />
            </div>
            <div>
              <label class="block font-bold uppercase text-sm mb-2">Status</label>
              <select
                class="w-full px-3 py-2 border-3 border-black font-mono bg-white"
                value={form().status}
                onChange={(e: any) => setForm((p) => ({ ...p, status: e.currentTarget.value }))}
              >
                <option value="active">Active</option>
                <option value="inactive">Inactive</option>
                <option value="prospect">Prospect</option>
                <option value="customer">Customer</option>
              </select>
            </div>
            <div>
              <label class="block font-bold uppercase text-sm mb-2">Email</label>
              <Input
                type="email"
                value={form().email}
                onInput={(e: any) => setForm((p) => ({ ...p, email: e.currentTarget.value }))}
                placeholder="client@example.com"
              />
            </div>
            <div>
              <label class="block font-bold uppercase text-sm mb-2">Phone</label>
              <Input
                type="tel"
                value={form().phone}
                onInput={(e: any) => setForm((p) => ({ ...p, phone: e.currentTarget.value }))}
                placeholder="+84 ..."
              />
            </div>
            <div>
              <label class="block font-bold uppercase text-sm mb-2">Company</label>
              <Input
                type="text"
                value={form().company}
                onInput={(e: any) => setForm((p) => ({ ...p, company: e.currentTarget.value }))}
                placeholder="Company name"
              />
            </div>
            <div>
              <label class="block font-bold uppercase text-sm mb-2">Position</label>
              <Input
                type="text"
                value={form().position}
                onInput={(e: any) => setForm((p) => ({ ...p, position: e.currentTarget.value }))}
                placeholder="Role / title"
              />
            </div>
            <div class="md:col-span-2">
              <label class="block font-bold uppercase text-sm mb-2">Address</label>
              <Input
                type="text"
                value={form().address}
                onInput={(e: any) => setForm((p) => ({ ...p, address: e.currentTarget.value }))}
                placeholder="Address"
              />
            </div>
            <div class="md:col-span-2">
              <label class="block font-bold uppercase text-sm mb-2">Assigned To (UUID)</label>
              <Input
                type="text"
                value={form().assigned_to}
                onInput={(e: any) => setForm((p) => ({ ...p, assigned_to: e.currentTarget.value }))}
                placeholder="optional (UUID)"
              />
            </div>
          </div>

          <div>
            <label class="block font-bold uppercase text-sm mb-2">Notes</label>
            <textarea
              class="w-full p-3 border-3 border-black font-mono"
              rows="4"
              value={form().notes}
              onInput={(e: any) => setForm((p) => ({ ...p, notes: e.currentTarget.value }))}
              placeholder="Notes..."
            />
          </div>

          <Show when={createClient.isError}>
            <div class="p-3 bg-red-100 border-3 border-red-500 text-red-700 text-sm font-bold">
              {createClient.error?.message}
            </div>
          </Show>
        </CardContent>
      </Card>
    </div>
  );
};

export default ClientCreate;

