import { Component, createEffect, createSignal, Show, onCleanup } from 'solid-js';
import { Card, CardContent, CardHeader, CardTitle, Badge, Button, Spinner } from '~/components/ui';
import { api, type ReportFormat, type ReportExportStatus, type ReportType } from '~/lib/api';

const Reports: Component = () => {
  const [reportType, setReportType] = createSignal<ReportType>('clients');
  const [format, setFormat] = createSignal<ReportFormat>('csv');

  const [jobId, setJobId] = createSignal<string | null>(null);
  const [status, setStatus] = createSignal<ReportExportStatus | null>(null);
  const [isPolling, setIsPolling] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  let interval: number | undefined;

  const stopPolling = () => {
    setIsPolling(false);
    if (interval) window.clearInterval(interval);
    interval = undefined;
  };

  const pollOnce = async (id: string) => {
    const res = await api.getReportExport(id);
    setStatus(res);

    if (res.status === 'ready' || res.status === 'failed') {
      stopPolling();
    }
  };

  createEffect(() => {
    const id = jobId();
    setError(null);

    if (!id) return;

    // If user starts a new job quickly, clear previous interval.
    if (interval) window.clearInterval(interval);

    setStatus(null);
    setIsPolling(true);

    // Immediate fetch + polling
    pollOnce(id).catch((e) => {
      setError(e?.message || String(e));
      stopPolling();
    });

    interval = window.setInterval(() => {
      const currentId = jobId();
      if (!currentId) return;
      if (!isPolling()) return;

      pollOnce(currentId).catch((e) => {
        setError(e?.message || String(e));
        stopPolling();
      });
    }, 2000);
  });

  onCleanup(() => stopPolling());

  const startExport = async () => {
    setError(null);
    setStatus(null);
    setJobId(null);

    const res = await api.startReportExport({
      report_type: reportType(),
      format: format(),
    });

    setJobId(res.job_id);
  };

  const currentStatus = status();
  const currentJobId = jobId();

  const badgeVariant = () => {
    if (!currentStatus) return 'default';
    switch (currentStatus.status) {
      case 'queued':
        return 'warning';
      case 'processing':
        return 'info';
      case 'ready':
        return 'success';
      case 'failed':
        return 'danger';
      default:
        return 'default';
    }
  };

  const isDone = currentStatus?.status === 'ready' || currentStatus?.status === 'failed';

  return (
    <div>
      <div class="mb-8">
        <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">
          Reports
        </h1>
        <p class="text-neutral-darkGray mt-1">
          Async export with queued/ready polling.
        </p>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-3 gap-4 mb-6">
        <Card class="border-5 border-black bg-white lg:col-span-1">
          <CardHeader>
            <CardTitle class="uppercase">Export Settings</CardTitle>
          </CardHeader>
          <CardContent class="space-y-4">
            <div>
              <label class="block text-sm font-bold mb-2">Report</label>
              <select
                class="w-full px-3 py-2 border-3 border-black font-bold bg-white cursor-pointer"
                value={reportType()}
                onChange={(e) => setReportType(e.currentTarget.value as ReportType)}
              >
                <option value="clients">Clients</option>
                <option value="tasks">Tasks</option>
                <option value="users">Users</option>
                <option value="dashboard">Dashboard Summary</option>
              </select>
            </div>

            <div>
              <label class="block text-sm font-bold mb-2">Format</label>
              <select
                class="w-full px-3 py-2 border-3 border-black font-bold bg-white cursor-pointer"
                value={format()}
                onChange={(e) => setFormat(e.currentTarget.value as ReportFormat)}
              >
                <option value="csv">CSV</option>
                <option value="json">JSON</option>
              </select>
            </div>

            <Button
              variant="primary"
              size="md"
              onClick={startExport}
              disabled={isPolling()}
              class="w-full"
            >
              Start Export
            </Button>
          </CardContent>
        </Card>

        <Card class="border-5 border-black bg-white lg:col-span-2">
          <CardHeader>
            <div class="flex items-center justify-between gap-3">
              <CardTitle class="uppercase">Job Status</CardTitle>
              <Show when={currentJobId}>
                <span class="text-xs font-bold text-neutral-darkGray break-all">
                  {currentJobId}
                </span>
              </Show>
            </div>
          </CardHeader>
          <CardContent>
            <Show when={!currentStatus && !error()}>
              <div class="text-center py-10">
                <div class="text-5xl mb-2">📤</div>
                <div class="font-bold">No export job yet</div>
                <div class="text-neutral-darkGray mt-1">Pick a report and click Start Export.</div>
              </div>
            </Show>

            <Show when={error()}>
              <div class="border-4 border-red-600 p-4 bg-red-50">
                <div class="font-bold mb-1">Export failed</div>
                <div class="text-sm text-neutral-darkGray">{error()}</div>
              </div>
            </Show>

            <Show when={currentStatus}>
              {(s) => (
                <div>
                  <div class="flex items-center gap-3 mb-4">
                    <Badge variant={badgeVariant()} class="border-3">
                      {s().status}
                    </Badge>
                    <Show when={s().status === 'queued' || s().status === 'processing'}>
                      <span class="flex items-center gap-2 text-neutral-darkGray">
                        <Spinner />
                        Polling...
                      </span>
                    </Show>
                  </div>

                  <Show when={s().status === 'ready'}>
                <div class="space-y-4">
                  <div class="font-bold text-lg">Report is ready</div>
                  <Show when={s().download_url}>
                    <Button
                      variant="secondary"
                      size="md"
                      onClick={() => window.open(s().download_url!, '_blank')}
                    >
                      Download Report
                    </Button>
                  </Show>
                  <Show when={!s().download_url}>
                    <div class="text-neutral-darkGray text-sm">
                      Download URL not available (object storage may be in local mode).
                    </div>
                  </Show>
                </div>
                  </Show>

                  <Show when={isDone && s().status === 'failed'}>
                <div class="space-y-2">
                  <div class="font-bold text-lg text-red-700">Job failed</div>
                  <div class="text-sm text-neutral-darkGray">
                    {s().error_message || 'Unknown error'}
                  </div>
                </div>
                  </Show>
                </div>
              )}
            </Show>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default Reports;

