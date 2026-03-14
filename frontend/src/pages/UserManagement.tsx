import { Component, createSignal, For, Show } from 'solid-js';
import { Button, Card, Input, Label, Badge, Spinner } from '~/components/ui';
import {
  useUsers,
  useSearchUsers,
  useUserStats,
  useCreateUser,
  useUpdateUserAdmin,
  useDeleteUser,
  useBulkUserActions,
  type CreateUserData,
  type UpdateUserAdminData,
} from '~/lib/hooks/useUsers';
import { toast } from '~/lib/toast';

const UserManagement: Component = () => {
  const [page, setPage] = createSignal(1);
  const [searchTerm, setSearchTerm] = createSignal('');
  const [selectedUsers, setSelectedUsers] = createSignal<string[]>([]);
  const [showCreateModal, setShowCreateModal] = createSignal(false);
  const [editingUser, setEditingUser] = createSignal<any>(null);

  const users = useUsers(page(), 20);
  const searchResults = useSearchUsers(searchTerm(), page(), 20);
  const stats = useUserStats();
  const createUser = useCreateUser();
  const updateUser = useUpdateUserAdmin();
  const deleteUser = useDeleteUser();
  const bulkActions = useBulkUserActions();

  const [createForm, setCreateForm] = createSignal<CreateUserData>({
    email: '',
    full_name: '',
    password: '',
    role: 'user',
  });

  const [editForm, setEditForm] = createSignal<UpdateUserAdminData>({});

  const displayData = () => (searchTerm() ? searchResults : users);

  const handleCreateUser = async (e: Event) => {
    e.preventDefault();

    try {
      await createUser.mutateAsync(createForm());
      toast.success('User created successfully');
      setShowCreateModal(false);
      setCreateForm({
        email: '',
        full_name: '',
        password: '',
        role: 'user',
      });
    } catch (error: any) {
      toast.error('Create failed', error.message);
    }
  };

  const handleUpdateUser = async (e: Event) => {
    e.preventDefault();
    const user = editingUser();
    if (!user) return;

    try {
      await updateUser.mutateAsync({
        id: user.id,
        data: editForm(),
      });
      toast.success('User updated successfully');
      setEditingUser(null);
      setEditForm({});
    } catch (error: any) {
      toast.error('Update failed', error.message);
    }
  };

  const handleDeleteUser = async (userId: string, userName: string) => {
    if (!confirm(`Are you sure you want to delete ${userName}?`)) return;

    try {
      await deleteUser.mutateAsync(userId);
      toast.success('User deleted successfully');
    } catch (error: any) {
      toast.error('Delete failed', error.message);
    }
  };

  const handleBulkAction = async (action: string) => {
    const selected = selectedUsers();
    if (selected.length === 0) {
      toast.warning('No users selected');
      return;
    }

    let role: string | undefined;
    if (action === 'change_role') {
      role = prompt('Enter new role (admin/manager/user):');
      if (!role || !['admin', 'manager', 'user'].includes(role)) {
        toast.error('Invalid role');
        return;
      }
    }

    if (action === 'delete' && !confirm(`Delete ${selected.length} users?`)) {
      return;
    }

    try {
      const result = await bulkActions.mutateAsync({
        user_ids: selected,
        action: action as any,
        role,
      });

      toast.success(`Bulk action completed: ${result.success} succeeded, ${result.failed} failed`);
      
      if (result.errors.length > 0) {
        result.errors.forEach((error: string) => {
          toast.error('Error', error);
        });
      }

      setSelectedUsers([]);
    } catch (error: any) {
      toast.error('Bulk action failed', error.message);
    }
  };

  const toggleUserSelection = (userId: string) => {
    setSelectedUsers((prev) =>
      prev.includes(userId) ? prev.filter((id) => id !== userId) : [...prev, userId]
    );
  };

  const getRoleBadgeColor = (role: string) => {
    switch (role) {
      case 'admin':
        return 'bg-red-400';
      case 'manager':
        return 'bg-blue-400';
      default:
        return 'bg-gray-400';
    }
  };

  const getStatusBadgeColor = (status: string) => {
    return status === 'active' ? 'bg-green-400' : 'bg-gray-400';
  };

  return (
    <div class="space-y-6">
      <div class="flex justify-between items-center">
        <h1 class="text-4xl font-black">User Management</h1>
        <Button onClick={() => setShowCreateModal(true)}>+ Create User</Button>
      </div>

      {/* Statistics */}
      <Show when={stats.data}>
        {(data) => (
          <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
            <Card>
              <div class="text-center">
                <div class="text-3xl font-black">{data().total}</div>
                <div class="text-sm font-bold">Total Users</div>
              </div>
            </Card>
            <Card>
              <div class="text-center">
                <div class="text-3xl font-black text-green-600">{data().active}</div>
                <div class="text-sm font-bold">Active</div>
              </div>
            </Card>
            <Card>
              <div class="text-center">
                <div class="text-3xl font-black text-red-600">{data().by_role.admin}</div>
                <div class="text-sm font-bold">Admins</div>
              </div>
            </Card>
            <Card>
              <div class="text-center">
                <div class="text-3xl font-black text-blue-600">{data().by_role.manager}</div>
                <div class="text-sm font-bold">Managers</div>
              </div>
            </Card>
          </div>
        )}
      </Show>

      {/* Search and Bulk Actions */}
      <Card>
        <div class="flex flex-col md:flex-row gap-4">
          <div class="flex-1">
            <Input
              type="search"
              placeholder="Search users by name or email..."
              value={searchTerm()}
              onInput={(e) => setSearchTerm(e.currentTarget.value)}
            />
          </div>
          <div class="flex gap-2">
            <Button
              onClick={() => handleBulkAction('activate')}
              variant="outline"
              size="sm"
              disabled={selectedUsers().length === 0}
            >
              Activate
            </Button>
            <Button
              onClick={() => handleBulkAction('deactivate')}
              variant="outline"
              size="sm"
              disabled={selectedUsers().length === 0}
            >
              Deactivate
            </Button>
            <Button
              onClick={() => handleBulkAction('change_role')}
              variant="outline"
              size="sm"
              disabled={selectedUsers().length === 0}
            >
              Change Role
            </Button>
            <Button
              onClick={() => handleBulkAction('delete')}
              variant="outline"
              size="sm"
              disabled={selectedUsers().length === 0}
              class="!bg-red-400 hover:!bg-red-500"
            >
              Delete Selected
            </Button>
          </div>
        </div>
      </Card>

      {/* Users Table */}
      <Card>
        <Show when={displayData().isLoading}>
          <div class="flex justify-center p-12">
            <Spinner size="lg" />
          </div>
        </Show>

        <Show when={displayData().data}>
          {(data) => (
            <>
              <div class="overflow-x-auto">
                <table class="w-full">
                  <thead>
                    <tr class="border-b-4 border-black">
                      <th class="text-left p-3 font-black">
                        <input
                          type="checkbox"
                          onChange={(e) => {
                            if (e.currentTarget.checked) {
                              setSelectedUsers(data().data.map((u: any) => u.id));
                            } else {
                              setSelectedUsers([]);
                            }
                          }}
                        />
                      </th>
                      <th class="text-left p-3 font-black">Name</th>
                      <th class="text-left p-3 font-black">Email</th>
                      <th class="text-left p-3 font-black">Role</th>
                      <th class="text-left p-3 font-black">Status</th>
                      <th class="text-left p-3 font-black">Created</th>
                      <th class="text-right p-3 font-black">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={data().data}>
                      {(user: any) => (
                        <tr class="border-b-2 border-black hover:bg-gray-100">
                          <td class="p-3">
                            <input
                              type="checkbox"
                              checked={selectedUsers().includes(user.id)}
                              onChange={() => toggleUserSelection(user.id)}
                            />
                          </td>
                          <td class="p-3 font-bold">{user.full_name}</td>
                          <td class="p-3">{user.email}</td>
                          <td class="p-3">
                            <span
                              class={`px-2 py-1 text-xs font-bold uppercase ${getRoleBadgeColor(
                                user.role
                              )}`}
                            >
                              {user.role}
                            </span>
                          </td>
                          <td class="p-3">
                            <span
                              class={`px-2 py-1 text-xs font-bold uppercase ${getStatusBadgeColor(
                                user.status || 'active'
                              )}`}
                            >
                              {user.status || 'active'}
                            </span>
                          </td>
                          <td class="p-3">
                            {new Date(user.created_at).toLocaleDateString()}
                          </td>
                          <td class="p-3 text-right">
                            <div class="flex gap-2 justify-end">
                              <Button
                                onClick={() => {
                                  setEditingUser(user);
                                  setEditForm({
                                    full_name: user.full_name,
                                    email: user.email,
                                    role: user.role,
                                    status: user.status || 'active',
                                  });
                                }}
                                size="sm"
                                variant="outline"
                              >
                                Edit
                              </Button>
                              <Button
                                onClick={() => handleDeleteUser(user.id, user.full_name)}
                                size="sm"
                                variant="outline"
                                class="!bg-red-400 hover:!bg-red-500"
                              >
                                Delete
                              </Button>
                            </div>
                          </td>
                        </tr>
                      )}
                    </For>
                  </tbody>
                </table>
              </div>

              {/* Pagination */}
              <div class="flex justify-between items-center pt-4 border-t-4 border-black mt-4">
                <div class="text-sm font-bold">
                  Page {data().pagination.page} of {data().pagination.total_pages} (
                  {data().pagination.total} total)
                </div>
                <div class="flex gap-2">
                  <Button
                    onClick={() => setPage(page() - 1)}
                    disabled={page() === 1}
                    size="sm"
                    variant="outline"
                  >
                    Previous
                  </Button>
                  <Button
                    onClick={() => setPage(page() + 1)}
                    disabled={!data().pagination.has_next}
                    size="sm"
                    variant="outline"
                  >
                    Next
                  </Button>
                </div>
              </div>
            </>
          )}
        </Show>
      </Card>

      {/* Create User Modal */}
      <Show when={showCreateModal()}>
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <Card class="w-full max-w-md">
            <h2 class="text-2xl font-black mb-4">Create New User</h2>
            <form onSubmit={handleCreateUser} class="space-y-4">
              <div>
                <Label>Full Name</Label>
                <Input
                  value={createForm().full_name}
                  onInput={(e) =>
                    setCreateForm({ ...createForm(), full_name: e.currentTarget.value })
                  }
                  required
                />
              </div>
              <div>
                <Label>Email</Label>
                <Input
                  type="email"
                  value={createForm().email}
                  onInput={(e) =>
                    setCreateForm({ ...createForm(), email: e.currentTarget.value })
                  }
                  required
                />
              </div>
              <div>
                <Label>Password</Label>
                <Input
                  type="password"
                  value={createForm().password}
                  onInput={(e) =>
                    setCreateForm({ ...createForm(), password: e.currentTarget.value })
                  }
                  required
                  minLength={6}
                />
              </div>
              <div>
                <Label>Role</Label>
                <select
                  class="w-full p-3 border-4 border-black font-bold"
                  value={createForm().role}
                  onChange={(e) =>
                    setCreateForm({
                      ...createForm(),
                      role: e.currentTarget.value as any,
                    })
                  }
                >
                  <option value="user">User</option>
                  <option value="manager">Manager</option>
                  <option value="admin">Admin</option>
                </select>
              </div>
              <div class="flex gap-2">
                <Button type="submit" loading={createUser.isPending}>
                  Create User
                </Button>
                <Button
                  type="button"
                  onClick={() => setShowCreateModal(false)}
                  variant="outline"
                >
                  Cancel
                </Button>
              </div>
            </form>
          </Card>
        </div>
      </Show>

      {/* Edit User Modal */}
      <Show when={editingUser()}>
        {(user) => (
          <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <Card class="w-full max-w-md">
              <h2 class="text-2xl font-black mb-4">Edit User</h2>
              <form onSubmit={handleUpdateUser} class="space-y-4">
                <div>
                  <Label>Full Name</Label>
                  <Input
                    value={editForm().full_name}
                    onInput={(e) =>
                      setEditForm({ ...editForm(), full_name: e.currentTarget.value })
                    }
                  />
                </div>
                <div>
                  <Label>Email</Label>
                  <Input
                    type="email"
                    value={editForm().email}
                    onInput={(e) =>
                      setEditForm({ ...editForm(), email: e.currentTarget.value })
                    }
                  />
                </div>
                <div>
                  <Label>Role</Label>
                  <select
                    class="w-full p-3 border-4 border-black font-bold"
                    value={editForm().role}
                    onChange={(e) =>
                      setEditForm({ ...editForm(), role: e.currentTarget.value as any })
                    }
                  >
                    <option value="user">User</option>
                    <option value="manager">Manager</option>
                    <option value="admin">Admin</option>
                  </select>
                </div>
                <div>
                  <Label>Status</Label>
                  <select
                    class="w-full p-3 border-4 border-black font-bold"
                    value={editForm().status}
                    onChange={(e) =>
                      setEditForm({ ...editForm(), status: e.currentTarget.value as any })
                    }
                  >
                    <option value="active">Active</option>
                    <option value="inactive">Inactive</option>
                  </select>
                </div>
                <div class="flex gap-2">
                  <Button type="submit" loading={updateUser.isPending}>
                    Update User
                  </Button>
                  <Button
                    type="button"
                    onClick={() => setEditingUser(null)}
                    variant="outline"
                  >
                    Cancel
                  </Button>
                </div>
              </form>
            </Card>
          </div>
        )}
      </Show>
    </div>
  );
};

export default UserManagement;
