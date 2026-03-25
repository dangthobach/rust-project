/**
 * ManagerRoute Component
 * Protects manager-only routes and redirects non-manager users
 */

import { Component, Show, createEffect } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { NO_AUTH } from '~/lib/env';

interface ManagerRouteProps {
  children: any;
}

/**
 * Wrapper component for manager+ routes
 * Automatically redirects non-manager users to home page
 */
const ManagerRoute: Component<ManagerRouteProps> = (props) => {
  const navigate = useNavigate();

  createEffect(() => {
    if (NO_AUTH) return;
    // Auth deferred (Keycloak PKCE). For now, treat as non-manager.
    navigate('/', { replace: true });
  });

  return (
    <Show when={NO_AUTH} fallback={
      <div class="flex items-center justify-center min-h-screen">
        <div class="border-4 border-black bg-white p-8 shadow-brutal">
          <h2 class="font-heading text-2xl font-bold mb-4">Access Denied</h2>
          <p class="mb-4">You need manager privileges to access this page.</p>
          <button
            onClick={() => navigate('/')}
            class="px-6 py-3 font-heading font-bold uppercase border-4 border-black bg-primary text-white shadow-brutal hover:shadow-brutal-lg transition-all"
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

export default ManagerRoute;
