import { Component, Show, JSX } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { createEffect } from 'solid-js';
import { NO_AUTH } from '~/lib/env';

interface ProtectedRouteProps {
  children: JSX.Element;
}

const ProtectedRoute: Component<ProtectedRouteProps> = (props) => {
  const navigate = useNavigate();
  
  createEffect(() => {
    if (NO_AUTH) return;
    const token = localStorage.getItem('access_token');
    if (!token) {
      navigate('/login', { replace: true });
    }
  });

  const isAuthenticated = () => {
    if (NO_AUTH) return true;
    return !!localStorage.getItem('access_token');
  };

  return (
    <Show when={isAuthenticated()} fallback={null}>
      {props.children}
    </Show>
  );
};

export default ProtectedRoute;
