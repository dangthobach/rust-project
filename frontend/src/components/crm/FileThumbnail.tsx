import { Component, createMemo, Show } from 'solid-js';
import { Spinner, Badge } from '~/components/ui';
import { useThumbnailUrl } from '~/lib/hooks/useFiles';

import type { File as FileMetadata } from '~/lib/api';

interface FileThumbnailProps {
  file: FileMetadata;
  class?: string;
}

const FileThumbnail: Component<FileThumbnailProps> = (props) => {
  const isImage = createMemo(() => props.file.file_type?.startsWith('image/') ?? false);

  // Only fetch when thumbnail exists and is image.
  const thumb = useThumbnailUrl(
    () => props.file.id,
    () => isImage() && !!props.file.thumbnail_path,
  );

  return (
    <div class={props.class ?? ''}>
      <Show when={isImage()} fallback={<span class="text-3xl">📄</span>}>
        <Show when={props.file.thumbnail_path} fallback={
          <div class="w-16 h-16 border-3 border-black bg-neutral-lightGray flex items-center justify-center">
            <Spinner class="w-5 h-5" />
          </div>
        }>
          <Show when={thumb.data} fallback={
            <div class="w-16 h-16 border-3 border-black bg-neutral-lightGray flex items-center justify-center">
              <Spinner class="w-5 h-5" />
            </div>
          }>
            <img
              src={thumb.data!.download_url}
              alt={props.file.original_name}
              class="w-16 h-16 object-cover border-3 border-black"
            />
          </Show>
        </Show>
        <Show when={isImage() && !props.file.thumbnail_path}>
          <Badge variant="warning" class="mt-1 border-2 border-black">
            processing
          </Badge>
        </Show>
      </Show>
    </div>
  );
};

export default FileThumbnail;

