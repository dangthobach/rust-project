import { Component, createSignal, createEffect, Show, For, createMemo, onCleanup } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent, Button, Spinner } from '~/components/ui';
import { useFiles, useUploadFile, useDeleteFile, useDownloadFile, useSearchFiles } from '~/lib/hooks/useFiles';
import { showToast } from '~/lib/toast';
import type { FileMetadata } from '~/lib/api';
import { FileThumbnail } from '~/components/crm';

const Files: Component = () => {
  const [page, setPage] = createSignal(1);
  const [limit] = createSignal(20);
  const [search, setSearch] = createSignal('');
  const [fileTypeFilter, setFileTypeFilter] = createSignal<string>('');
  const [selectedFiles, setSelectedFiles] = createSignal<string[]>([]);
  const [isDragging, setIsDragging] = createSignal(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = createSignal<string | null>(null);

  // Queries and mutations
  const files = useFiles(() => ({ 
    page: page(), 
    limit: limit(),
    file_type: fileTypeFilter() || undefined
  }));
  
  const searchFiles = useSearchFiles(
    () => search(),
    () => ({
      page: page(),
      limit: limit(),
      file_type: fileTypeFilter() || undefined,
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

  // Poll softly for image thumbnails once they are uploaded (worker updates DB asynchronously).
  // Stop polling as soon as all visible image files have thumbnail_path or after max attempts.
  let pollTimer: number | undefined = undefined;
  let pollAttempts = 0;
  const POLL_MS = 2000;
  const MAX_POLL_ATTEMPTS = 10;

  createEffect(() => {
    const list = displayFiles();
    const anyMissingThumb =
      search().length === 0 &&
      list.some((f: FileMetadata) => f.file_type?.startsWith('image/') && !f.thumbnail_path);

    if (anyMissingThumb) {
      if (!pollTimer) {
        pollAttempts = 0;
        pollTimer = window.setInterval(() => {
          pollAttempts++;
          // Query should re-run and refresh thumbnail_path values.
          files.refetch();
          if (pollAttempts >= MAX_POLL_ATTEMPTS) {
            window.clearInterval(pollTimer);
            pollTimer = undefined;
          }
        }, POLL_MS);
      }
    } else {
      if (pollTimer) {
        window.clearInterval(pollTimer);
        pollTimer = undefined;
      }
    }
  });

  onCleanup(() => {
    if (pollTimer) window.clearInterval(pollTimer);
  });

  // File upload handlers
  const handleFileUpload = async (fileList: FileList | null) => {
    if (!fileList || fileList.length === 0) return;

    const file = fileList[0];
    
    // Validate file size (max 50MB)
    const maxSize = 50 * 1024 * 1024;
    if (file.size > maxSize) {
      showToast('error', 'File size must be less than 50MB');
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
    downloadFile.mutate({ id: file.id, filename: file.original_name });
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
    <div>
      {/* Header */}
      <div class="mb-8">
        <div class="flex items-center justify-between flex-wrap gap-4">
          <div>
            <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">
              Files
            </h1>
            <p class="text-neutral-darkGray mt-1">
              Manage your documents and files
            </p>
          </div>
          
          <div class="flex gap-3">
            <Show when={selectedFiles().length > 0}>
              <Button 
                variant="danger" 
                size="md"
                onClick={() => {
                  if (confirm(`Delete ${selectedFiles().length} selected files?`)) {
                    selectedFiles().forEach(id => deleteFile.mutate(id));
                    setSelectedFiles([]);
                  }
                }}
              >
                🗑️ Delete Selected ({selectedFiles().length})
              </Button>
            </Show>
            
            <label class="relative cursor-pointer">
              <input
                type="file"
                class="hidden"
                onChange={handleInputChange}
                disabled={uploadFile.isPending}
              />
              <Button variant="primary" size="md" disabled={uploadFile.isPending}>
                <Show when={uploadFile.isPending} fallback="⬆️ Upload File">
                  <Spinner class="inline-block mr-2" />
                  Uploading...
                </Show>
              </Button>
            </label>
          </div>
        </div>
      </div>

      {/* Search and Filter Bar */}
      <div class="mb-6 flex gap-4 flex-wrap">
        <div class="flex-1 min-w-[300px]">
          <input
            type="text"
            placeholder="🔍 Search files..."
            class="w-full px-4 py-2 border-3 border-black font-bold focus:outline-none focus:ring-2 focus:ring-primary-yellow"
            value={search()}
            onInput={(e) => {
              setSearch(e.currentTarget.value);
              setPage(1);
            }}
          />
        </div>
        
        <select
          class="px-4 py-2 border-3 border-black font-bold bg-white cursor-pointer"
          value={fileTypeFilter()}
          onChange={(e) => {
            setFileTypeFilter(e.currentTarget.value);
            setPage(1);
          }}
        >
          <option value="">All Types</option>
          <option value="image/">Images</option>
          <option value="video/">Videos</option>
          <option value="audio/">Audio</option>
          <option value="pdf">PDF</option>
          <option value="document">Documents</option>
          <option value="spreadsheet">Spreadsheets</option>
        </select>
      </div>

      {/* Drag & Drop Upload Area */}
      <div
        class={`mb-6 border-4 border-dashed rounded-lg p-8 text-center transition-all ${
          isDragging() 
            ? 'border-primary-yellow bg-primary-yellow/10' 
            : 'border-neutral-darkGray bg-neutral-lightGray/30'
        }`}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
      >
        <div class="text-4xl mb-2">📁</div>
        <p class="font-bold text-lg mb-1">Drag & Drop Files Here</p>
        <p class="text-sm text-neutral-darkGray">
          or click the "Upload File" button above
        </p>
        <p class="text-xs text-neutral-darkGray mt-2">
          Maximum file size: 50MB
        </p>
      </div>

      {/* Files List */}
      <Card>
        <CardHeader>
          <div class="flex items-center justify-between">
            <CardTitle>
              File Manager
              <Show when={pagination()}>
                {(p) => (
                  <span class="ml-2 text-sm font-normal text-neutral-darkGray">
                    ({p().total} total)
                  </span>
                )}
              </Show>
            </CardTitle>
            
            <Show when={displayFiles().length > 0}>
              <Button
                variant="ghost"
                size="sm"
                onClick={toggleSelectAll}
              >
                {selectedFiles().length === displayFiles().length ? '☑️ Deselect All' : '☐ Select All'}
              </Button>
            </Show>
          </div>
        </CardHeader>
        <CardContent>
          <Show
            when={!isLoading()}
            fallback={
              <div class="py-12 flex justify-center">
                <Spinner />
              </div>
            }
          >
            <Show
              when={displayFiles().length > 0}
              fallback={
                <div class="text-center py-12">
                  <div class="text-6xl mb-4">📁</div>
                  <p class="text-neutral-darkGray">
                    {search().length > 0 
                      ? `No files found matching "${search()}"`
                      : 'No files uploaded yet. Upload your first file to get started!'
                    }
                  </p>
                </div>
              }
            >
              <div class="divide-y-3 divide-black">
                <For each={displayFiles()}>
                  {(file) => (
                    <div class="p-4 flex items-center justify-between hover:bg-neutral-beige transition-colors group">
                      <div class="flex items-center gap-3 flex-1">
                        {/* Checkbox */}
                        <input
                          type="checkbox"
                          class="w-5 h-5 cursor-pointer"
                          checked={selectedFiles().includes(file.id)}
                          onChange={() => toggleFileSelection(file.id)}
                        />
                        
                        {/* Thumbnail (or processing state) */}
                        <FileThumbnail file={file} class="flex-shrink-0" />
                        
                        {/* File Info */}
                        <div class="flex-1">
                          <h4 class="font-heading font-bold text-sm break-all">
                            {file.original_name}
                          </h4>
                          <p class="text-xs text-neutral-darkGray">
                            {formatFileSize(file.file_size)} • {file.file_type} • {formatDate(file.created_at)}
                          </p>
                        </div>
                      </div>
                      
                      {/* Action Buttons */}
                      <div class="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDownload(file)}
                          disabled={downloadFile.isPending}
                        >
                          ⬇️ Download
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDeleteClick(file.id)}
                          class="text-red-600 hover:bg-red-50"
                        >
                          🗑️ Delete
                        </Button>
                      </div>
                    </div>
                  )}
                </For>
              </div>

              {/* Pagination */}
              <Show when={pagination() && (pagination() as any)!.total_pages > 1}>
                {(p) => (
                  <div class="mt-6 flex items-center justify-center gap-2">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setPage(p => Math.max(1, p - 1))}
                      disabled={!(p() as any).has_prev}
                    >
                      ← Previous
                    </Button>
                    
                    <span class="px-4 py-2 font-bold">
                      Page {(p() as any).page} of {(p() as any).total_pages}
                    </span>
                    
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setPage(p => p + 1)}
                      disabled={!(p() as any).has_next}
                    >
                      Next →
                    </Button>
                  </div>
                )}
              </Show>
            </Show>
          </Show>
        </CardContent>
      </Card>

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
