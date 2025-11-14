import { Component, createSignal } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { Card, CardHeader, CardTitle, CardContent, Input, Button } from '~/components/ui';

const Login: Component = () => {
  const navigate = useNavigate();
  const [email, setEmail] = createSignal('');
  const [password, setPassword] = createSignal('');

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    // Mock login - redirect to dashboard
    navigate('/');
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
              />
            </div>

            <Button type="submit" variant="primary" size="lg" fullWidth>
              Sign In
            </Button>

            <p class="text-center text-sm text-neutral-darkGray mt-4">
              Demo: Use any email/password to login
            </p>
          </form>
        </CardContent>
      </Card>
    </div>
  );
};

export default Login;

