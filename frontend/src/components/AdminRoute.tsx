import { Component, Show, createEffect } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { NO_AUTH } from '~/lib/env';
import { isAuthenticated, hasRole } from '~/lib/auth';

interface AdminRouteProps {
  children: any;
}

const AdminRoute: Component<AdminRouteProps> = (props) => {
  const navigate = useNavigate();

  createEffect(() => {
    if (NO_AUTH) return;
    if (!isAuthenticated()) {
      navigate('/login', { replace: true });
      return;
    }
    if (!hasRole('admin')) {
      navigate('/', { replace: true });
    }
  });

  return (
    <Show when={NO_AUTH || (isAuthenticated() && hasRole('admin'))} fallback={
      <div class="flex items-center justify-center min-h-screen">
        <div class="border-4 border-black bg-white p-8 shadow-brutal">
          <h2 class="font-heading text-2xl font-bold mb-4">Access Denied</h2>
          <p class="mb-4">You need admin privileges to access this page.</p>
          <button
            onClick={() => navigate('/')}
            class="btn btn-primary"
          >
            Go Home
          </button>
        </div>
      </div>
    }>
      {props.children}
    </Show>
  );
};

export default AdminRoute;
