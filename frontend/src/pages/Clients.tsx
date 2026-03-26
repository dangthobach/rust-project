import { Component, createSignal, For, Show } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { Card, CardHeader, CardTitle, CardContent, Button, Badge, Input, Spinner } from '~/components/ui';
import { ClientCard } from '~/components/crm';
import ExportButton from '~/components/ExportButton';
import { useClients, useDeleteClient } from '~/lib/hooks';
import { api } from '~/lib/api';
import { showToast } from '~/lib/toast';

const Clients: Component = () => {
  const navigate = useNavigate();
  const [page, setPage] = createSignal(1);
  const [search, setSearch] = createSignal('');
  const [status, setStatus] = createSignal('');

  // API hooks
  const clients = useClients(() => ({
    page: page(),
    limit: 12,
    search: search() || undefined,
    status: status() || undefined,
  }));

  const deleteClient = useDeleteClient();

  const [isExporting, setIsExporting] = createSignal(false);

  const handleExport = async (format: 'csv' | 'json' | 'pdf') => {
    setIsExporting(true);
    try {
      const blob = await api.exportClients(format, {
        status: status() || undefined,
      });
      
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `clients_export.${format}`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
      
      showToast('success', 'Export Successful', `Clients exported as ${format.toUpperCase()}`);
    } catch (error) {
      showToast('error', 'Export Failed', 'Failed to export clients');
    } finally {
      setIsExporting(false);
    }
  };

  const handleDeleteClient = (clientId: string) => {
    if (confirm('Are you sure you want to delete this client?')) {
      deleteClient.mutate(clientId);
    }
  };

  const handleSearch = (query: string) => {
    setSearch(query);
    setPage(1); // Reset to first page when searching
  };

  const handleStatusFilter = (statusFilter: string) => {
    setStatus(statusFilter);
    setPage(1);
  };

  return (
    <div>
      {/* Header */}
      <div class="flex items-center justify-between mb-8">
        <div>
          <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">
            Clients
          </h1>
          <p class="text-neutral-darkGray text-lg">
            Manage your client relationships
          </p>
        </div>
        
        <div class="flex gap-3">
          <ExportButton 
            onExport={handleExport}
            isExporting={isExporting()}
            label="Export Clients"
          />
          <Button 
            variant="primary" 
            size="lg"
            onClick={() => navigate('/clients/new')}
          >
            ➕ New Client
          </Button>
        </div>
      </div>

      {/* Filters */}
      <div class="flex flex-wrap gap-4 mb-6">
        <div class="flex-1 min-w-64">
          <Input
            type="text"
            placeholder="Search clients..."
            value={search()}
            onInput={(e: any) => handleSearch(e.currentTarget.value)}
          />
        </div>
        
        <div class="flex gap-2">
          <Button
            variant={status() === '' ? 'primary' : 'secondary'}
            onClick={() => handleStatusFilter('')}
          >
            All
          </Button>
          <Button
            variant={status() === 'active' ? 'primary' : 'secondary'}
            onClick={() => handleStatusFilter('active')}
          >
            Active
          </Button>
          <Button
            variant={status() === 'inactive' ? 'primary' : 'secondary'}
            onClick={() => handleStatusFilter('inactive')}
          >
            Inactive
          </Button>
        </div>
      </div>

      {/* Client List */}
      <Show when={clients.isPending}>
        <div class="flex justify-center p-8">
          <Spinner />
        </div>
      </Show>

      <Show when={clients.isError}>
        <Card class="p-6 bg-red-100 border-red-500">
          <p class="text-red-700 font-bold">
            Error loading clients: {clients.error?.message}
          </p>
          <Button 
            variant="secondary" 
            size="sm" 
            class="mt-4"
            onClick={() => clients.refetch()}
          >
            Retry
          </Button>
        </Card>
      </Show>

      <Show when={clients.data}>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <For each={clients.data?.data || []}>
            {(client: any) => (
              <div class="relative group">
                <ClientCard
                  id={client.id}
                  name={client.name}
                  email={client.email || ''}
                  phone={client.phone || ''}
                  status={client.status}
                  lastContact={new Date(client.created_at).toLocaleDateString()}
                  onView={() => navigate(`/clients/${client.id}`)}
                  onEdit={() => navigate(`/clients/${client.id}/edit`)}
                />
                
                {/* Action buttons */}
                <div class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  <Button
                    variant="primary"
                    size="sm"
                    class="mr-2"
                    onClick={() => navigate(`/clients/${client.id}/edit`)}
                  >
                    ✏️
                  </Button>
                  <Button
                    variant="secondary"
                    size="sm"
                    class="bg-red-500 hover:bg-red-600"
                    onClick={() => handleDeleteClient(client.id)}
                    disabled={deleteClient.isPending}
                  >
                    🗑️
                  </Button>
                </div>
              </div>
            )}
          </For>
        </div>

        {/* Pagination (approximate for CQRS list; disabled while searching) */}
        <Show when={clients.data?.pagination && search().trim().length === 0}>
          <div class="flex items-center justify-between mt-8">
            <p class="text-sm text-neutral-darkGray">
              Showing {clients.data?.data?.length || 0} of {clients.data?.pagination?.total || 0} clients
            </p>
            
            <div class="flex gap-2">
              <Button
                variant="secondary"
                disabled={!clients.data?.pagination?.has_prev}
                onClick={() => setPage(p => Math.max(1, p - 1))}
              >
                ← Previous
              </Button>
              
              <Badge variant="primary" class="px-4 py-2">
                Page {page()} of {clients.data?.pagination?.total_pages || 1}
              </Badge>
              
              <Button
                variant="secondary"
                disabled={!clients.data?.pagination?.has_next}
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

export default Clients;