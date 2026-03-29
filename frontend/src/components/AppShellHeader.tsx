import { Component, createSignal } from 'solid-js';
import { A, useLocation, useNavigate } from '@solidjs/router';
import { useCurrentUser } from '~/lib/hooks';

function pathActive(pathname: string, href: string): boolean {
  if (href === '/') return pathname === '/';
  return pathname === href || pathname.startsWith(`${href}/`);
}

const IconSearch = () => (
  <svg class="h-5 w-5 shrink-0" fill="none" stroke="currentColor" stroke-width="2.25" viewBox="0 0 24 24" aria-hidden="true">
    <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" />
  </svg>
);

/** Desktop top strip: brand, inline nav, global search, utilities — matches Industrial Ledger mockups. */
const AppShellHeader: Component = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const [query, setQuery] = createSignal('');
  const me = useCurrentUser();

  const displayName = () => {
    const u = me.data as { full_name?: string; username?: string; email?: string } | undefined;
    return u?.full_name || u?.username || u?.email || 'User';
  };

  const initials = () => {
    const n = displayName();
    const parts = n.split(/\s+/).filter(Boolean);
    if (parts.length >= 2) return (parts[0][0] + parts[1][0]).toUpperCase();
    return n.slice(0, 2).toUpperCase() || 'U';
  };

  const linkClass = (href: string) => {
    const active = pathActive(location.pathname, href);
    return [
      'font-heading text-xs font-black uppercase tracking-wide no-underline text-black',
      active ? 'underline decoration-2 underline-offset-4 decoration-black' : 'hover:underline',
    ].join(' ');
  };

  const submitSearch = () => {
    const q = query().trim();
    if (q.length >= 2) navigate(`/search?q=${encodeURIComponent(q)}`);
    else if (q) navigate(`/tasks?q=${encodeURIComponent(q)}`);
    else navigate('/search');
  };

  return (
    <header class="hidden border-b-[3px] border-black bg-background lg:block">
      <div class="mx-auto flex max-w-[1600px] flex-wrap items-center gap-4 px-6 py-3">
        <div class="flex min-w-0 shrink-0 items-center gap-3">
          <A href="/" class="font-heading text-sm font-black uppercase tracking-tight text-black no-underline sm:text-base">
            Industrial Ledger
          </A>
        </div>

        <nav class="flex flex-wrap items-center gap-x-5 gap-y-2" aria-label="Primary">
          <A href="/" class={linkClass('/')}>
            Dashboard
          </A>
          <A href="/clients" class={linkClass('/clients')}>
            Clients
          </A>
          <A href="/tasks" class={linkClass('/tasks')}>
            Tasks
          </A>
          <A href="/reports" class={linkClass('/reports')}>
            Reports
          </A>
        </nav>

        <div class="relative min-w-[200px] flex-1 basis-[220px]">
          <span class="pointer-events-none absolute left-3 top-1/2 z-10 -translate-y-1/2 text-neutral-darkGray" aria-hidden="true">
            <IconSearch />
          </span>
          <input
            type="search"
            placeholder="SEARCH DATA..."
            value={query()}
            onInput={(e) => setQuery(e.currentTarget.value)}
            onKeyDown={(e) => e.key === 'Enter' && submitSearch()}
            class="w-full border-[3px] border-black bg-white py-2.5 pl-11 pr-3 font-mono text-xs font-semibold uppercase tracking-wide placeholder:text-neutral-gray focus:outline-none focus:shadow-brutal-sm"
            aria-label="Search data"
          />
        </div>

        <div class="ml-auto flex shrink-0 items-center gap-2">
          <A
            href="/notifications"
            class="inline-flex h-10 w-10 items-center justify-center border-[3px] border-black bg-white text-lg shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5"
            aria-label="Notifications"
          >
            🔔
          </A>
          <A
            href="/profile"
            class="inline-flex h-10 w-10 items-center justify-center border-[3px] border-black bg-white text-lg shadow-brutal-sm transition-all hover:-translate-x-0.5 hover:-translate-y-0.5"
            aria-label="Settings"
          >
            ⚙️
          </A>
          <A
            href="/profile"
            class="flex h-10 w-10 shrink-0 items-center justify-center border-[3px] border-black bg-ledger-lime font-heading text-xs font-black uppercase shadow-brutal-sm"
            title={displayName()}
          >
            {initials()}
          </A>
        </div>
      </div>
    </header>
  );
};

export default AppShellHeader;
