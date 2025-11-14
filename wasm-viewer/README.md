# WASM File Viewer

A high-performance file viewer built with Rust and compiled to WebAssembly.

## Supported File Types

- **Text**: .txt, .md
- **Images**: .png, .jpg, .jpeg, .gif, .webp
- **Documents**: .pdf, .csv
- **Code**: Various programming languages (with syntax highlighting)

## Build

```bash
# Install wasm-pack if you haven't
cargo install wasm-pack

# Build for web
wasm-pack build --target web

# Build for production
wasm-pack build --target web --release
```

## Usage in JavaScript/Qwik

```javascript
import init, { FileViewer, detect_file_type } from './wasm-viewer/pkg';

// Initialize WASM
await init();

// Detect file type
const fileType = detect_file_type('document.pdf');

// Create viewer
const viewer = new FileViewer(fileType);

// Load file content
const fileData = await fetch('/api/files/123').then(r => r.arrayBuffer());
viewer.load_content(new Uint8Array(fileData));

// Render
const html = viewer.render();
document.getElementById('viewer').innerHTML = html;

// Get file info
const info = JSON.parse(viewer.get_file_info());
console.log(info);
```

## Integration with Qwik

```tsx
import { component$, useSignal, useVisibleTask$ } from '@builder.io/qwik';

export const FileViewerComponent = component$(() => {
  const viewerHtml = useSignal('');

  useVisibleTask$(async () => {
    const { default: init, FileViewer } = await import('~/wasm/pkg');
    await init();

    const viewer = new FileViewer('image/png');
    // Load and render file...
    viewerHtml.value = viewer.render();
  });

  return <div dangerouslySetInnerHTML={viewerHtml.value} />;
});
```

## Performance

- Compiled with `opt-level = "z"` for minimal size
- LTO enabled for better optimization
- Typical bundle size: ~50KB (gzipped)

## Development

```bash
# Watch mode (requires wasm-pack-watch)
cargo watch -s 'wasm-pack build --target web'
```
