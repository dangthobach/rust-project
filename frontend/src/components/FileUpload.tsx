import { Component, createSignal, For, Show, onMount } from 'solid-js';
import { Button, Card, Badge, Spinner } from '~/components/ui';
import { api } from '~/lib/api';

interface FileUploadProps {
  clientId?: string;
  taskId?: string;
  onUploadComplete?: (files: any[]) => void;
  multiple?: boolean;
  acceptedTypes?: string;
  maxSize?: number; // in MB
}

interface UploadFile {
  id: string;
  file: File;
  name: string;
  size: number;
  type: string;
  progress: number;
  status: 'pending' | 'uploading' | 'completed' | 'error';
  preview?: string;
  error?: string;
}

export const FileUpload: Component<FileUploadProps> = (props) => {
  const [isDragOver, setIsDragOver] = createSignal(false);
  const [uploadFiles, setUploadFiles] = createSignal<UploadFile[]>([]);
  const [isUploading, setIsUploading] = createSignal(false);

  let fileInputRef: HTMLInputElement | undefined;

  const maxSize = () => props.maxSize || 10; // 10MB default
  const acceptedTypes = () => props.acceptedTypes || '*/*';

  // Generate file preview for images
  const generatePreview = (file: File): Promise<string | undefined> => {
    return new Promise((resolve) => {
      if (!file.type.startsWith('image/')) {
        resolve(undefined);
        return;
      }

      const reader = new FileReader();
      reader.onload = (e) => resolve(e.target?.result as string);
      reader.onerror = () => resolve(undefined);
      reader.readAsDataURL(file);
    });
  };

  // Validate file
  const validateFile = (file: File): string | null => {
    // Check file size
    const fileSizeMB = file.size / (1024 * 1024);
    if (fileSizeMB > maxSize()) {
      return `File size exceeds ${maxSize()}MB limit`;
    }

    // Check file type if restricted
    if (acceptedTypes() !== '*/*') {
      const types = acceptedTypes().split(',').map(t => t.trim());
      const isValidType = types.some(type => {
        if (type.startsWith('.')) {
          return file.name.toLowerCase().endsWith(type.toLowerCase());
        }
        if (type.includes('/')) {
          return file.type.match(type.replace('*', '.*'));
        }
        return false;
      });

      if (!isValidType) {
        return `File type not allowed. Accepted: ${acceptedTypes()}`;
      }
    }

    return null;
  };

  // Add files to upload queue
  const addFiles = async (files: FileList) => {
    const newFiles: UploadFile[] = [];

    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      const validationError = validateFile(file);
      
      if (validationError) {
        // Show error for invalid files
        const errorFile: UploadFile = {
          id: `error-${Date.now()}-${i}`,
          file,
          name: file.name,
          size: file.size,
          type: file.type,
          progress: 0,
          status: 'error',
          error: validationError,
        };
        newFiles.push(errorFile);
        continue;
      }

      const preview = await generatePreview(file);
      
      const uploadFile: UploadFile = {
        id: `file-${Date.now()}-${i}`,
        file,
        name: file.name,
        size: file.size,
        type: file.type,
        progress: 0,
        status: 'pending',
        preview,
      };
      
      newFiles.push(uploadFile);
    }

    if (!props.multiple && newFiles.length > 1) {
      // If not multiple, only take the first valid file
      const validFile = newFiles.find(f => f.status === 'pending');
      setUploadFiles(validFile ? [validFile] : []);
    } else {
      setUploadFiles(prev => [...prev, ...newFiles]);
    }
  };

  // Handle drag and drop
  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    setIsDragOver(true);
  };

  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    if (e.currentTarget === e.target) {
      setIsDragOver(false);
    }
  };

  const handleDrop = async (e: DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    
    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      await addFiles(files);
    }
  };

  // Handle file input change
  const handleFileSelect = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    const files = target.files;
    
    if (files && files.length > 0) {
      await addFiles(files);
    }
    
    // Reset input
    target.value = '';
  };

  // Remove file from upload queue
  const removeFile = (fileId: string) => {
    setUploadFiles(prev => prev.filter(f => f.id !== fileId));
  };

  // Upload a single file
  const uploadSingleFile = async (uploadFile: UploadFile): Promise<any> => {
    setUploadFiles(prev => prev.map(f => 
      f.id === uploadFile.id 
        ? { ...f, status: 'uploading' as const, progress: 0 }
        : f
    ));

    try {
      const formData = new FormData();
      formData.append('file', uploadFile.file);
      
      if (props.clientId) {
        formData.append('client_id', props.clientId);
      }
      if (props.taskId) {
        formData.append('task_id', props.taskId);
      }

      // Simulate upload progress
      const progressInterval = setInterval(() => {
        setUploadFiles(prev => prev.map(f => 
          f.id === uploadFile.id && f.progress < 90
            ? { ...f, progress: f.progress + 10 }
            : f
        ));
      }, 100);

      // Upload file
      const response = await api.uploadFile(formData);

      clearInterval(progressInterval);

      setUploadFiles(prev => prev.map(f => 
        f.id === uploadFile.id 
          ? { ...f, status: 'completed' as const, progress: 100 }
          : f
      ));

      return response;
    } catch (error: any) {
      setUploadFiles(prev => prev.map(f => 
        f.id === uploadFile.id 
          ? { 
              ...f, 
              status: 'error' as const, 
              error: error.message || 'Upload failed',
              progress: 0 
            }
          : f
      ));
      throw error;
    }
  };

  // Upload all pending files
  const uploadAll = async () => {
    const pendingFiles = uploadFiles().filter(f => f.status === 'pending');
    
    if (pendingFiles.length === 0) return;

    setIsUploading(true);

    try {
      const uploadPromises = pendingFiles.map(file => uploadSingleFile(file));
      const results = await Promise.allSettled(uploadPromises);
      
      const successfulUploads = results
        .filter((result, index) => result.status === 'fulfilled')
        .map((result, index) => (result as PromiseFulfilledResult<any>).value);

      if (successfulUploads.length > 0 && props.onUploadComplete) {
        props.onUploadComplete(successfulUploads);
      }
    } catch (error) {
      console.error('Upload error:', error);
    } finally {
      setIsUploading(false);
    }
  };

  // Retry failed upload
  const retryUpload = async (fileId: string) => {
    const file = uploadFiles().find(f => f.id === fileId);
    if (file && file.status === 'error') {
      await uploadSingleFile(file);
    }
  };

  // Clear completed/error files
  const clearCompleted = () => {
    setUploadFiles(prev => prev.filter(f => 
      f.status !== 'completed' && f.status !== 'error'
    ));
  };

  // Format file size
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  // Get file type icon
  const getFileIcon = (type: string): string => {
    if (type.startsWith('image/')) return '🖼️';
    if (type.startsWith('video/')) return '🎥';
    if (type.startsWith('audio/')) return '🎵';
    if (type.includes('pdf')) return '📄';
    if (type.includes('word')) return '📝';
    if (type.includes('excel') || type.includes('spreadsheet')) return '📊';
    if (type.includes('powerpoint') || type.includes('presentation')) return '📋';
    if (type.includes('zip') || type.includes('rar')) return '🗜️';
    return '📎';
  };

  return (
    <div class="space-y-4">
      {/* Upload Area */}
      <div
        class={`relative border-4 border-dashed rounded-lg p-8 text-center transition-all ${
          isDragOver()
            ? 'border-primary bg-blue-50'
            : 'border-neutral-darkGray bg-white hover:bg-neutral-beige'
        }`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <input
          ref={fileInputRef}
          type="file"
          multiple={props.multiple}
          accept={acceptedTypes()}
          onChange={handleFileSelect}
          class="hidden"
        />

        <div class="space-y-4">
          <div class="text-6xl">📁</div>
          
          <div>
            <h3 class="text-xl font-heading font-bold uppercase mb-2">
              Upload Files
            </h3>
            <p class="text-neutral-darkGray">
              Drag and drop files here, or click to browse
            </p>
            <p class="text-sm text-neutral-darkGray mt-2">
              Max size: {maxSize()}MB
              {acceptedTypes() !== '*/*' && (
                <span> • Accepted: {acceptedTypes()}</span>
              )}
            </p>
          </div>

          <Button
            variant="primary"
            size="lg"
            onClick={() => fileInputRef?.click()}
          >
            📎 Choose Files
          </Button>
        </div>

        <Show when={isDragOver()}>
          <div class="absolute inset-0 bg-primary bg-opacity-10 border-4 border-primary rounded-lg flex items-center justify-center">
            <div class="text-primary font-heading font-bold text-xl uppercase">
              Drop files here
            </div>
          </div>
        </Show>
      </div>

      {/* File List */}
      <Show when={uploadFiles().length > 0}>
        <Card>
          <div class="p-4 border-b-3 border-black">
            <div class="flex items-center justify-between">
              <h3 class="font-heading font-bold uppercase">
                Files ({uploadFiles().length})
              </h3>
              <div class="flex gap-2">
                <Show when={uploadFiles().some(f => f.status === 'pending')}>
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={uploadAll}
                    disabled={isUploading()}
                  >
                    <Show when={isUploading()} fallback="🚀 Upload All">
                      <Spinner class="inline-block mr-2" />
                      Uploading...
                    </Show>
                  </Button>
                </Show>
                
                <Show when={uploadFiles().some(f => f.status === 'completed' || f.status === 'error')}>
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={clearCompleted}
                  >
                    🗑️ Clear
                  </Button>
                </Show>
              </div>
            </div>
          </div>

          <div class="divide-y-3 divide-black">
            <For each={uploadFiles()}>
              {(file) => (
                <div class="p-4">
                  <div class="flex items-start gap-4">
                    {/* File Preview/Icon */}
                    <div class="flex-shrink-0">
                      <Show
                        when={file.preview}
                        fallback={
                          <div class="w-16 h-16 bg-neutral-beige border-3 border-black flex items-center justify-center text-2xl">
                            {getFileIcon(file.type)}
                          </div>
                        }
                      >
                        <img
                          src={file.preview}
                          alt="Preview"
                          class="w-16 h-16 object-cover border-3 border-black"
                        />
                      </Show>
                    </div>

                    {/* File Info */}
                    <div class="flex-1 min-w-0">
                      <div class="flex items-start justify-between">
                        <div class="flex-1 min-w-0">
                          <p class="font-bold truncate" title={file.name}>
                            {file.name}
                          </p>
                          <p class="text-sm text-neutral-darkGray">
                            {formatFileSize(file.size)} • {file.type}
                          </p>
                        </div>

                        <div class="flex items-center gap-2 ml-4">
                          <Badge
                            variant={
                              file.status === 'completed' ? 'success' :
                              file.status === 'error' ? 'destructive' :
                              file.status === 'uploading' ? 'warning' : 'secondary'
                            }
                          >
                            {file.status === 'completed' && '✅ Completed'}
                            {file.status === 'error' && '❌ Error'}
                            {file.status === 'uploading' && '⏳ Uploading'}
                            {file.status === 'pending' && '⏸️ Pending'}
                          </Badge>

                          <Show when={file.status === 'error'}>
                            <Button
                              variant="secondary"
                              size="xs"
                              onClick={() => retryUpload(file.id)}
                            >
                              🔄 Retry
                            </Button>
                          </Show>

                          <Button
                            variant="destructive"
                            size="xs"
                            onClick={() => removeFile(file.id)}
                            disabled={file.status === 'uploading'}
                          >
                            🗑️
                          </Button>
                        </div>
                      </div>

                      {/* Progress Bar */}
                      <Show when={file.status === 'uploading'}>
                        <div class="w-full bg-neutral-concrete h-2 border-2 border-black mt-2 overflow-hidden">
                          <div
                            class="h-full bg-blue-500 transition-all duration-300"
                            style={{ width: `${file.progress}%` }}
                          />
                        </div>
                        <p class="text-xs text-neutral-darkGray mt-1">
                          {file.progress}%
                        </p>
                      </Show>

                      {/* Error Message */}
                      <Show when={file.error}>
                        <p class="text-red-600 text-sm mt-2 font-bold">
                          {file.error}
                        </p>
                      </Show>
                    </div>
                  </div>
                </div>
              )}
            </For>
          </div>
        </Card>
      </Show>
    </div>
  );
};

export default FileUpload;