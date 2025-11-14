import { Component } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent, Button } from '~/components/ui';

const Files: Component = () => {
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
          <Button variant="primary" size="md">
            ⬆️ Upload File
          </Button>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>File Manager</CardTitle>
        </CardHeader>
        <CardContent>
          <p class="text-neutral-darkGray text-center py-12">
            📁 No files uploaded yet. Click "Upload File" to get started.
          </p>
        </CardContent>
      </Card>
    </div>
  );
};

export default Files;
