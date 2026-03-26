import { Component, Show } from 'solid-js';
import { useNavigate, useParams } from '@solidjs/router';
import { Card, CardHeader, CardTitle, CardContent, Button, Badge, Spinner } from '~/components/ui';
import { useClient, useDeleteClient } from '~/lib/hooks/useClients';

const ClientDetail: Component = () => {
  const params = useParams();
  const navigate = useNavigate();
  const id = () => params.id;

  const client = useClient(id);
  const deleteClient = useDeleteClient();

  const onDelete = () => {
    if (!id()) return;
    if (!confirm('Delete this client?')) return;
    deleteClient.mutate(id()!, {
      onSuccess: () => navigate('/clients', { replace: true }),
    });
  };

  return (
    <div class="max-w-4xl">
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">Client</h1>
          <p class="text-neutral-darkGray break-all">{id()}</p>
        </div>
        <div class="flex gap-2">
          <Button variant="secondary" onClick={() => navigate('/clients')}>
            ← Back
          </Button>
          <Button variant="primary" onClick={() => navigate(`/clients/${id()}/edit`)}>
            ✏️ Edit
          </Button>
          <Button variant="secondary" class="bg-red-500 hover:bg-red-600" onClick={onDelete} disabled={deleteClient.isPending}>
            <Show when={deleteClient.isPending} fallback="🗑️ Delete">
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
            <Show when={client.data?.status}>
              <Badge variant="primary" class="border-3">
                {client.data!.status}
              </Badge>
            </Show>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Show
            when={!client.isPending && !client.isError && !!client.data}
            fallback={
              <div class="py-8">
                <Show when={client.isPending}>
                  <div class="flex items-center gap-2">
                    <Spinner />
                    <span class="font-bold">Loading...</span>
                  </div>
                </Show>
                <Show when={client.isError}>
                  <div class="p-3 bg-red-100 border-3 border-red-500 text-red-700 text-sm font-bold">
                    {client.error?.message}
                  </div>
                </Show>
              </div>
            }
          >
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="p-4 border-3 border-black bg-white">
                <div class="text-xs font-bold uppercase text-neutral-darkGray">Name</div>
                <div class="font-heading font-black text-xl">{client.data!.name}</div>
              </div>
              <div class="p-4 border-3 border-black bg-white">
                <div class="text-xs font-bold uppercase text-neutral-darkGray">Email</div>
                <div class="font-mono break-all">{client.data!.email || '-'}</div>
              </div>
              <div class="p-4 border-3 border-black bg-white">
                <div class="text-xs font-bold uppercase text-neutral-darkGray">Phone</div>
                <div class="font-mono">{client.data!.phone || '-'}</div>
              </div>
              <div class="p-4 border-3 border-black bg-white">
                <div class="text-xs font-bold uppercase text-neutral-darkGray">Company</div>
                <div class="font-mono">{client.data!.company || '-'}</div>
              </div>
              <div class="p-4 border-3 border-black bg-white md:col-span-2">
                <div class="text-xs font-bold uppercase text-neutral-darkGray">Notes</div>
                <div class="whitespace-pre-wrap">{(client.data as any)?.notes || '-'}</div>
              </div>
            </div>
          </Show>
        </CardContent>
      </Card>
    </div>
  );
};

export default ClientDetail;