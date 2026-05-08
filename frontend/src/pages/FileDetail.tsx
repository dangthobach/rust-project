import { Component, For, Show, createEffect, createMemo, createSignal, onCleanup, onMount } from 'solid-js';
import { A, useParams } from '@solidjs/router';
import { useDownloadFile, useFile, useFileActivity, useFileDownloadUrl, useFileVersions, useRollbackVersion } from '~/lib/hooks/useFiles';
import { Spinner } from '~/components/ui';
import { useQueryClient } from '@tanstack/solid-query';
import initWasm, { FileViewer } from '~/wasm/wasm_file_viewer';

const MaterialIcon: Component<{ name: string; class?: string }> = (props) => (
  <span class={['material-symbols-outlined', props.class ?? ''].join(' ')} aria-hidden="true">
    {props.name}
  </span>
);

function formatDateTiny(dateStr: string) {
  const d = new Date(dateStr);
  if (Number.isNaN(d.getTime())) return dateStr;
  return d.toLocaleString(undefined, { year: 'numeric', month: 'short', day: '2-digit' });
}

function formatFileSize(bytes: number) {
  if (!Number.isFinite(bytes)) return '-';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

const FileDetail: Component = () => {
  const params = useParams<{ id: string }>();
  const fileId = () => params.id;

  const file = useFile(fileId, () => !!fileId());
  const download = useDownloadFile();
  const versions = useFileVersions(fileId, () => !!fileId());
  const rollback = useRollbackVersion();
  const activity = useFileActivity(fileId, () => !!fileId());
  const qc = useQueryClient();

  const isImage = createMemo(() => ((file.data as any)?.mime_type ?? '').startsWith('image/'));
  // Always get download url to feed to wasm if not image
  const dlUrl = useFileDownloadUrl(fileId, () => !!fileId());
  const [forceProxy, setForceProxy] = createSignal(false);
  const proxySrc = createMemo(() => `/api/fs/files/${fileId()}/download`);
  const imageSrc = createMemo(() => {
    const url = (dlUrl.data as any)?.download_url as string | undefined;
    if (!forceProxy() && url) return url;
    return proxySrc();
  });

  const [wasmHtml, setWasmHtml] = createSignal<string | null>(null);
  const [wasmLoading, setWasmLoading] = createSignal(false);

  let refreshTimer: number | undefined;
  createEffect(() => {
    // Only refresh when using presigned URL.
    if (forceProxy()) return;
    if (!dlUrl.data) return;
    if (refreshTimer) window.clearTimeout(refreshTimer);
    const expiresIn = Number((dlUrl.data as any).expires_in ?? 0);
    const refreshInMs = Math.max(5_000, (expiresIn - 60) * 1000);
    if (expiresIn > 0) {
      refreshTimer = window.setTimeout(() => {
        qc.invalidateQueries({ queryKey: ['files', 'download-url', fileId()] });
      }, refreshInMs);
    }
  });

  createEffect(() => {
    if (dlUrl.isError) {
      setForceProxy(true);
    } else if ((dlUrl.data as any)?.download_url) {
      setForceProxy(false);
    }
  });

  // Load WASM and file content when it's not an image
  createEffect(() => {
    const f = file.data as any;
    if (!f || isImage()) return;
    const url = imageSrc();
    if (!url || dlUrl.isLoading) return;

    let isActive = true;
    setWasmLoading(true);

    const load = async () => {
      try {
        await initWasm();
        
        // Fetch with auth token if using proxy
        const token = localStorage.getItem('auth_token');
        const headers = new Headers();
        if (token && url.startsWith('/api')) {
          headers.set('Authorization', `Bearer ${token}`);
        }
        
        const res = await fetch(url, { headers });
        if (!res.ok) throw new Error('Failed to fetch file');
        
        const buffer = await res.arrayBuffer();
        if (!isActive) return;

        const viewer = new FileViewer(f.mime_type || f.file_type);
        viewer.load_content(new Uint8Array(buffer));
        setWasmHtml(viewer.render());
      } catch (err) {
        console.error('WASM load error:', err);
        if (isActive) setWasmHtml('<div class="text-red-500 font-bold">Failed to load preview via WASM</div>');
      } finally {
        if (isActive) setWasmLoading(false);
      }
    };

    load();

    onCleanup(() => {
      isActive = false;
    });
  });

  onCleanup(() => {
    if (refreshTimer) window.clearTimeout(refreshTimer);
  });

  const title = createMemo(() => (file.data as any)?.name || `FILE_${fileId()}`);
  const breadcrumbs = createMemo(() => {
    return ['Documents', 'Engineering', title()];
  });

  const handleDownload = () => {
    const f = file.data as any;
    if (!f) return;
    download.mutate({ id: f.id, filename: f.name });
  };

  return (
    <div class="space-y-6">
      <div class="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
        <div class="min-w-0">
          <nav class="flex flex-wrap gap-2 text-[10px] font-heading font-black uppercase text-neutral-gray">
            <For each={breadcrumbs()}>
              {(item, idx) => (
                <>
                  <span class={idx() === breadcrumbs().length - 1 ? 'text-black' : ''}>{item}</span>
                  <Show when={idx() !== breadcrumbs().length - 1}>
                    <span>/</span>
                  </Show>
                </>
              )}
            </For>
          </nav>
          <h1 class="mt-2 text-heading-2 font-heading font-black uppercase tracking-tight">{title()}</h1>
          <Show when={file.data}>
            {(f: any) => (
              <div class="mt-2 flex flex-wrap gap-3 text-[10px] font-heading font-bold uppercase text-neutral-darkGray">
                <span class="border-2 border-black bg-white px-2 py-1">ID: {f.id}</span>
                <span class="border-2 border-black bg-white px-2 py-1">{formatFileSize(f.size || f.file_size)}</span>
                <span class="border-2 border-black bg-white px-2 py-1">{f.mime_type || f.file_type}</span>
                <span class="border-2 border-black bg-white px-2 py-1">{formatDateTiny(f.created_at)}</span>
              </div>
            )}
          </Show>
        </div>

        <div class="flex gap-3">
          <button
            type="button"
            class="inline-flex items-center gap-2 border-[3px] border-black bg-white px-4 py-2 font-heading text-sm font-black uppercase shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5"
            onClick={handleDownload}
            disabled={download.isPending || !file.data}
          >
            <MaterialIcon name="download" />
            Download
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-2 border-[3px] border-black bg-secondary px-4 py-2 font-heading text-sm font-black uppercase text-white shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5"
            onClick={() => alert('Share (mock)')}
          >
            <MaterialIcon name="share" />
            Share
          </button>
        </div>
      </div>

      <Show
        when={!file.isLoading}
        fallback={
          <div class="flex items-center justify-center border-[3px] border-black bg-white p-10 shadow-brutal-sm">
            <Spinner />
          </div>
        }
      >
        <Show
          when={file.data}
          fallback={
            <div class="border-[3px] border-black bg-white p-8 shadow-brutal-sm">
              <div class="text-sm font-bold">File not found.</div>
              <div class="mt-4">
                <A href="/files" class="btn btn-sm no-underline">
                  Back to files
                </A>
              </div>
            </div>
          }
        >
          <div class="grid grid-cols-12 gap-6">
            {/* Preview + Activity (left bento) */}
            <div class="col-span-12 lg:col-span-8 flex flex-col gap-6">
              <div class="relative min-h-[620px] border-[3px] border-black bg-white p-8 shadow-brutal-sm flex flex-col">
                <div class="absolute right-4 top-4 flex gap-2 z-10">
                  <button
                    type="button"
                    class="flex h-10 w-10 items-center justify-center border-[3px] border-black bg-white hover:bg-primary"
                    onClick={() => alert('Zoom in (mock)')}
                  >
                    <MaterialIcon name="zoom_in" />
                  </button>
                  <button
                    type="button"
                    class="flex h-10 w-10 items-center justify-center border-[3px] border-black bg-white hover:bg-primary"
                    onClick={() => alert('Zoom out (mock)')}
                  >
                    <MaterialIcon name="zoom_out" />
                  </button>
                </div>

                <Show
                  when={isImage()}
                  fallback={
                    <div class="flex-grow flex flex-col max-h-[620px]">
                      <Show when={wasmLoading()}>
                        <div class="flex-grow flex items-center justify-center">
                          <Spinner />
                        </div>
                      </Show>
                      <Show when={!wasmLoading() && wasmHtml()}>
                        <div class="overflow-auto border-2 border-black p-4 flex-grow bg-neutral-lightGray/20 text-sm" innerHTML={wasmHtml()!} />
                      </Show>
                      <Show when={!wasmLoading() && !wasmHtml()}>
                        <div class="mx-auto max-w-2xl space-y-6 font-body text-neutral-darkGray mt-12">
                          <div class="border-b-[3px] border-black pb-4">
                            <div class="text-xs font-heading font-black uppercase text-neutral-gray">
                              Technical Specifications: Ledger Alpha
                            </div>
                            <div class="mt-2 text-2xl font-heading font-black uppercase text-black">
                              {title()}
                            </div>
                            <div class="mt-1 text-[10px] font-heading font-bold uppercase text-neutral-gray">
                              DOCUMENT ID: #{file.data!.id.slice(0, 8).toUpperCase()} • CLASSIFICATION: CONFIDENTIAL
                            </div>
                          </div>
                          <p class="leading-relaxed">
                            Preview not available for this file type. Please download the file to view it.
                          </p>
                        </div>
                      </Show>
                    </div>
                  }
                >
                  <Show
                    when={!dlUrl.isLoading && (forceProxy() || (dlUrl.data as any)?.download_url)}
                    fallback={
                      <div class="flex items-center justify-center py-20">
                        <Spinner />
                      </div>
                    }
                  >
                    <div class="flex items-center justify-center flex-grow">
                      <img
                        src={imageSrc()}
                        alt={title()}
                        class="max-h-[520px] w-full max-w-3xl border-[3px] border-black object-contain bg-white"
                        onError={() => {
                          if (!forceProxy()) {
                            qc.invalidateQueries({ queryKey: ['files', 'download-url', fileId()] });
                            setForceProxy(true);
                          }
                        }}
                      />
                    </div>
                  </Show>
                </Show>
              </div>

              <div class="border-[3px] border-black bg-white p-6 shadow-brutal-sm">
                <div class="mb-4 flex items-center gap-2">
                  <MaterialIcon name="history_edu" class="text-black" />
                  <div class="text-lg font-heading font-black uppercase">Activity Log</div>
                </div>
                <Show
                  when={!activity.isLoading}
                  fallback={
                    <div class="flex items-center justify-center py-10">
                      <Spinner />
                    </div>
                  }
                >
                  <Show
                    when={(activity.data ?? []).length > 0}
                    fallback={<div class="text-xs font-bold text-neutral-darkGray">No activity yet.</div>}
                  >
                    <div class="space-y-4">
                      <For each={activity.data ?? []}>
                        {(row: any) => (
                          <div class="flex items-center justify-between border-b-2 border-dashed border-neutral-lightGray py-2">
                            <div class="flex items-center gap-3">
                              <MaterialIcon name="history" class="text-secondary" />
                              <div class="text-sm font-bold text-black">{row.action}</div>
                            </div>
                            <div class="text-[10px] font-heading font-bold uppercase text-neutral-gray">
                              {formatDateTiny(row.created_at)}
                            </div>
                          </div>
                        )}
                      </For>
                    </div>
                  </Show>
                </Show>
              </div>
            </div>

            {/* Side panels (right bento) */}
            <div class="col-span-12 lg:col-span-4 flex flex-col gap-6">
              <div class="border-[3px] border-black bg-background p-6 shadow-brutal-sm">
                <div class="mb-6 flex items-center justify-between">
                  <div class="text-lg font-heading font-black uppercase">Version History</div>
                  <div class="bg-black px-2 py-1 text-[10px] font-heading font-black uppercase text-[#A2FE00]">
                    V.{(versions.data?.find((v: any) => v.is_current)?.version_no || '01').toString().padStart(2, '0')} ACTIVE
                  </div>
                </div>

                <Show
                  when={!versions.isLoading}
                  fallback={
                    <div class="flex items-center justify-center py-10">
                      <Spinner />
                    </div>
                  }
                >
                  <Show
                    when={(versions.data ?? []).length > 0}
                    fallback={<div class="text-xs font-bold text-neutral-darkGray">No versions yet.</div>}
                  >
                    <div class="relative space-y-6 pl-1 before:absolute before:left-4 before:top-2 before:bottom-2 before:w-[3px] before:bg-black">
                      <For each={versions.data ?? []}>
                        {(v: any) => (
                          <div class={['relative pl-12', v.is_current ? '' : 'opacity-80 hover:opacity-100 transition-opacity'].join(' ')}>
                            <div
                              class={[
                                'absolute left-0 top-1 flex h-10 w-10 items-center justify-center border-[3px] border-black',
                                v.is_current ? 'bg-[#A2FE00]' : 'bg-white',
                              ].join(' ')}
                            >
                              <span class="text-xs font-heading font-black">{String(v.version_no).padStart(2, '0')}</span>
                            </div>
                            <div class={['border-[3px] border-black bg-white p-3', v.is_current ? 'shadow-brutal-sm' : ''].join(' ')}>
                              <Show when={v.is_current}>
                                <div class="text-xs font-heading font-black uppercase text-secondary">Current Version</div>
                              </Show>
                              <div class="text-sm font-bold text-black">
                                {v.note || `Version ${v.version_no}`}
                              </div>
                              <div class="mt-2 text-[10px] font-heading font-bold uppercase text-neutral-gray">
                                {formatDateTiny(v.created_at)}
                              </div>
                              <Show when={!v.is_current}>
                                <button
                                  type="button"
                                  class="mt-3 w-full border-[3px] border-black bg-white py-1 text-[10px] font-heading font-black uppercase hover:bg-primary disabled:opacity-60"
                                  disabled={rollback.isPending}
                                  onClick={() => {
                                    if (confirm(`Are you sure you want to rollback to version ${v.version_no}?`)) {
                                      rollback.mutate({ fileId: fileId(), versionId: v.id });
                                    }
                                  }}
                                >
                                  Rollback to this version
                                </button>
                              </Show>
                            </div>
                          </div>
                        )}
                      </For>
                    </div>
                  </Show>
                </Show>
              </div>

              <div class="border-[3px] border-black bg-white p-6 shadow-brutal-sm">
                <div class="mb-4 flex items-center gap-2">
                  <MaterialIcon name="lock_open" />
                  <div class="text-lg font-heading font-black uppercase">Permissions</div>
                </div>

                <div class="space-y-4">
                  {[
                    { title: 'Admin Group', subtitle: 'Full Access Control', role: 'Admin', roleTone: 'bg-black text-white', icon: 'shield', iconBg: 'bg-[#A2FE00]' },
                    { title: 'Engineering Team', subtitle: '12 Members', role: 'Editor', roleTone: 'bg-secondary text-white', icon: 'edit', iconBg: 'bg-white' },
                    { title: 'Project Stakeholders', subtitle: 'Read-only access', role: 'Viewer', roleTone: 'border-2 border-black', icon: 'visibility', iconBg: 'bg-white' },
                  ].map((row, idx) => (
                    <div
                      class={[
                        'flex items-center justify-between gap-3 border-[3px] border-transparent p-2 transition-colors hover:border-black hover:bg-neutral-lightGray/40',
                        idx === 0 ? 'bg-neutral-lightGray/30' : '',
                      ].join(' ')}
                    >
                      <div class="flex items-center gap-3">
                        <div class={['flex h-8 w-8 items-center justify-center border-2 border-black', row.iconBg].join(' ')}>
                          <MaterialIcon name={row.icon} class="text-sm" />
                        </div>
                        <div>
                          <div class="text-xs font-heading font-black uppercase text-black">{row.title}</div>
                          <div class="text-[10px] font-bold text-neutral-darkGray">{row.subtitle}</div>
                        </div>
                      </div>
                      <div class="flex items-center gap-1">
                        <span class={['px-2 py-0.5 text-[10px] font-heading font-black uppercase', row.roleTone].join(' ')}>
                          {row.role}
                        </span>
                        <MaterialIcon name="expand_more" class="text-sm" />
                      </div>
                    </div>
                  ))}
                </div>

                <button
                  type="button"
                  class="mt-6 inline-flex w-full items-center justify-center gap-2 border-[3px] border-black bg-black py-3 font-heading text-xs font-black uppercase text-[#A2FE00] shadow-brutal-sm"
                  onClick={() => alert('Manage access (mock)')}
                >
                  <MaterialIcon name="person_add" />
                  Manage Access
                </button>
              </div>

              <div class="border-[3px] border-black bg-red-50 p-4 shadow-brutal-sm">
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <div class="text-[10px] font-heading font-black uppercase text-red-700">Retention Policy</div>
                    <div class="mt-1 text-xs font-bold text-black">This file is scheduled for archival in 14 days.</div>
                  </div>
                  <MaterialIcon name="warning" class="text-red-700" />
                </div>
                <button
                  type="button"
                  class="mt-3 text-left text-[10px] font-heading font-black uppercase text-red-700 underline hover:no-underline"
                  onClick={() => alert('Update policy (mock)')}
                >
                  Update Policy
                </button>
              </div>
            </div>
          </div>
        </Show>
      </Show>
    </div>
  );
};

export default FileDetail;

