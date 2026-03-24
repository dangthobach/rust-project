import { Component, createSignal, Show, For, createMemo } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent, Button, Spinner } from '~/components/ui';
import { 
  useUsers, 
  useSearchUsers,
  useCreateUser, 
  useUpdateUserAdmin, 
  useDeleteUser,
  useBulkUserActions,
  useUserStats
} from '~/lib/hooks/useUsers';
import { showToast } from '~/lib/toast';
import type { User } from '~/lib/api';

const Users: Component = () => {
  const [page, setPage] = createSignal(1);
  const [limit] = createSignal(20);
  const [search, setSearch] = createSignal('');
  const [roleFilter, setRoleFilter] = createSignal<'admin' | 'manager' | 'user' | ''>('');
  const [selectedIds, setSelectedIds] = createSignal<string[]>([]);
  const [showCreateModal, setShowCreateModal] = createSignal(false);
  const [showEditModal, setShowEditModal] = createSignal<User | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = createSignal<string | null>(null);
  const [showBulkRoleModal, setShowBulkRoleModal] = createSignal(false);

  // Form states
  const [createForm, setCreateForm] = createSignal({
    email: '',
    name: '',
    password: '',
    role: 'user' as 'admin' | 'manager' | 'user'
  });

  const [editForm, setEditForm] = createSignal({
    name: '',
    email: '',
    role: 'user' as 'admin' | 'manager' | 'user'
  });

  const [bulkRole, setBulkRole] = createSignal<'admin' | 'manager' | 'user'>('user');

  // Queries and mutations
  const users = useUsers(() => ({ 
    page: page(), 
    limit: limit(),
    role: roleFilter() || undefined
  }));
  
  const searchUsers = useSearchUsers(
    () => search(),
    () => ({ page: page(), limit: limit() })
  );
  
  const stats = useUserStats();
  const createUser = useCreateUser();
  const updateUser = useUpdateUserAdmin();
  const deleteUser = useDeleteUser();
  const bulkActions = useBulkUserActions();

  const displayUsers = createMemo(() => {
    if (search().length > 0) {
      return searchUsers.data?.data || [];
    }
    return users.data?.data || [];
  });

  const pagination = createMemo(() => {
    if (search().length > 0) {
      return searchUsers.data?.pagination;
    }
    return users.data?.pagination;
  });

  const isLoading = createMemo(() => {
    if (search().length > 0) {
      return searchUsers.isLoading;
    }
    return users.isLoading;
  });

  // Handlers
  const handleCreateSubmit = async (e: Event) => {
    e.preventDefault();
    const form = createForm();
    
    if (!form.email || !form.name || !form.password) {
      showToast('error', 'Please fill in all fields');
      return;
    }

    createUser.mutate(form, {
      onSuccess: () => {
        setShowCreateModal(false);
        setCreateForm({ email: '', name: '', password: '', role: 'user' });
      }
    });
  };

  const handleEditClick = (user: User) => {
    setEditForm({
      name: user.name,
      email: user.email,
      role: user.role
    });
    setShowEditModal(user);
  };

  const handleEditSubmit = async (e: Event) => {
    e.preventDefault();
    const user = showEditModal();
    if (!user) return;

    const form = editForm();
    updateUser.mutate({ id: user.id, data: form }, {
      onSuccess: () => {
        setShowEditModal(null);
      }
    });
  };

  const handleDeleteClick = (id: string) => {
    setShowDeleteConfirm(id);
  };

  const handleDeleteConfirm = () => {
    const id = showDeleteConfirm();
    if (id) {
      deleteUser.mutate(id, {
        onSuccess: () => {
          setShowDeleteConfirm(null);
        }
      });
    }
  };

  const handleBulkDelete = () => {
    if (selectedIds().length === 0) return;
    
    if (confirm(`Delete ${selectedIds().length} selected users?`)) {
      bulkActions.bulkDelete.mutate(selectedIds(), {
        onSuccess: () => {
          setSelectedIds([]);
        }
      });
    }
  };

  const handleBulkRoleSubmit = () => {
    if (selectedIds().length === 0) return;

    bulkActions.bulkUpdateRole.mutate(
      { ids: selectedIds(), role: bulkRole() },
      {
        onSuccess: () => {
          setShowBulkRoleModal(false);
          setSelectedIds([]);
        }
      }
    );
  };

  const toggleSelection = (id: string) => {
    setSelectedIds(prev => 
      prev.includes(id) 
        ? prev.filter(uid => uid !== id)
        : [...prev, id]
    );
  };

  const toggleSelectAll = () => {
    const allIds = displayUsers().map(u => u.id);
    if (selectedIds().length === allIds.length) {
      setSelectedIds([]);
    } else {
      setSelectedIds(allIds);
    }
  };

  // Utility functions
  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  const getRoleBadge = (role: string) => {
    switch (role) {
      case 'admin':
        return 'bg-red-500 text-white';
      case 'manager':
        return 'bg-blue-500 text-white';
      default:
        return 'bg-gray-500 text-white';
    }
  };

  return (
    <div>
      {/* Header */}
      <div class="mb-8">
        <div class="flex items-center justify-between flex-wrap gap-4">
          <div>
            <h1 class="text-heading-1 font-heading font-black uppercase text-shadow-brutal">
              User Management
            </h1>
            <p class="text-neutral-darkGray mt-1">
              Manage system users and permissions (Admin Only)
            </p>
          </div>
          
          <div class="flex gap-3 flex-wrap">
            <Show when={selectedIds().length > 0}>
              <Button 
                variant="ghost" 
                size="md"
                onClick={() => setShowBulkRoleModal(true)}
              >
                👥 Change Role ({selectedIds().length})
              </Button>
              <Button 
                variant="danger" 
                size="md"
                onClick={handleBulkDelete}
              >
                🗑️ Delete Selected ({selectedIds().length})
              </Button>
            </Show>
            
            <Button 
              variant="primary" 
              size="md"
              onClick={() => setShowCreateModal(true)}
            >
              ➕ Create User
            </Button>
          </div>
        </div>
      </div>

      {/* Stats Cards */}
      <Show when={stats.data}>
        {(s) => (
          <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
            <div class="p-4 border-3 border-black bg-white">
              <div class="text-2xl font-bold">{s().total}</div>
              <div class="text-sm text-neutral-darkGray">Total Users</div>
            </div>
            <div class="p-4 border-3 border-black bg-red-50">
              <div class="text-2xl font-bold">{s().byRole.admin}</div>
              <div class="text-sm text-neutral-darkGray">Admins</div>
            </div>
            <div class="p-4 border-3 border-black bg-blue-50">
              <div class="text-2xl font-bold">{s().byRole.manager}</div>
              <div class="text-sm text-neutral-darkGray">Managers</div>
            </div>
            <div class="p-4 border-3 border-black bg-gray-50">
              <div class="text-2xl font-bold">{s().byRole.user}</div>
              <div class="text-sm text-neutral-darkGray">Users</div>
            </div>
          </div>
        )}
      </Show>

      {/* Search and Filter Bar */}
      <div class="mb-6 flex gap-4 flex-wrap">
        <div class="flex-1 min-w-[300px]">
          <input
            type="text"
            placeholder="🔍 Search users by name or email..."
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
          value={roleFilter()}
          onChange={(e) => {
            setRoleFilter(e.currentTarget.value as any);
            setPage(1);
          }}
        >
          <option value="">All Roles</option>
          <option value="admin">Admin</option>
          <option value="manager">Manager</option>
          <option value="user">User</option>
        </select>
      </div>

      {/* Users List */}
      <Card>
        <CardHeader>
          <div class="flex items-center justify-between">
            <CardTitle>
              Users
              <Show when={pagination()}>
                {(p) => (
                  <span class="ml-2 text-sm font-normal text-neutral-darkGray">
                    ({p().total} total)
                  </span>
                )}
              </Show>
            </CardTitle>
            
            <Show when={displayUsers().length > 0}>
              <Button
                variant="ghost"
                size="sm"
                onClick={toggleSelectAll}
              >
                {selectedIds().length === displayUsers().length ? '☑️ Deselect All' : '☐ Select All'}
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
              when={displayUsers().length > 0}
              fallback={
                <div class="text-center py-12">
                  <div class="text-6xl mb-4">👥</div>
                  <p class="text-neutral-darkGray">
                    {search().length > 0 
                      ? `No users found matching "${search()}"`
                      : 'No users yet'
                    }
                  </p>
                </div>
              }
            >
              <div class="overflow-x-auto">
                <table class="w-full border-collapse">
                  <thead>
                    <tr class="border-b-3 border-black bg-neutral-lightGray">
                      <th class="p-3 text-left">
                        <input
                          type="checkbox"
                          class="w-5 h-5 cursor-pointer"
                          checked={selectedIds().length === displayUsers().length && displayUsers().length > 0}
                          onChange={toggleSelectAll}
                        />
                      </th>
                      <th class="p-3 text-left font-heading font-bold">Name</th>
                      <th class="p-3 text-left font-heading font-bold">Email</th>
                      <th class="p-3 text-left font-heading font-bold">Role</th>
                      <th class="p-3 text-left font-heading font-bold">Created</th>
                      <th class="p-3 text-right font-heading font-bold">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={displayUsers()}>
                      {(user) => (
                        <tr class="border-b-2 border-neutral-lightGray hover:bg-neutral-beige transition-colors group">
                          <td class="p-3">
                            <input
                              type="checkbox"
                              class="w-5 h-5 cursor-pointer"
                              checked={selectedIds().includes(user.id)}
                              onChange={() => toggleSelection(user.id)}
                            />
                          </td>
                          <td class="p-3">
                            <div class="flex items-center gap-2">
                              <Show when={user.avatar_url} fallback={
                                <div class="w-10 h-10 rounded-full bg-primary-yellow flex items-center justify-center font-bold">
                                  {user.name[0].toUpperCase()}
                                </div>
                              }>
                                <img 
                                  src={user.avatar_url} 
                                  alt={user.name}
                                  class="w-10 h-10 rounded-full border-2 border-black"
                                />
                              </Show>
                              <span class="font-bold">{user.name}</span>
                            </div>
                          </td>
                          <td class="p-3 text-sm text-neutral-darkGray">{user.email}</td>
                          <td class="p-3">
                            <span class={`px-2 py-1 text-xs font-bold rounded ${getRoleBadge(user.role)}`}>
                              {user.role.toUpperCase()}
                            </span>
                          </td>
                          <td class="p-3 text-sm text-neutral-darkGray">
                            {formatDate(user.created_at)}
                          </td>
                          <td class="p-3 text-right">
                            <div class="flex gap-2 justify-end opacity-0 group-hover:opacity-100 transition-opacity">
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => handleEditClick(user)}
                              >
                                ✏️ Edit
                              </Button>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => handleDeleteClick(user.id)}
                                class="text-red-600 hover:bg-red-50"
                              >
                                🗑️ Delete
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
              <Show when={pagination() && pagination()!.total_pages > 1}>
                {(p) => (
                  <div class="mt-6 flex items-center justify-center gap-2">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setPage(p => Math.max(1, p - 1))}
                      disabled={!p().has_prev}
                    >
                      ← Previous
                    </Button>
                    
                    <span class="px-4 py-2 font-bold">
                      Page {p().page} of {p().total_pages}
                    </span>
                    
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setPage(p => p + 1)}
                      disabled={!p().has_next}
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

      {/* Create User Modal */}
      <Show when={showCreateModal()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div class="bg-white border-4 border-black p-6 max-w-md w-full">
            <h3 class="text-xl font-heading font-black mb-4">Create New User</h3>
            <form onSubmit={handleCreateSubmit}>
              <div class="space-y-4">
                <div>
                  <label class="block font-bold mb-1">Name</label>
                  <input
                    type="text"
                    class="w-full px-3 py-2 border-2 border-black"
                    value={createForm().name}
                    onInput={(e) => setCreateForm({ ...createForm(), name: e.currentTarget.value })}
                    required
                  />
                </div>
                <div>
                  <label class="block font-bold mb-1">Email</label>
                  <input
                    type="email"
                    class="w-full px-3 py-2 border-2 border-black"
                    value={createForm().email}
                    onInput={(e) => setCreateForm({ ...createForm(), email: e.currentTarget.value })}
                    required
                  />
                </div>
                <div>
                  <label class="block font-bold mb-1">Password</label>
                  <input
                    type="password"
                    class="w-full px-3 py-2 border-2 border-black"
                    value={createForm().password}
                    onInput={(e) => setCreateForm({ ...createForm(), password: e.currentTarget.value })}
                    required
                    minLength={8}
                  />
                </div>
                <div>
                  <label class="block font-bold mb-1">Role</label>
                  <select
                    class="w-full px-3 py-2 border-2 border-black bg-white"
                    value={createForm().role}
                    onChange={(e) => setCreateForm({ ...createForm(), role: e.currentTarget.value as any })}
                  >
                    <option value="user">User</option>
                    <option value="manager">Manager</option>
                    <option value="admin">Admin</option>
                  </select>
                </div>
              </div>
              <div class="flex gap-3 justify-end mt-6">
                <Button
                  type="button"
                  variant="ghost"
                  size="md"
                  onClick={() => setShowCreateModal(false)}
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  variant="primary"
                  size="md"
                  disabled={createUser.isPending}
                >
                  {createUser.isPending ? 'Creating...' : 'Create User'}
                </Button>
              </div>
            </form>
          </div>
        </div>
      </Show>

      {/* Edit User Modal */}
      <Show when={showEditModal()}>
        {(user) => (
          <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
            <div class="bg-white border-4 border-black p-6 max-w-md w-full">
              <h3 class="text-xl font-heading font-black mb-4">Edit User</h3>
              <form onSubmit={handleEditSubmit}>
                <div class="space-y-4">
                  <div>
                    <label class="block font-bold mb-1">Name</label>
                    <input
                      type="text"
                      class="w-full px-3 py-2 border-2 border-black"
                      value={editForm().name}
                      onInput={(e) => setEditForm({ ...editForm(), name: e.currentTarget.value })}
                      required
                    />
                  </div>
                  <div>
                    <label class="block font-bold mb-1">Email</label>
                    <input
                      type="email"
                      class="w-full px-3 py-2 border-2 border-black"
                      value={editForm().email}
                      onInput={(e) => setEditForm({ ...editForm(), email: e.currentTarget.value })}
                      required
                    />
                  </div>
                  <div>
                    <label class="block font-bold mb-1">Role</label>
                    <select
                      class="w-full px-3 py-2 border-2 border-black bg-white"
                      value={editForm().role}
                      onChange={(e) => setEditForm({ ...editForm(), role: e.currentTarget.value as any })}
                    >
                      <option value="user">User</option>
                      <option value="manager">Manager</option>
                      <option value="admin">Admin</option>
                    </select>
                  </div>
                </div>
                <div class="flex gap-3 justify-end mt-6">
                  <Button
                    type="button"
                    variant="ghost"
                    size="md"
                    onClick={() => setShowEditModal(null)}
                  >
                    Cancel
                  </Button>
                  <Button
                    type="submit"
                    variant="primary"
                    size="md"
                    disabled={updateUser.isPending}
                  >
                    {updateUser.isPending ? 'Updating...' : 'Update User'}
                  </Button>
                </div>
              </form>
            </div>
          </div>
        )}
      </Show>

      {/* Delete Confirmation Modal */}
      <Show when={showDeleteConfirm()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div class="bg-white border-4 border-black p-6 max-w-md w-full">
            <h3 class="text-xl font-heading font-black mb-4">Delete User?</h3>
            <p class="text-neutral-darkGray mb-6">
              Are you sure you want to delete this user? This action cannot be undone.
            </p>
            <div class="flex gap-3 justify-end">
              <Button
                variant="ghost"
                size="md"
                onClick={() => setShowDeleteConfirm(null)}
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

      {/* Bulk Role Change Modal */}
      <Show when={showBulkRoleModal()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div class="bg-white border-4 border-black p-6 max-w-md w-full">
            <h3 class="text-xl font-heading font-black mb-4">Change Role for {selectedIds().length} Users</h3>
            <div class="space-y-4">
              <div>
                <label class="block font-bold mb-1">New Role</label>
                <select
                  class="w-full px-3 py-2 border-2 border-black bg-white"
                  value={bulkRole()}
                  onChange={(e) => setBulkRole(e.currentTarget.value as any)}
                >
                  <option value="user">User</option>
                  <option value="manager">Manager</option>
                  <option value="admin">Admin</option>
                </select>
              </div>
            </div>
            <div class="flex gap-3 justify-end mt-6">
              <Button
                variant="ghost"
                size="md"
                onClick={() => setShowBulkRoleModal(false)}
              >
                Cancel
              </Button>
              <Button
                variant="primary"
                size="md"
                onClick={handleBulkRoleSubmit}
                disabled={bulkActions.bulkUpdateRole.isPending}
              >
                {bulkActions.bulkUpdateRole.isPending ? 'Updating...' : 'Update Roles'}
              </Button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default Users;
