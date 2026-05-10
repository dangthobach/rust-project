import { Component, createSignal, Show } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { api } from '~/lib/api';

const Login: Component = () => {
  const navigate = useNavigate();
  const [email, setEmail] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [error, setError] = createSignal('');
  const [loading, setLoading] = createSignal(false);

  const handleSubmit = async (e: SubmitEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);
    try {
      await api.login(email(), password());
      navigate('/', { replace: true });
    } catch (err: any) {
      setError(err.message || 'Email hoặc mật khẩu không đúng');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="min-h-screen flex items-center justify-center bg-background px-4">
      <div class="w-full max-w-sm">
        {/* Header */}
        <div class="mb-8 text-center">
          <h1 class="font-heading text-4xl font-black uppercase tracking-tight">
            Neo<span class="text-primary">CRM</span>
          </h1>
          <p class="mt-1 text-sm font-body text-neutral-gray">Industrial Ledger System</p>
        </div>

        {/* Card */}
        <div class="border-3 border-black bg-white p-8 shadow-brutal">
          <h2 class="font-heading text-xl font-bold uppercase mb-6">Đăng nhập</h2>

          <form onSubmit={handleSubmit} class="space-y-4">
            <div class="space-y-1">
              <label class="block text-xs font-bold uppercase tracking-wide">Email</label>
              <input
                type="email"
                class="input"
                placeholder="email@example.com"
                value={email()}
                onInput={(e) => setEmail(e.currentTarget.value)}
                autocomplete="email"
                required
              />
            </div>

            <div class="space-y-1">
              <label class="block text-xs font-bold uppercase tracking-wide">Mật khẩu</label>
              <input
                type="password"
                class="input"
                placeholder="••••••••"
                value={password()}
                onInput={(e) => setPassword(e.currentTarget.value)}
                autocomplete="current-password"
                required
              />
            </div>

            <Show when={error()}>
              <p class="border-2 border-red-500 bg-red-50 px-3 py-2 text-sm font-bold text-red-600">
                {error()}
              </p>
            </Show>

            <button
              type="submit"
              class="btn btn-primary w-full mt-2"
              disabled={loading()}
            >
              {loading() ? 'Đang đăng nhập...' : 'Đăng nhập'}
            </button>
          </form>
        </div>

        <p class="mt-4 text-center text-xs text-neutral-gray">
          Neo CRM v{import.meta.env.VITE_APP_VERSION || '1.0'}
        </p>
      </div>
    </div>
  );
};

export default Login;
