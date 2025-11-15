import { Component, Show, JSX } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { createEffect } from 'solid-js';

interface ProtectedRouteProps {
  children: JSX.Element;
}

const ProtectedRoute: Component<ProtectedRouteProps> = (props) => {
  const navigate = useNavigate();
  
  createEffect(() => {
    const token = localStorage.getItem('token');
    if (!token) {
      navigate('/login', { replace: true });
    }
  });

  const isAuthenticated = () => {
    return !!localStorage.getItem('token');
  };

  return (
    <Show when={isAuthenticated()} fallback={null}>
      {props.children}
    </Show>
  );
};

export default ProtectedRoute;
