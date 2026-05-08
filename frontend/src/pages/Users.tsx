import { Component, createSignal, For, Show } from 'solid-js';
import { 
  Card, CardHeader, CardTitle, CardContent, CardFooter,
  Button, Badge, Spinner, Input, Label 
} from '~/components/ui';
import { useUsers, useCreateUser, useUpdateUser, useDeleteUser, type User } from '~/lib/hooks/useUsers';

const Users: Component = () => {
  const [page, setPage] = createSignal(1);
  const [limit] = createSignal(10);
  const [showCreateModal, setShowCreateModal] = createSignal(false);
  const [editingUser, setEditingUser] = createSignal<User | null>(null);

  const usersQuery = useUsers(() => ({ page: page(), limit: limit() }));
  const createUser = useCreateUser();
  const updateUser = useUpdateUser();
  const deleteUser = useDeleteUser();

  const handleCreate = async (e: Event) => {
    e.preventDefault();
    const formData = new FormData(e.target as HTMLFormElement);
    const data = Object.fromEntries(formData.entries());
    
    await createUser.mutateAsync(data as any);
    setShowCreateModal(false);
  };

  const handleUpdate = async (e: Event) => {
    e.preventDefault();
    const user = editingUser();
    if (!user) return;

    const formData = new FormData(e.target as HTMLFormElement);
    const data = Object.fromEntries(formData.entries());
    
    await updateUser.mutateAsync({ id: user.id, ...data } as any);
    setEditingUser(null);
  };

  const handleDelete = async (id: string) => {
    if (confirm('Are you sure you want to delete this user?')) {
      await deleteUser.mutateAsync(id);
    }
  };

  return (
    <div class="space-y-6">
      <div class="flex justify-between items-center">
        <h1 class="text-3xl font-bold">User Management</h1>
        <Button onClick={() => setShowCreateModal(true)}>Add User</Button>
      </div>

      <Card>
        <CardContent class="p-0">
          <div class="overflow-x-auto">
            <table class="w-full text-left border-collapse">
              <thead>
                <tr class="bg-neutral-beige border-b-2 border-black">
                  <th class="p-4 font-bold">Name</th>
                  <th class="p-4 font-bold">Email</th>
                  <th class="p-4 font-bold">Role</th>
                  <th class="p-4 font-bold">Status</th>
                  <th class="p-4 font-bold">Actions</th>
                </tr>
              </thead>
              <tbody>
                <Show when={!usersQuery.isLoading} fallback={
                  <tr>
                    <td colspan="5" class="p-8 text-center"><Spinner /></td>
                  </tr>
                }>
                  <For each={usersQuery.data?.data}>
                    {(user) => (
                      <tr class="border-b border-black hover:bg-neutral-beige/50 transition-colors">
                        <td class="p-4">{user.full_name}</td>
                        <td class="p-4">{user.email}</td>
                        <td class="p-4">
                          <Badge variant={user.role === 'admin' ? 'default' : 'secondary'}>
                            {user.role}
                          </Badge>
                        </td>
                        <td class="p-4">
                          <Badge variant={user.is_active ? 'success' : 'destructive'}>
                            {user.is_active ? 'Active' : 'Inactive'}
                          </Badge>
                        </td>
                        <td class="p-4 space-x-2">
                          <Button variant="outline" size="sm" onClick={() => setEditingUser(user)}>Edit</Button>
                          <Button variant="destructive" size="sm" onClick={() => handleDelete(user.id)}>Delete</Button>
                        </td>
                      </tr>
                    )}
                  </For>
                </Show>
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {/* Pagination */}
      <div class="flex justify-center gap-2">
        <Button 
          variant="outline" 
          disabled={page() === 1} 
          onClick={() => setPage(p => p - 1)}
        >
          Previous
        </Button>
        <div class="flex items-center px-4 font-bold border-2 border-black bg-white">
          Page {page()}
        </div>
        <Button 
          variant="outline" 
          disabled={!usersQuery.data?.pagination || page() >= usersQuery.data.pagination.total_pages} 
          onClick={() => setPage(p => p + 1)}
        >
          Next
        </Button>
      </div>

      {/* Create Modal */}
      <Show when={showCreateModal()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <Card class="w-full max-w-md m-4">
            <CardHeader>
              <CardTitle>Create New User</CardTitle>
            </CardHeader>
            <form onSubmit={handleCreate}>
              <CardContent class="space-y-4">
                <div class="space-y-2">
                  <Label>Full Name</Label>
                  <Input name="full_name" required />
                </div>
                <div class="space-y-2">
                  <Label>Email</Label>
                  <Input name="email" type="email" required />
                </div>
                <div class="space-y-2">
                  <Label>Password</Label>
                  <Input name="password" type="password" required />
                </div>
                <div class="space-y-2">
                  <Label>Role</Label>
                  <select name="role" class="w-full p-2 border-2 border-black rounded-none bg-white font-bold outline-none focus:translate-x-1 focus:translate-y-1 transition-transform">
                    <option value="user">User</option>
                    <option value="manager">Manager</option>
                    <option value="admin">Admin</option>
                  </select>
                </div>
              </CardContent>
              <CardFooter class="flex justify-end gap-2">
                <Button variant="outline" type="button" onClick={() => setShowCreateModal(false)}>Cancel</Button>
                <Button type="submit" disabled={createUser.isPending}>
                  {createUser.isPending ? 'Creating...' : 'Create'}
                </Button>
              </CardFooter>
            </form>
          </Card>
        </div>
      </Show>

      {/* Edit Modal */}
      <Show when={editingUser()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <Card class="w-full max-w-md m-4">
            <CardHeader>
              <CardTitle>Edit User</CardTitle>
            </CardHeader>
            <form onSubmit={handleUpdate}>
              <CardContent class="space-y-4">
                <div class="space-y-2">
                  <Label>Full Name</Label>
                  <Input name="full_name" value={editingUser()?.full_name} required />
                </div>
                <div class="space-y-2">
                  <Label>Email</Label>
                  <Input name="email" type="email" value={editingUser()?.email} required />
                </div>
                <div class="space-y-2">
                  <Label>Role</Label>
                  <select name="role" value={editingUser()?.role} class="w-full p-2 border-2 border-black rounded-none bg-white font-bold outline-none">
                    <option value="user">User</option>
                    <option value="manager">Manager</option>
                    <option value="admin">Admin</option>
                  </select>
                </div>
                <div class="flex items-center gap-2">
                  <input type="checkbox" name="is_active" checked={editingUser()?.is_active} class="w-5 h-5 border-2 border-black rounded-none accent-black" />
                  <Label>Active Account</Label>
                </div>
              </CardContent>
              <CardFooter class="flex justify-end gap-2">
                <Button variant="outline" type="button" onClick={() => setEditingUser(null)}>Cancel</Button>
                <Button type="submit" disabled={updateUser.isPending}>
                  {updateUser.isPending ? 'Updating...' : 'Save Changes'}
                </Button>
              </CardFooter>
            </form>
          </Card>
        </div>
      </Show>
    </div>
  );
};

export default Users;
