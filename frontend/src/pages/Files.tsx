import { Component, createSignal, Show, For, createResource } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent, Button, Spinner } from '~/components/ui';
import { api } from '~/lib/api';

interface FileItem {
  id: string;
  name: string;
  path: string;
  size: number;
  uploaded_at: string;
}

const Files: Component = () => {
  const [files, { refetch }] = createResource<FileItem[]>(() => api.getFiles());
  const [uploading, setUploading] = createSignal(false);
  const [uploadError, setUploadError] = createSignal('');

  const handleFileUpload = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    setUploading(true);
    setUploadError('');

    try {
      await api.uploadFile(file);
      await refetch();
      // Reset input
      target.value = '';
    } catch (err: any) {
      setUploadError(err.message || 'Failed to upload file');
    } finally {
      setUploading(false);
    }
  };

  const handleDownload = async (fileId: string, fileName: string) => {
    try {
      const blob = await api.downloadFile(fileId);
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    } catch (err) {
      console.error('Download failed:', err);
    }
  };

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

  return (
    <div>
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
          <label class="relative cursor-pointer">
            <input
              type="file"
              class="hidden"
              onChange={handleFileUpload}
              disabled={uploading()}
            />
            <Button variant="primary" size="md" disabled={uploading()}>
              <Show when={uploading()} fallback="⬆️ Upload File">
                <Spinner class="inline-block mr-2" />
                Uploading...
              </Show>
            </Button>
          </label>
        </div>
      </div>

      <Show when={uploadError()}>
        <div class="mb-4 p-3 bg-red-100 border-3 border-red-500 text-red-700 text-sm font-bold">
          {uploadError()}
        </div>
      </Show>

      <Card>
        <CardHeader>
          <CardTitle>File Manager</CardTitle>
        </CardHeader>
        <CardContent>
          <Show
            when={!files.loading}
            fallback={
              <div class="py-12 flex justify-center">
                <Spinner />
              </div>
            }
          >
            <Show
              when={!files.error}
              fallback={
                <div class="text-center py-12 text-red-600">
                  Failed to load files. Please try again.
                </div>
              }
            >
              <Show
                when={files()?.length ?? 0 > 0}
                fallback={
                  <p class="text-neutral-darkGray text-center py-12">
                    📁 No files uploaded yet. Click "Upload File" to get started.
                  </p>
                }
              >
                <div class="divide-y-3 divide-black">
                  <For each={files()}>
                    {(file) => (
                      <div class="p-4 flex items-center justify-between hover:bg-neutral-beige transition-colors">
                          <div class="flex items-center gap-3 flex-1">
                          <span class="text-2xl">📄</span>
                          <div>
                            <h4 class="font-heading font-bold text-sm">{file.original_name}</h4>
                            <p class="text-xs text-neutral-darkGray">
                              {formatFileSize(file.file_size)} • {formatDate(file.created_at)}
                            </p>
                          </div>
                        </div>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDownload(file.id, file.original_name)}
                        >
                          ⬇️ Download
                        </Button>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </Show>
          </Show>
        </CardContent>
      </Card>
    </div>
  );
};

export default Files;
