import { Component, Show, JSX, createEffect } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { NO_AUTH } from '~/lib/env';
import { isAuthenticated } from '~/lib/auth';

interface ProtectedRouteProps {
  children: JSX.Element;
}

const ProtectedRoute: Component<ProtectedRouteProps> = (props) => {
  const navigate = useNavigate();

  createEffect(() => {
    if (NO_AUTH) return;
    if (!isAuthenticated()) {
      navigate('/login', { replace: true });
    }
  });

  const allowed = () => NO_AUTH || isAuthenticated();

  return (
    <Show when={allowed()} fallback={null}>
      {props.children}
    </Show>
  );
};

export default ProtectedRoute;
