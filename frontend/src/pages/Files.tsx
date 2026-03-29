import { Component, createSignal, createEffect, Show, For, createMemo, onCleanup } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { Button, Spinner } from '~/components/ui';
import { useFiles, useUploadFile, useDeleteFile, useDownloadFile, useSearchFiles } from '~/lib/hooks/useFiles';
import { showToast } from '~/lib/toast';
import type { FileMetadata } from '~/lib/api';

const MaterialIcon: Component<{ name: string; class?: string }> = (props) => (
  <span class={['material-symbols-outlined', props.class ?? ''].join(' ')} aria-hidden="true">
    {props.name}
  </span>
);

const Files: Component = () => {
  const navigate = useNavigate();
  const [page, setPage] = createSignal(1);
  const [limit] = createSignal(20);
  const [search, setSearch] = createSignal('');
  const [selectedFiles, setSelectedFiles] = createSignal<string[]>([]);
  const [isDragging, setIsDragging] = createSignal(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = createSignal<string | null>(null);
  const [activeTab, setActiveTab] = createSignal<'shared' | 'recent' | 'starred'>('recent');
  const [viewMode, setViewMode] = createSignal<'grid' | 'list'>('grid');
  const [sortBy, setSortBy] = createSignal<'modified' | 'name' | 'size'>('modified');

  // Queries and mutations
  const files = useFiles(() => ({ 
    page: page(), 
    limit: limit(),
    tab: activeTab(),
  }));
  
  const searchFiles = useSearchFiles(
    () => search(),
    () => ({
      page: page(),
      limit: limit(),
      tab: activeTab(),
    })
  );
  
  const uploadFile = useUploadFile();
  const deleteFile = useDeleteFile();
  const downloadFile = useDownloadFile();

  // Use search results if search query exists, otherwise use regular files
  const displayFiles = createMemo(() => {
    if (search().length > 0) {
      return searchFiles.data?.data || [];
    }
    return files.data?.data || [];
  });

  const pagination = createMemo(() => {
    if (search().length > 0) {
      return searchFiles.data?.pagination;
    }
    return files.data?.pagination;
  });

  const isLoading = createMemo(() => {
    if (search().length > 0) {
      return searchFiles.isLoading;
    }
    return files.isLoading;
  });

  // Note: `/api/fs/*` does not expose thumbnail paths yet; avoid thumbnail polling.
  onCleanup(() => undefined);

  // File upload handlers
  const handleFileUpload = async (fileList: FileList | null) => {
    if (!fileList || fileList.length === 0) return;

    const file = fileList[0];
    
    // Validate file size (max 500MB — match DMS mock)
    const maxSize = 500 * 1024 * 1024;
    if (file.size > maxSize) {
      showToast('error', 'File size must be less than 500MB');
      return;
    }

    uploadFile.mutate({ file });
  };

  const handleInputChange = (e: Event) => {
    const target = e.target as HTMLInputElement;
    handleFileUpload(target.files);
    target.value = ''; // Reset input
  };

  // Drag and drop handlers
  const handleDragEnter = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    handleFileUpload(e.dataTransfer?.files || null);
  };

  // Download handler
  const handleDownload = (file: FileMetadata) => {
    downloadFile.mutate({ id: file.id, filename: file.name });
  };

  // Delete handlers
  const handleDeleteClick = (id: string) => {
    setShowDeleteConfirm(id);
  };

  const handleDeleteConfirm = () => {
    const id = showDeleteConfirm();
    if (id) {
      deleteFile.mutate(id);
      setShowDeleteConfirm(null);
    }
  };

  const handleDeleteCancel = () => {
    setShowDeleteConfirm(null);
  };

  // Selection handlers
  const toggleFileSelection = (id: string) => {
    setSelectedFiles(prev => 
      prev.includes(id) 
        ? prev.filter(fid => fid !== id)
        : [...prev, id]
    );
  };

  const toggleSelectAll = () => {
    const allIds = displayFiles().map(f => f.id);
    if (selectedFiles().length === allIds.length) {
      setSelectedFiles([]);
    } else {
      setSelectedFiles(allIds);
    }
  };

  // Utility functions
  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getFileIcon = (fileType: string) => {
    if (fileType.startsWith('image/')) return '🖼️';
    if (fileType.startsWith('video/')) return '🎥';
    if (fileType.startsWith('audio/')) return '🎵';
    if (fileType.includes('pdf')) return '📕';
    if (fileType.includes('word') || fileType.includes('document')) return '📘';
    if (fileType.includes('excel') || fileType.includes('spreadsheet')) return '📗';
    if (fileType.includes('powerpoint') || fileType.includes('presentation')) return '📙';
    if (fileType.includes('zip') || fileType.includes('compressed')) return '📦';
    return '📄';
  };

  return (
    <div class="space-y-8">
      {/* DMS top strip (page-level) */}
      <div class="flex flex-col gap-4 border-[3px] border-black bg-background p-4 md:flex-row md:items-center md:justify-between">
        <div class="flex flex-wrap items-center gap-3">
          <button
            type="button"
            class={[
              'px-2 py-1 font-heading text-xs font-black uppercase tracking-wider transition-colors',
              activeTab() === 'shared' ? 'bg-[#A2FE00] text-black' : 'hover:bg-[#A2FE00]',
            ].join(' ')}
            onClick={() => setActiveTab('shared')}
          >
            Shared
          </button>
          <button
            type="button"
            class={[
              'px-2 py-1 font-heading text-xs font-black uppercase tracking-wider transition-colors',
              activeTab() === 'recent' ? 'bg-[#A2FE00] text-black' : 'hover:bg-[#A2FE00]',
            ].join(' ')}
            onClick={() => setActiveTab('recent')}
          >
            Recent
          </button>
          <button
            type="button"
            class={[
              'px-2 py-1 font-heading text-xs font-black uppercase tracking-wider transition-colors',
              activeTab() === 'starred' ? 'bg-[#A2FE00] text-black' : 'hover:bg-[#A2FE00]',
            ].join(' ')}
            onClick={() => setActiveTab('starred')}
          >
            Starred
          </button>
        </div>

        <div class="flex flex-col gap-3 sm:flex-row sm:items-center">
          <div class="relative min-w-[260px]">
            <input
              type="search"
              placeholder="SEARCH_FILES..."
              value={search()}
              onInput={(e) => {
                setSearch(e.currentTarget.value);
                setPage(1);
              }}
              class="w-full border-[3px] border-black bg-white px-4 py-2 pr-10 font-heading text-xs font-black uppercase tracking-wider focus:outline-none focus:shadow-brutal-sm"
            />
            <span class="absolute right-3 top-2.5 text-black" aria-hidden="true">
              <MaterialIcon name="search" />
            </span>
          </div>

          <label class="relative cursor-pointer">
            <input type="file" class="hidden" onChange={handleInputChange} disabled={uploadFile.isPending} />
            <button
              type="button"
              class="inline-flex items-center justify-center border-[3px] border-black bg-[#A2FE00] px-4 py-2 font-heading text-xs font-black uppercase shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5 disabled:opacity-60"
              disabled={uploadFile.isPending}
              onClick={(e) => (e.currentTarget.previousElementSibling as HTMLInputElement | null)?.click()}
            >
              <Show when={uploadFile.isPending} fallback="Upload File">
                <span class="mr-2 inline-flex items-center">
                  <Spinner class="h-4 w-4" />
                </span>
                Uploading...
              </Show>
            </button>
          </label>
        </div>
      </div>

      {/* Header + stats */}
      <div class="flex flex-col gap-6 md:flex-row md:items-end md:justify-between">
        <div>
          <h1 class="text-display-2 font-heading font-black uppercase tracking-tight text-black">Documents</h1>
          <div class="mt-2 flex flex-wrap gap-3">
            <span class="bg-black px-3 py-1 text-xs font-heading font-black uppercase text-white">
              Root / Projects / 2026_ledger
            </span>
            <span class="self-center text-xs font-heading font-black uppercase text-neutral-gray">
              Updated {new Date().toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })}
            </span>
          </div>
        </div>
        <div class="flex gap-4">
          <div class="min-w-[140px] border-[3px] border-black bg-white p-4 shadow-brutal-sm">
            <div class="mb-1 text-[10px] font-heading font-black uppercase text-neutral-gray">Total Files</div>
            <div class="text-3xl font-heading font-black">{pagination()?.total ?? displayFiles().length}</div>
          </div>
          <div class="min-w-[140px] border-[3px] border-black bg-[#A2FE00] p-4 shadow-brutal-sm">
            <div class="mb-1 text-[10px] font-heading font-black uppercase text-black">New This Week</div>
            <div class="text-3xl font-heading font-black text-black">+{Math.min(42, displayFiles().length)}</div>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-12 gap-8">
        {/* Browser */}
        <div class="col-span-12 xl:col-span-9">
          {/* Toolbar */}
          <div class="mb-6 flex flex-wrap items-center justify-between gap-4 border-[3px] border-black bg-white p-4">
            <div class="flex items-center gap-2">
              <button
                type="button"
                class={[
                  'border-[2px] border-black p-2',
                  viewMode() === 'grid' ? 'bg-black text-white' : 'bg-white hover:bg-neutral-lightGray/60',
                ].join(' ')}
                onClick={() => setViewMode('grid')}
                aria-label="Grid view"
              >
                <MaterialIcon name="grid_view" />
              </button>
              <button
                type="button"
                class={[
                  'border-[2px] border-black p-2',
                  viewMode() === 'list' ? 'bg-black text-white' : 'bg-white hover:bg-neutral-lightGray/60',
                ].join(' ')}
                onClick={() => setViewMode('list')}
                aria-label="List view"
              >
                <MaterialIcon name="list" />
              </button>
              <div class="mx-2 h-8 w-px bg-black/20" />
              <div class="flex items-center gap-3">
                <span class="text-[10px] font-heading font-black uppercase text-neutral-gray">Sort By:</span>
                <select
                  class="cursor-pointer border-0 bg-transparent text-xs font-heading font-black uppercase focus:ring-0"
                  value={sortBy()}
                  onChange={(e) => setSortBy(e.currentTarget.value as any)}
                >
                  <option value="modified">Date Modified</option>
                  <option value="name">File Name</option>
                  <option value="size">File Size</option>
                </select>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <button
                type="button"
                class="inline-flex items-center gap-2 border-[3px] border-black bg-white px-4 py-2 text-xs font-heading font-black uppercase transition-all hover:bg-black hover:text-white"
                onClick={() => showToast('info', 'Filter', 'Advanced filters are UI-only in this iteration.')}
              >
                <MaterialIcon name="filter_list" class="text-sm" />
                Filter
              </button>
              <button
                type="button"
                class="inline-flex items-center gap-2 border-[3px] border-black bg-white px-4 py-2 text-xs font-heading font-black uppercase transition-all hover:bg-black hover:text-white"
                onClick={() => showToast('info', 'Bulk Export', 'Bulk export will be wired after selecting target formats.')}
              >
                <MaterialIcon name="file_download" class="text-sm" />
                Bulk Export
              </button>
            </div>
          </div>

          {/* Grid */}
          <Show
            when={!isLoading()}
            fallback={
              <div class="flex items-center justify-center border-[3px] border-black bg-white p-10 shadow-brutal-sm">
                <Spinner />
              </div>
            }
          >
            <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
              {/* Mock folder card (until backend folders are integrated) */}
              <div
                class="group relative cursor-pointer border-[3px] border-black bg-white p-5 shadow-brutal-sm transition-all hover:translate-x-1 hover:translate-y-1 hover:shadow-none"
                onClick={() => showToast('info', 'Folder', 'Folder navigation will be enabled once /api/fs/folders is wired in UI.')}
              >
                <div class="mb-4 flex items-start justify-between">
                  <MaterialIcon name="folder" class="text-5xl" />
                  <button type="button" class="opacity-0 transition-opacity group-hover:opacity-100 hover:bg-[#A2FE00] p-1">
                    <MaterialIcon name="more_vert" />
                  </button>
                </div>
                <div class="mb-1 truncate text-lg font-heading font-black uppercase">Marketing_Assets_2026</div>
                <div class="flex items-center justify-between text-[10px] font-heading font-black uppercase text-neutral-gray">
                  <span>24 Files</span>
                  <span>1.2 GB</span>
                </div>
                <div class="mt-4 flex -space-x-2">
                  <div class="h-6 w-6 overflow-hidden border-[2px] border-black bg-white" />
                  <div class="h-6 w-6 overflow-hidden border-[2px] border-black bg-white" />
                  <div class="flex h-6 w-6 items-center justify-center border-[2px] border-black bg-black text-[8px] font-black text-[#A2FE00]">
                    +5
                  </div>
                </div>
              </div>

              <For each={displayFiles()}>
                {(file) => {
                  const mime = () => file.mime_type ?? '';
                  const name = () => file.name ?? '';
                  const isImage = () => mime().startsWith('image/');
                  const isPdf = () => mime().includes('pdf') || name().toLowerCase().endsWith('.pdf');
                  const isSheet = () => mime().includes('spreadsheet') || mime().includes('excel') || /\.(xlsx|xls|csv)$/i.test(name());
                  const isCode = () => mime().includes('typescript') || /\.(ts|tsx)$/i.test(name());

                  const openDetails = () => navigate(`/files/${file.id}`);

                  return (
                    <div
                      class={[
                        'group relative cursor-pointer border-[3px] border-black bg-white shadow-brutal-sm transition-all hover:translate-x-1 hover:translate-y-1 hover:shadow-none',
                        isImage() ? 'overflow-hidden p-0' : 'p-5',
                      ].join(' ')}
                      onClick={openDetails}
                      role="button"
                      tabindex={0}
                      onKeyDown={(e) => e.key === 'Enter' && openDetails()}
                    >
                      <Show
                        when={isImage()}
                        fallback={
                          <>
                            <div class="mb-4 flex items-start justify-between">
                              <span class={['text-5xl', isPdf() ? 'text-red-600' : isSheet() ? 'text-primary' : 'text-black'].join(' ')}>
                                <MaterialIcon name={isPdf() ? 'picture_as_pdf' : isSheet() ? 'table_chart' : isCode() ? 'code' : 'description'} />
                              </span>
                              <Show when={isPdf()}>
                                <div class="bg-red-600 px-2 py-0.5 text-[8px] font-heading font-black uppercase text-white">LOCKED</div>
                              </Show>
                              <Show when={!isPdf() && isSheet()}>
                                <div class="bg-[#A2FE00] px-2 py-0.5 text-[8px] font-heading font-black uppercase text-black">SHARED</div>
                              </Show>
                            </div>
                            <div class="mb-1 truncate text-lg font-heading font-black uppercase">{file.name}</div>
                            <div class="flex items-center justify-between text-[10px] font-heading font-black uppercase text-neutral-gray">
                              <span>{formatFileSize(file.size)}</span>
                              <span>{formatDate(file.updated_at ?? file.created_at)}</span>
                            </div>
                            <Show when={!isCode()}>
                              <div class="mt-4 border-t border-black/10 pt-4 flex items-center justify-between">
                                <span class="text-[10px] font-heading font-black uppercase text-secondary">
                                  Viewed by Admin
                                </span>
                                <MaterialIcon name="history" class="text-sm" />
                              </div>
                            </Show>
                            <Show when={isCode()}>
                              <div class="mt-4 flex gap-1">
                                <span class="bg-neutral-lightGray px-2 py-0.5 text-[8px] font-heading font-black">TYPESCRIPT</span>
                                <span class="bg-neutral-lightGray px-2 py-0.5 text-[8px] font-heading font-black">UI_REF</span>
                              </div>
                            </Show>
                          </>
                        }
                      >
                        <div class="h-32 overflow-hidden bg-neutral-lightGray flex items-center justify-center">
                          <MaterialIcon name="image" class="text-5xl text-black/60" />
                        </div>
                        <div class="p-4">
                          <div class="mb-1 truncate text-lg font-heading font-black uppercase">{file.name}</div>
                          <div class="flex items-center justify-between text-[10px] font-heading font-black uppercase text-neutral-gray">
                            <span>{formatFileSize(file.size)}</span>
                            <span>{formatDate(file.updated_at ?? file.created_at)}</span>
                          </div>
                        </div>
                        <div class="absolute right-2 top-2 flex gap-1">
                          <button
                            type="button"
                            class="border-[2px] border-black bg-white p-1 hover:bg-[#A2FE00]"
                            onClick={(e) => {
                              e.stopPropagation();
                              showToast('info', 'Share', 'Share UI is a stub for now.');
                            }}
                            aria-label="Share"
                          >
                            <MaterialIcon name="share" class="text-xs" />
                          </button>
                          <button
                            type="button"
                            class="border-[2px] border-black bg-white p-1 hover:bg-[#A2FE00]"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleDownload(file);
                            }}
                            aria-label="Download"
                          >
                            <MaterialIcon name="download" class="text-xs" />
                          </button>
                        </div>
                      </Show>

                      <div class="absolute right-3 top-3 flex gap-2 opacity-0 transition-opacity group-hover:opacity-100">
                        <button
                          type="button"
                          class="border-[2px] border-black bg-white px-2 py-1 text-[10px] font-heading font-black uppercase hover:bg-[#A2FE00]"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleDownload(file);
                          }}
                        >
                          Download
                        </button>
                        <button
                          type="button"
                          class="border-[2px] border-black bg-white px-2 py-1 text-[10px] font-heading font-black uppercase hover:bg-red-50 text-red-700"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleDeleteClick(file.id);
                          }}
                        >
                          Delete
                        </button>
                      </div>
                    </div>
                  );
                }}
              </For>

              {/* Dropzone */}
              <div
                class={[
                  'group flex cursor-pointer flex-col items-center justify-center border-[3px] border-dashed border-black bg-white/60 p-8 text-center transition-colors hover:bg-[#A2FE00]/10',
                  isDragging() ? 'bg-[#A2FE00]/20' : '',
                ].join(' ')}
                onDragEnter={handleDragEnter}
                onDragLeave={handleDragLeave}
                onDragOver={handleDragOver}
                onDrop={handleDrop}
                onClick={() => showToast('info', 'Upload', 'Use the Upload File button to pick a file.')}
              >
                <div class="mb-4 transition-transform group-hover:scale-110">
                  <MaterialIcon name="cloud_upload" class="text-4xl" />
                </div>
                <div class="text-sm font-heading font-black uppercase">Drop New Files Here</div>
                <div class="mt-2 text-[10px] font-heading font-bold uppercase text-neutral-gray">Max 500MB per file</div>
              </div>
            </div>

            <Show when={displayFiles().length === 0 && !isLoading()}>
              <div class="mt-6 border-[3px] border-black bg-white p-8 shadow-brutal-sm">
                <div class="text-sm font-bold text-neutral-darkGray">
                  {search().length > 0 ? `No files found matching "${search()}"` : 'No files uploaded yet.'}
                </div>
              </div>
            </Show>

            <Show when={pagination() && (pagination() as any)!.total_pages > 1}>
              {(p) => (
                <div class="mt-6 flex items-center justify-center gap-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setPage((pg) => Math.max(1, pg - 1))}
                    disabled={!(p() as any).has_prev}
                  >
                    ← Previous
                  </Button>
                  <span class="px-4 py-2 font-heading font-black uppercase text-xs">
                    Page {(p() as any).page} of {(p() as any).total_pages}
                  </span>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setPage((pg) => pg + 1)}
                    disabled={!(p() as any).has_next}
                  >
                    Next →
                  </Button>
                </div>
              )}
            </Show>
          </Show>
        </div>

        {/* Sidebar */}
        <div class="col-span-12 xl:col-span-3 space-y-8">
          <div class="border-[3px] border-black bg-black p-6 shadow-brutal-sm text-white">
            <div class="mb-4 text-xl font-heading font-black uppercase leading-tight">Quick Action Ledger</div>
            <div class="space-y-3">
              <button
                type="button"
                class="w-full border-[3px] border-[#A2FE00] bg-[#A2FE00] py-3 font-heading text-xs font-black uppercase text-black transition-all hover:bg-black hover:text-[#A2FE00]"
                onClick={() => showToast('info', 'Request Signature', 'This action is UI-only for now.')}
              >
                Request Signature
              </button>
              <button
                type="button"
                class="w-full border-[3px] border-white py-3 font-heading text-xs font-black uppercase transition-all hover:bg-white hover:text-black"
                onClick={() => showToast('info', 'Secure Transfer', 'This action is UI-only for now.')}
              >
                Secure Transfer
              </button>
            </div>
            <div class="mt-6 border-t border-white/20 pt-6">
              <div class="mb-2 text-[10px] font-heading font-black uppercase text-white/70">Encryption Status</div>
              <div class="flex items-center gap-2">
                <span class="h-2 w-2 bg-[#A2FE00]" />
                <span class="text-xs font-heading font-black uppercase">AES-256 ACTIVE</span>
              </div>
            </div>
          </div>

          <div class="border-[3px] border-black bg-white p-6 shadow-brutal-sm">
            <div class="mb-6 flex items-center justify-between">
              <div class="text-xl font-heading font-black uppercase">Activity</div>
              <MaterialIcon name="history" class="text-secondary" />
            </div>
            <div class="space-y-6">
              {[
                { icon: 'edit', iconBg: 'bg-black text-[#A2FE00]', title: "Mark S. edited 'Contract_V4'", meta: '14:22 — Project Alpha' },
                { icon: 'share', iconBg: 'bg-secondary text-white', title: 'External Share Created', meta: '11:05 — Marketing_Assets' },
                { icon: 'upload_file', iconBg: 'bg-[#A2FE00] text-black', title: '54 Files Uploaded', meta: 'Yesterday — Bulk Import' },
              ].map((row, idx) => (
                <div class="relative flex gap-4">
                  <Show when={idx !== 2}>
                    <div class="absolute left-3 top-6 h-full w-px bg-black/10" />
                  </Show>
                  <div class={['z-10 flex h-6 w-6 items-center justify-center', row.iconBg].join(' ')}>
                    <MaterialIcon name={row.icon} class="text-[14px]" />
                  </div>
                  <div>
                    <div class="text-xs font-heading font-black uppercase text-black">{row.title}</div>
                    <div class="mt-1 text-[10px] font-heading font-black uppercase text-neutral-gray">{row.meta}</div>
                  </div>
                </div>
              ))}
            </div>
            <button
              type="button"
              class="mt-8 w-full border-[3px] border-black py-2 text-[10px] font-heading font-black uppercase transition-all hover:bg-black hover:text-white"
              onClick={() => showToast('info', 'Logs', 'System logs view is not wired yet.')}
            >
              View System Logs
            </button>
          </div>

          <div class="border-[3px] border-black bg-white p-6">
            <div class="mb-4 text-xl font-heading font-black uppercase">Starred Nodes</div>
            <div class="space-y-2">
              {['Brand_Kit_2026', 'Employee_Onboarding'].map((name) => (
                <div
                  class="group flex cursor-pointer items-center justify-between border-[2px] border-black bg-white p-2 transition-colors hover:bg-[#A2FE00]"
                  onClick={() => showToast('info', 'Starred', `${name} (mock)`)}
                >
                  <div class="flex items-center gap-3">
                    <MaterialIcon name="star" class="text-black" />
                    <div class="text-xs font-heading font-black uppercase">{name}</div>
                  </div>
                  <div class="opacity-0 transition-opacity group-hover:opacity-100">
                    <MaterialIcon name="open_in_new" class="text-sm" />
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Delete Confirmation Modal */}
      <Show when={showDeleteConfirm()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div class="bg-white border-4 border-black p-6 max-w-md w-full mx-4">
            <h3 class="text-xl font-heading font-black mb-4">Delete File?</h3>
            <p class="text-neutral-darkGray mb-6">
              Are you sure you want to delete this file? This action cannot be undone.
            </p>
            <div class="flex gap-3 justify-end">
              <Button
                variant="ghost"
                size="md"
                onClick={handleDeleteCancel}
              >
                Cancel
              </Button>
              <Button
                variant="danger"
                size="md"
                onClick={handleDeleteConfirm}
              >
                Delete
              </Button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default Files;
