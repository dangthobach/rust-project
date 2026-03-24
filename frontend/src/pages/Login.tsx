import { Component, createSignal, Show } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent, Input, Button, Spinner } from '~/components/ui';
import { useLogin } from '~/lib/hooks';

const Login: Component = () => {
  const [email, setEmail] = createSignal('test@example.com');
  const [password, setPassword] = createSignal('test123');
  
  const login = useLogin();

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    
    if (!email() || !password()) {
      return;
    }

    login.mutate({ email: email(), password: password() });
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
            <Show when={login.isError}>
              <div class="p-3 bg-red-100 border-3 border-red-500 text-red-700 text-sm font-bold">
                {login.error?.message || 'Login failed. Please try again.'}
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
                disabled={login.isPending}
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
                disabled={login.isPending}
              />
            </div>

            <Button 
              type="submit" 
              variant="primary" 
              size="lg" 
              fullWidth 
              disabled={login.isPending}
            >
              <Show when={login.isPending} fallback="Sign In">
                <Spinner class="inline-block mr-2" />
                Signing In...
              </Show>
            </Button>

            <p class="text-center text-sm text-neutral-darkGray mt-4">
              Demo: test@example.com / test123<br />
              Don't have an account? Contact your administrator.
            </p>
          </form>
        </CardContent>
      </Card>
    </div>
  );
};

export default Login;

