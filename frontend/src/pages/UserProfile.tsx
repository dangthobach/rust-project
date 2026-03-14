import { Component, createSignal, Show } from 'solid-js';
import { Button, Card, Input, Label, Spinner } from '~/components/ui';
import { 
  useUserProfile, 
  useUpdateProfile, 
  useChangePassword, 
  useUploadAvatar 
} from '~/lib/hooks/useUsers';
import { toast } from '~/lib/toast';

const UserProfile: Component = () => {
  const profile = useUserProfile();
  const updateProfile = useUpdateProfile();
  const changePassword = useChangePassword();
  const uploadAvatar = useUploadAvatar();

  const [isEditing, setIsEditing] = createSignal(false);
  const [formData, setFormData] = createSignal({
    full_name: '',
    email: '',
  });

  const [passwordData, setPasswordData] = createSignal({
    current_password: '',
    new_password: '',
    confirm_password: '',
  });

  const [showPasswordForm, setShowPasswordForm] = createSignal(false);
  const [avatarFile, setAvatarFile] = createSignal<File | null>(null);
  const [avatarPreview, setAvatarPreview] = createSignal<string | null>(null);

  // Initialize form with profile data
  const initializeForm = () => {
    if (profile.data) {
      setFormData({
        full_name: profile.data.user.full_name,
        email: profile.data.user.email,
      });
    }
  };

  const handleEdit = () => {
    initializeForm();
    setIsEditing(true);
  };

  const handleCancel = () => {
    setIsEditing(false);
    initializeForm();
  };

  const handleSave = async () => {
    if (!profile.data) return;

    try {
      await updateProfile.mutateAsync({
        id: profile.data.user.id,
        data: formData(),
      });

      toast.success('Profile updated successfully');
      setIsEditing(false);
    } catch (error: any) {
      toast.error('Update failed', error.message);
    }
  };

  const handlePasswordChange = async (e: Event) => {
    e.preventDefault();

    const data = passwordData();
    
    if (data.new_password !== data.confirm_password) {
      toast.error('Passwords do not match');
      return;
    }

    if (data.new_password.length < 6) {
      toast.error('Password must be at least 6 characters');
      return;
    }

    try {
      await changePassword.mutateAsync({
        current_password: data.current_password,
        new_password: data.new_password,
      });

      toast.success('Password changed successfully');
      setShowPasswordForm(false);
      setPasswordData({
        current_password: '',
        new_password: '',
        confirm_password: '',
      });
    } catch (error: any) {
      toast.error('Password change failed', error.message);
    }
  };

  const handleAvatarSelect = (e: Event) => {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];

    if (!file) return;

    // Validate file type
    if (!file.type.startsWith('image/')) {
      toast.error('Please select an image file');
      return;
    }

    // Validate file size (2MB)
    if (file.size > 2 * 1024 * 1024) {
      toast.error('Image size must be less than 2MB');
      return;
    }

    setAvatarFile(file);

    // Create preview
    const reader = new FileReader();
    reader.onload = (e) => {
      setAvatarPreview(e.target?.result as string);
    };
    reader.readAsDataURL(file);
  };

  const handleAvatarUpload = async () => {
    const file = avatarFile();
    if (!file) return;

    try {
      await uploadAvatar.mutateAsync(file);
      toast.success('Avatar uploaded successfully');
      setAvatarFile(null);
      setAvatarPreview(null);
    } catch (error: any) {
      toast.error('Avatar upload failed', error.message);
    }
  };

  return (
    <div class="space-y-6">
      <div class="flex justify-between items-center">
        <h1 class="text-4xl font-black">My Profile</h1>
      </div>

      <Show when={profile.isLoading}>
        <div class="flex justify-center p-12">
          <Spinner size="lg" />
        </div>
      </Show>

      <Show when={profile.data}>
        {(data) => (
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Avatar Section */}
            <Card class="lg:col-span-1">
              <div class="space-y-4">
                <h2 class="text-2xl font-bold">Avatar</h2>
                
                <div class="flex flex-col items-center space-y-4">
                  <div class="relative">
                    <div class="w-32 h-32 rounded-lg border-4 border-black overflow-hidden bg-gray-200">
                      <Show
                        when={avatarPreview() || data().user.avatar_url}
                        fallback={
                          <div class="w-full h-full flex items-center justify-center text-4xl font-bold">
                            {data().user.full_name.charAt(0).toUpperCase()}
                          </div>
                        }
                      >
                        <img
                          src={avatarPreview() || data().user.avatar_url}
                          alt="Avatar"
                          class="w-full h-full object-cover"
                        />
                      </Show>
                    </div>
                  </div>

                  <input
                    type="file"
                    accept="image/*"
                    onChange={handleAvatarSelect}
                    class="hidden"
                    id="avatar-upload"
                  />
                  
                  <Show
                    when={avatarFile()}
                    fallback={
                      <label for="avatar-upload">
                        <Button as="span" variant="outline" size="sm">
                          Choose Image
                        </Button>
                      </label>
                    }
                  >
                    <div class="flex gap-2">
                      <Button
                        onClick={handleAvatarUpload}
                        loading={uploadAvatar.isPending}
                        size="sm"
                      >
                        Upload
                      </Button>
                      <Button
                        onClick={() => {
                          setAvatarFile(null);
                          setAvatarPreview(null);
                        }}
                        variant="outline"
                        size="sm"
                      >
                        Cancel
                      </Button>
                    </div>
                  </Show>
                </div>

                {/* User Stats */}
                <div class="pt-4 border-t-4 border-black space-y-2">
                  <div class="flex justify-between">
                    <span class="font-bold">Tasks Created:</span>
                    <span>{data().stats.tasks_created}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="font-bold">Tasks Completed:</span>
                    <span>{data().stats.tasks_completed}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="font-bold">Clients Assigned:</span>
                    <span>{data().stats.clients_assigned}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="font-bold">Files Uploaded:</span>
                    <span>{data().stats.files_uploaded}</span>
                  </div>
                </div>
              </div>
            </Card>

            {/* Profile Information */}
            <Card class="lg:col-span-2">
              <div class="space-y-4">
                <div class="flex justify-between items-center">
                  <h2 class="text-2xl font-bold">Profile Information</h2>
                  <Show
                    when={isEditing()}
                    fallback={
                      <Button onClick={handleEdit} size="sm">
                        Edit Profile
                      </Button>
                    }
                  >
                    <div class="flex gap-2">
                      <Button
                        onClick={handleSave}
                        loading={updateProfile.isPending}
                        size="sm"
                      >
                        Save
                      </Button>
                      <Button onClick={handleCancel} variant="outline" size="sm">
                        Cancel
                      </Button>
                    </div>
                  </Show>
                </div>

                <div class="space-y-4">
                  <div>
                    <Label>Full Name</Label>
                    <Show
                      when={isEditing()}
                      fallback={
                        <div class="p-3 border-4 border-black bg-white">
                          {data().user.full_name}
                        </div>
                      }
                    >
                      <Input
                        value={formData().full_name}
                        onInput={(e) =>
                          setFormData({ ...formData(), full_name: e.currentTarget.value })
                        }
                      />
                    </Show>
                  </div>

                  <div>
                    <Label>Email</Label>
                    <Show
                      when={isEditing()}
                      fallback={
                        <div class="p-3 border-4 border-black bg-white">
                          {data().user.email}
                        </div>
                      }
                    >
                      <Input
                        type="email"
                        value={formData().email}
                        onInput={(e) =>
                          setFormData({ ...formData(), email: e.currentTarget.value })
                        }
                      />
                    </Show>
                  </div>

                  <div>
                    <Label>Role</Label>
                    <div class="p-3 border-4 border-black bg-gray-100">
                      <span class="px-3 py-1 bg-primary text-white font-bold uppercase">
                        {data().user.role}
                      </span>
                    </div>
                  </div>

                  <div>
                    <Label>Status</Label>
                    <div class="p-3 border-4 border-black bg-gray-100">
                      <span
                        class={`px-3 py-1 font-bold uppercase ${
                          data().user.status === 'active'
                            ? 'bg-green-400 text-black'
                            : 'bg-gray-400 text-white'
                        }`}
                      >
                        {data().user.status || 'active'}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </Card>

            {/* Password Change */}
            <Card class="lg:col-span-3">
              <div class="space-y-4">
                <div class="flex justify-between items-center">
                  <h2 class="text-2xl font-bold">Change Password</h2>
                  <Show when={!showPasswordForm()}>
                    <Button onClick={() => setShowPasswordForm(true)} size="sm">
                      Change Password
                    </Button>
                  </Show>
                </div>

                <Show when={showPasswordForm()}>
                  <form onSubmit={handlePasswordChange} class="space-y-4">
                    <div>
                      <Label>Current Password</Label>
                      <Input
                        type="password"
                        value={passwordData().current_password}
                        onInput={(e) =>
                          setPasswordData({
                            ...passwordData(),
                            current_password: e.currentTarget.value,
                          })
                        }
                        required
                      />
                    </div>

                    <div>
                      <Label>New Password</Label>
                      <Input
                        type="password"
                        value={passwordData().new_password}
                        onInput={(e) =>
                          setPasswordData({
                            ...passwordData(),
                            new_password: e.currentTarget.value,
                          })
                        }
                        required
                        minLength={6}
                      />
                    </div>

                    <div>
                      <Label>Confirm New Password</Label>
                      <Input
                        type="password"
                        value={passwordData().confirm_password}
                        onInput={(e) =>
                          setPasswordData({
                            ...passwordData(),
                            confirm_password: e.currentTarget.value,
                          })
                        }
                        required
                        minLength={6}
                      />
                    </div>

                    <div class="flex gap-2">
                      <Button type="submit" loading={changePassword.isPending}>
                        Update Password
                      </Button>
                      <Button
                        type="button"
                        onClick={() => {
                          setShowPasswordForm(false);
                          setPasswordData({
                            current_password: '',
                            new_password: '',
                            confirm_password: '',
                          });
                        }}
                        variant="outline"
                      >
                        Cancel
                      </Button>
                    </div>
                  </form>
                </Show>
              </div>
            </Card>
          </div>
        )}
      </Show>
    </div>
  );
};

export default UserProfile;
