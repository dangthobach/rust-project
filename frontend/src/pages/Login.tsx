import { Component, createSignal, Show } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { Card, CardHeader, CardTitle, CardContent, Input, Button, Spinner } from '~/components/ui';
import { api } from '~/lib/api';

const Login: Component = () => {
  const navigate = useNavigate();
  const [email, setEmail] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal('');

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setLoading(true);
    setError('');

    try {
      const response = await api.login(email(), password());
      if (response.token) {
        // Token is already stored in localStorage by api.login()
        navigate('/');
      }
    } catch (err: any) {
      setError(err.message || 'Login failed. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="min-h-screen bg-background flex items-center justify-center p-4">
      <Card class="w-full max-w-md">
        <CardHeader class="text-center">
          <div class="w-16 h-16 bg-primary border-3 border-black shadow-brutal flex items-center justify-center mx-auto mb-4">
            <span class="text-4xl">🎨</span>
          </div>
          <CardTitle class="text-heading-1">Neo CRM</CardTitle>
          <p class="text-neutral-darkGray mt-2">Sign in to your account</p>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} class="space-y-4">
            <Show when={error()}>
              <div class="p-3 bg-red-100 border-3 border-red-500 text-red-700 text-sm font-bold">
                {error()}
              </div>
            </Show>

            <div>
              <label class="block font-heading font-bold uppercase text-sm mb-2">
                Email
              </label>
              <Input
                type="email"
                placeholder="you@example.com"
                value={email()}
                onInput={(e) => setEmail(e.currentTarget.value)}
                required
                disabled={loading()}
              />
            </div>

            <div>
              <label class="block font-heading font-bold uppercase text-sm mb-2">
                Password
              </label>
              <Input
                type="password"
                placeholder="••••••••"
                value={password()}
                onInput={(e) => setPassword(e.currentTarget.value)}
                required
                disabled={loading()}
              />
            </div>

            <Button type="submit" variant="primary" size="lg" fullWidth disabled={loading()}>
              <Show when={loading()} fallback="Sign In">
                <Spinner class="inline-block mr-2" />
                Signing In...
              </Show>
            </Button>

            <p class="text-center text-sm text-neutral-darkGray mt-4">
              Don't have an account? Contact your administrator.
            </p>
          </form>
        </CardContent>
      </Card>
    </div>
  );
};

export default Login;

