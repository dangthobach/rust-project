import {
  Component,
  Show,
  For,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
} from 'solid-js';
import { createQuery, useQueryClient } from '@tanstack/solid-query';
import { Card, CardContent, CardHeader, CardTitle, Badge, Button, Spinner } from '~/components/ui';
import { api, type ReportFormat, type ReportExportStatus, type ReportType } from '~/lib/api';
import { useDashboardStats } from '~/lib/hooks';

function formatExportDate(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString();
}

function statusBadgeClass(status: string): string {
  const s = status.toLowerCase();
  if (s === 'ready') return 'border-2 border-black bg-ledger-electric px-2 py-0.5 text-[10px] font-bold uppercase text-white';
  if (s === 'failed') return 'border-2 border-black bg-red-600 px-2 py-0.5 text-[10px] font-bold uppercase text-white';
  if (s === 'processing' || s === 'queued')
    return 'border-2 border-black bg-ledger-pale px-2 py-0.5 text-[10px] font-bold uppercase text-black';
  return 'border-2 border-black bg-neutral-lightGray px-2 py-0.5 text-[10px] font-bold uppercase';
}

const Reports: Component = () => {
  const qc = useQueryClient();
  const dashboard = useDashboardStats();

  const [reportType, setReportType] = createSignal<ReportType>('clients');
  const [format, setFormat] = createSignal<ReportFormat>('csv');
  const [startDate, setStartDate] = createSignal('');
  const [endDate, setEndDate] = createSignal('');

  const [jobId, setJobId] = createSignal<string | null>(null);
  const [exportJobStatus, setExportJobStatus] = createSignal<ReportExportStatus | null>(null);
  const [isPolling, setIsPolling] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  let interval: number | undefined;

  const exportList = createQuery(() => ({
    queryKey: ['reports', 'exports'],
    queryFn: () => api.listReportExports({ page: 1, limit: 50 }),
  }));

  createEffect(() => {
    if (!isPolling()) return;
    const t = window.setInterval(() => {
      void qc.invalidateQueries({ queryKey: ['reports', 'exports'] });
    }, 2000);
    onCleanup(() => window.clearInterval(t));
  });

  const metrics = createMemo(() => {
    const d = dashboard.data as
      | {
          clients?: {
            total?: number;
            active?: number;
            pipeline_open?: number;
          };
          tasks?: { total?: number; completed?: number };
        }
      | undefined;
    if (!d) {
      return {
        conversion: 0,
        pipelineOpen: 0,
        activeLeads: 0,
        efficiency: 0,
      };
    }
    const c = d.clients ?? {};
    const t = d.tasks ?? {};
    const totalC = Math.max(Number(c.total), 1);
    const conversion = Math.round((Number(c.active) / totalC) * 1000) / 10;
    const totalT = Math.max(Number(t.total), 1);
    const efficiency = Math.round((Number(t.completed) / totalT) * 1000) / 10;
    return {
      conversion,
      pipelineOpen: Number(c.pipeline_open ?? 0),
      activeLeads: Number(c.active ?? 0),
      efficiency,
    };
  });

  const stopPolling = () => {
    setIsPolling(false);
    if (interval) window.clearInterval(interval);
    interval = undefined;
  };

  const pollOnce = async (id: string) => {
    const res = await api.getReportExport(id);
    setExportJobStatus(res);

    if (res.status === 'ready' || res.status === 'failed') {
      stopPolling();
      void qc.invalidateQueries({ queryKey: ['reports', 'exports'] });
    }
  };

  createEffect(() => {
    const id = jobId();
    setError(null);

    if (!id) return;

    if (interval) window.clearInterval(interval);

    setExportJobStatus(null);
    setIsPolling(true);

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
    setExportJobStatus(null);
    setJobId(null);

    const sd = startDate().trim();
    const ed = endDate().trim();

    try {
      const res = await api.startReportExport({
        report_type: reportType(),
        format: format(),
        start_date: sd || undefined,
        end_date: ed || undefined,
      });

      setJobId(res.job_id);
      void qc.invalidateQueries({ queryKey: ['reports', 'exports'] });
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    }
  };

  const isDone = () => {
    const st = exportJobStatus();
    return st?.status === 'ready' || st?.status === 'failed';
  };

  const formatBtn = (key: ReportFormat | 'pdf', label: string, disabled?: boolean) => {
    const active = key !== 'pdf' && format() === key;
    return (
      <button
        type="button"
        disabled={disabled}
        class={[
          'flex-1 border-[3px] border-black px-3 py-2 font-heading text-[10px] font-black uppercase tracking-wide transition-all',
          disabled ? 'cursor-not-allowed bg-neutral-lightGray text-neutral-gray' : '',
          !disabled && active ? 'bg-ledger-lime text-black shadow-brutal-sm' : '',
          !disabled && !active ? 'bg-white hover:bg-ledger-pale' : '',
        ].join(' ')}
        onClick={() => {
          if (key !== 'pdf') setFormat(key as ReportFormat);
        }}
      >
        {label}
      </button>
    );
  };

  const rows = () => exportList.data?.data ?? [];

  return (
    <div class="font-body">
      <div class="mb-8 max-w-4xl">
        <h1 class="font-heading text-2xl font-black uppercase tracking-tight text-shadow-brutal sm:text-heading-1">
          Reports &amp; analytics
        </h1>
        <p class="mt-2 font-mono text-sm font-medium uppercase tracking-wide text-neutral-darkGray">
          Export pipeline data, track async jobs, and archive deliverables — industrial-grade reporting.
        </p>
      </div>

      <div class="mb-8 grid grid-cols-2 gap-3 lg:grid-cols-4">
        <div class="border-[3px] border-black bg-white p-4 shadow-brutal-sm">
          <div class="mb-2 font-mono text-[10px] font-bold uppercase text-neutral-darkGray">Conversion rate</div>
          <div class="flex items-end justify-between gap-2">
            <span class="font-heading text-3xl font-black tabular-nums">{metrics().conversion}%</span>
            <span class="text-2xl" aria-hidden="true">
              📊
            </span>
          </div>
        </div>
        <div class="border-[3px] border-black bg-ledger-electric p-4 text-white shadow-brutal-sm">
          <div class="mb-2 font-mono text-[10px] font-bold uppercase text-white/80">Pipeline (open)</div>
          <div class="font-heading text-3xl font-black tabular-nums">{metrics().pipelineOpen}</div>
          <div class="mt-1 font-mono text-[10px] font-semibold uppercase text-white/70">
            Prospect + active clients
          </div>
        </div>
        <div class="border-[3px] border-black bg-ledger-lime p-4 shadow-brutal-sm">
          <div class="mb-2 flex items-center justify-between">
            <span class="font-mono text-[10px] font-bold uppercase text-black/80">Active leads</span>
            <Badge variant="success" class="border-2 border-black !bg-black !text-ledger-lime">
              Live
            </Badge>
          </div>
          <div class="font-heading text-3xl font-black tabular-nums text-black">{metrics().activeLeads}</div>
        </div>
        <div class="border-[3px] border-black bg-white p-4 shadow-brutal-sm">
          <div class="mb-2 font-mono text-[10px] font-bold uppercase text-neutral-darkGray">Task efficiency</div>
          <div class="flex items-center gap-2">
            <span class="font-heading text-3xl font-black tabular-nums">{metrics().efficiency}%</span>
            <span class="text-2xl" aria-hidden="true">
              ⚡
            </span>
          </div>
          <div class="mt-2 h-2 w-full border-2 border-black bg-neutral-lightGray">
            <div
              class="h-full bg-ledger-lime transition-all"
              style={{ width: `${Math.min(100, metrics().efficiency)}%` }}
            />
          </div>
        </div>
      </div>

      <div class="grid grid-cols-1 gap-6 xl:grid-cols-5">
        <div class="space-y-4 xl:col-span-2">
          <Card class="border-[3px] border-black bg-white shadow-brutal">
            <CardHeader>
              <CardTitle class="font-mono text-xs uppercase tracking-widest">Report generator</CardTitle>
            </CardHeader>
            <CardContent class="space-y-5">
              <div>
                <label class="mb-2 block font-heading text-xs font-black uppercase">Select report type</label>
                <select
                  class="select w-full border-[3px] border-black bg-white py-3 font-mono text-sm font-bold uppercase"
                  value={reportType()}
                  onChange={(e) => setReportType(e.currentTarget.value as ReportType)}
                >
                  <option value="clients">Clients</option>
                  <option value="tasks">Tasks</option>
                  <option value="users">Users</option>
                  <option value="dashboard">Dashboard summary</option>
                </select>
              </div>

              <div>
                <span class="mb-2 block font-heading text-xs font-black uppercase">Output format</span>
                <div class="flex gap-0 border-[3px] border-black bg-white">
                  {formatBtn('pdf', 'PDF', true)}
                  {formatBtn('csv', 'CSV')}
                  {formatBtn('json', 'JSON')}
                </div>
                <p class="mt-1 font-mono text-[10px] text-neutral-darkGray">PDF export is not wired yet — use CSV or JSON.</p>
              </div>

              <div>
                <span class="mb-2 block font-heading text-xs font-black uppercase">Date range</span>
                <div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
                  <input
                    type="date"
                    class="input border-[3px] border-black py-2 font-mono text-xs"
                    value={startDate()}
                    onInput={(e) => setStartDate(e.currentTarget.value)}
                  />
                  <input
                    type="date"
                    class="input border-[3px] border-black py-2 font-mono text-xs"
                    value={endDate()}
                    onInput={(e) => setEndDate(e.currentTarget.value)}
                  />
                </div>
                <p class="mt-1 font-mono text-[10px] text-neutral-darkGray">
                  Filters exported rows by <span class="font-bold">created_at</span> (inclusive dates).
                </p>
              </div>

              <Button
                variant="primary"
                size="md"
                onClick={startExport}
                disabled={isPolling()}
                class="w-full !bg-ledger-lime !text-black hover:!bg-primary-dark"
              >
                🚀 Start export
              </Button>
            </CardContent>
          </Card>

          <div class="border-[3px] border-black bg-ledger-lime p-4 shadow-brutal-sm">
            <div class="font-heading text-xs font-black uppercase">Automated scheduling</div>
            <p class="mt-1 font-mono text-[10px] font-medium text-black/80">
              Schedule recurring exports to your inbox or SFTP bucket.
            </p>
            <button
              type="button"
              disabled
              class="mt-3 w-full border-[3px] border-black bg-white py-2 font-heading text-xs font-black uppercase shadow-brutal-sm opacity-60"
            >
              Configure schedule
            </button>
          </div>
        </div>

        <Card class="border-[3px] border-black bg-white shadow-brutal xl:col-span-3">
          <CardHeader>
            <div class="flex flex-wrap items-center justify-between gap-3">
              <CardTitle class="font-mono text-xs uppercase tracking-widest">Job status &amp; history</CardTitle>
              <Show when={jobId()}>
                <span class="max-w-[min(100%,280px)] truncate font-mono text-[10px] font-semibold text-neutral-darkGray">
                  {jobId()}
                </span>
              </Show>
            </div>
          </CardHeader>
          <CardContent>
            <Show when={exportList.isPending}>
              <div class="flex justify-center py-8">
                <Spinner />
              </div>
            </Show>

            <div class="overflow-x-auto border-[3px] border-black">
              <table class="w-full min-w-[520px] border-collapse font-mono text-xs">
                <thead>
                  <tr class="bg-neutral-lightGray">
                    <th class="border-b-[3px] border-black px-3 py-2 text-left font-heading text-[10px] font-black uppercase">
                      Filename
                    </th>
                    <th class="border-b-[3px] border-black px-3 py-2 text-left font-heading text-[10px] font-black uppercase">
                      Status
                    </th>
                    <th class="border-b-[3px] border-black px-3 py-2 text-left font-heading text-[10px] font-black uppercase">
                      Date
                    </th>
                    <th class="border-b-[3px] border-black px-3 py-2 text-right font-heading text-[10px] font-black uppercase">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody>
                  <For each={rows()}>
                    {(row) => (
                      <tr class="bg-white">
                        <td class="border-b-[3px] border-black px-3 py-3 font-bold">
                          {row.report_type}_export.{row.format}
                          <Show when={row.start_date || row.end_date}>
                            <div class="mt-1 text-[10px] font-normal text-neutral-darkGray">
                              {row.start_date ?? '…'} → {row.end_date ?? '…'}
                            </div>
                          </Show>
                        </td>
                        <td class="border-b-[3px] border-black px-3 py-3">
                          <div class="flex flex-wrap items-center gap-2">
                            <span class={statusBadgeClass(row.status)}>{row.status}</span>
                            <Show when={jobId() === row.job_id && (exportJobStatus()?.status === 'queued' || exportJobStatus()?.status === 'processing')}>
                              <Spinner />
                            </Show>
                          </div>
                        </td>
                        <td class="border-b-[3px] border-black px-3 py-3 text-neutral-darkGray">
                          {formatExportDate(row.created_at)}
                        </td>
                        <td class="border-b-[3px] border-black px-3 py-3 text-right">
                          <Show when={row.download_url}>
                            <button
                              type="button"
                              class="mr-2 inline-flex h-8 w-8 items-center justify-center border-2 border-black bg-ledger-electric text-white"
                              onClick={() => window.open(row.download_url!, '_blank')}
                              aria-label="Download"
                            >
                              ⬇
                            </button>
                          </Show>
                        </td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>

            <Show when={!exportList.isPending && rows().length === 0}>
              <div class="mt-4 text-center font-mono text-xs text-neutral-darkGray">
                No exports yet — use <span class="font-bold text-black">Start export</span> to queue one.
              </div>
            </Show>

            <Show when={error()}>
              <div class="mt-4 border-[3px] border-red-600 bg-red-50 p-4">
                <div class="font-heading font-bold uppercase text-red-800">Export failed</div>
                <div class="mt-1 font-mono text-sm text-neutral-darkGray">{error()}</div>
              </div>
            </Show>

            <Show when={exportJobStatus()}>
              {(s) => (
                <div class="mt-6 space-y-4 border-t-[3px] border-black pt-4">
                  <Show when={isDone() && s().status === 'failed'}>
                    <div>
                      <div class="font-heading font-bold uppercase text-red-700">Job failed</div>
                      <div class="font-mono text-sm text-neutral-darkGray">{s().error_message || 'Unknown error'}</div>
                    </div>
                  </Show>

                  <Show when={s().status === 'ready' && !s().download_url}>
                    <div class="font-mono text-sm text-neutral-darkGray">
                      Ready — download URL unavailable (local / dev storage).
                    </div>
                  </Show>
                </div>
              )}
            </Show>

            <div class="mt-6 flex justify-end">
              <button
                type="button"
                class="border-[3px] border-black bg-white px-4 py-2 font-heading text-xs font-black uppercase shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5"
                onClick={() => void qc.invalidateQueries({ queryKey: ['reports', 'exports'] })}
              >
                Refresh archive
              </button>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default Reports;
