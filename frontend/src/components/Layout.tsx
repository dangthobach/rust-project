import { Component, createSignal, Show, JSX } from 'solid-js';
import { A, useLocation } from '@solidjs/router';
import ToastContainer from './ToastContainer';

const IconDashboard = () => (
  <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" stroke-width="2.25" viewBox="0 0 24 24" aria-hidden="true">
    <path stroke-linecap="round" stroke-linejoin="round" d="M4 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM14 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1V5zM4 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1v-4zM14 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
  </svg>
);

const IconClients = () => (
  <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" stroke-width="2.25" viewBox="0 0 24 24" aria-hidden="true">
    <path stroke-linecap="round" stroke-linejoin="round" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
  </svg>
);

const IconTasks = () => (
  <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" stroke-width="2.25" viewBox="0 0 24 24" aria-hidden="true">
    <path stroke-linecap="round" stroke-linejoin="round" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
  </svg>
);

const IconReports = () => (
  <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" stroke-width="2.25" viewBox="0 0 24 24" aria-hidden="true">
    <path stroke-linecap="round" stroke-linejoin="round" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
  </svg>
);

const IconSettings = () => (
  <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" stroke-width="2.25" viewBox="0 0 24 24" aria-hidden="true">
    <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
    <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
  </svg>
);

const IconSupport = () => (
  <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" stroke-width="2.25" viewBox="0 0 24 24" aria-hidden="true">
    <path stroke-linecap="round" stroke-linejoin="round" d="M18.364 5.636l-3.536 3.536m0 5.656l3.536 3.536M9.172 9.172L5.636 5.636m3.536 9.192l-3.536 3.536M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-5 0a4 4 0 11-8 0 4 4 0 018 0z" />
  </svg>
);

function pathActive(pathname: string, href: string): boolean {
  if (href === '/') return pathname === '/';
  return pathname === href || pathname.startsWith(`${href}/`);
}

const Layout: Component<{ children?: JSX.Element }> = (props) => {
  const location = useLocation();
  const [mobileOpen, setMobileOpen] = createSignal(false);

  const navClass = (href: string) => {
    const active = pathActive(location.pathname, href);
    return [
      'flex items-center gap-3 px-3 py-2.5 font-heading font-bold text-sm uppercase tracking-wide border-3 border-black transition-all duration-150 no-underline',
      active
        ? 'bg-ledger-lime text-black shadow-brutal-sm -translate-x-0.5 -translate-y-0.5'
        : 'bg-white text-black shadow-brutal-sm hover:-translate-x-0.5 hover:-translate-y-0.5 hover:bg-ledger-pale',
    ].join(' ');
  };

  const SidebarInner = () => (
    <>
      <div class="mb-6">
        <A href="/" class="block no-underline group" onClick={() => setMobileOpen(false)}>
          <div class="font-heading font-black text-lg uppercase tracking-tight text-black leading-tight">
            LEDGER CRM
          </div>
          <div class="font-heading text-[10px] font-bold uppercase tracking-[0.2em] text-neutral-darkGray mt-1">
            Industrial v1.0
          </div>
        </A>
      </div>

      <A
        href="/tasks/new"
        class="mb-8 flex items-center justify-center gap-2 border-3 border-black bg-ledger-lime px-4 py-3 font-heading font-black text-sm uppercase tracking-wide text-black shadow-brutal no-underline transition-all hover:-translate-x-0.5 hover:-translate-y-0.5 hover:shadow-brutal-lg active:translate-x-0 active:translate-y-0"
        onClick={() => setMobileOpen(false)}
      >
        <span class="text-lg leading-none">+</span>
        <span>New Task</span>
      </A>

      <nav class="flex flex-col gap-2">
        <A href="/" class={navClass('/')} onClick={() => setMobileOpen(false)}>
          <IconDashboard />
          <span>Dashboard</span>
        </A>
        <A href="/clients" class={navClass('/clients')} onClick={() => setMobileOpen(false)}>
          <IconClients />
          <span>Clients</span>
        </A>
        <A href="/tasks" class={navClass('/tasks')} onClick={() => setMobileOpen(false)}>
          <IconTasks />
          <span>Tasks</span>
        </A>
        <A href="/reports" class={navClass('/reports')} onClick={() => setMobileOpen(false)}>
          <IconReports />
          <span>Reports</span>
        </A>
      </nav>

      <div class="mt-4 flex flex-col gap-2 border-t-3 border-black pt-4">
        <A
          href="/notifications"
          class="flex items-center gap-3 px-3 py-2 font-heading font-bold text-xs uppercase text-black no-underline hover:bg-ledger-pale border-2 border-transparent hover:border-black"
          onClick={() => setMobileOpen(false)}
        >
          <span class="text-base" aria-hidden="true">
            🔔
          </span>
          <span>Alerts</span>
        </A>
        <A
          href="/files"
          class="flex items-center gap-3 px-3 py-2 font-heading font-bold text-xs uppercase text-black no-underline hover:bg-ledger-pale border-2 border-transparent hover:border-black"
          onClick={() => setMobileOpen(false)}
        >
          <span class="text-base" aria-hidden="true">
            📁
          </span>
          <span>Files</span>
        </A>
      </div>

      <div class="flex-1 min-h-4" />

      <div class="border-t-3 border-black pt-4 flex flex-col gap-2">
        <A
          href="/profile"
          class="flex items-center gap-3 px-3 py-2 font-heading font-bold text-xs uppercase text-black no-underline border-3 border-transparent hover:border-black hover:bg-white hover:shadow-brutal-sm"
          onClick={() => setMobileOpen(false)}
        >
          <IconSettings />
          <span>Settings</span>
        </A>
        <a
          href="https://github.com"
          target="_blank"
          rel="noopener noreferrer"
          class="flex items-center gap-3 px-3 py-2 font-heading font-bold text-xs uppercase text-black no-underline border-3 border-transparent hover:border-black hover:bg-white hover:shadow-brutal-sm"
        >
          <IconSupport />
          <span>Support</span>
        </a>
      </div>
    </>
  );

  return (
    <div class="min-h-screen flex flex-col bg-background font-body text-black">
      <div class="flex flex-1 min-h-0">
        {/* Desktop sidebar */}
        <aside class="hidden lg:flex w-[260px] shrink-0 flex-col border-r-[3px] border-black bg-background px-4 py-6">
          <SidebarInner />
        </aside>

        {/* Mobile drawer */}
        <Show when={mobileOpen()}>
          <div
            class="fixed inset-0 z-50 bg-black/40 lg:hidden"
            role="presentation"
            onClick={() => setMobileOpen(false)}
          />
          <aside class="fixed left-0 top-0 z-50 h-full w-[280px] border-r-[3px] border-black bg-background px-4 py-6 shadow-brutal-lg overflow-y-auto lg:hidden">
            <button
              type="button"
              class="mb-4 w-full border-3 border-black bg-white py-2 font-heading font-bold uppercase text-sm shadow-brutal-sm"
              onClick={() => setMobileOpen(false)}
            >
              Close
            </button>
            <SidebarInner />
          </aside>
        </Show>

        <div class="flex min-w-0 flex-1 flex-col">
          <div class="flex items-center gap-3 border-b-[3px] border-black bg-background px-4 py-3 lg:hidden">
            <button
              type="button"
              class="border-3 border-black bg-white px-3 py-2 font-heading font-black shadow-brutal-sm"
              onClick={() => setMobileOpen(true)}
              aria-label="Open menu"
            >
              ☰
            </button>
            <span class="font-heading font-black uppercase text-sm tracking-wide">Ledger CRM</span>
          </div>

          <main class="flex-1 px-4 py-6 sm:px-6 lg:px-8 lg:py-8 max-w-[1600px] w-full mx-auto">{props.children}</main>
        </div>
      </div>

      <footer class="border-t-[3px] border-black bg-black text-white">
        <div class="mx-auto flex max-w-[1600px] flex-col gap-3 px-4 py-4 sm:flex-row sm:items-center sm:justify-between sm:px-8">
          <div class="font-body text-xs sm:text-sm">
            <span class="text-white/90">© 2026 Industrial Ledger CRM</span>
            <span class="mx-2 text-white/40">·</span>
            <span class="font-heading font-bold uppercase text-ledger-lime">System status: optimal</span>
          </div>
          <div class="flex flex-wrap gap-x-4 gap-y-1 font-heading text-[10px] font-bold uppercase tracking-wider sm:text-xs">
            <a href="/files" class="text-white/90 hover:text-ledger-lime no-underline">
              Tech stack
            </a>
            <span class="text-white/30">|</span>
            <a href="/profile" class="text-white/90 hover:text-ledger-lime no-underline">
              Privacy policy
            </a>
            <span class="text-white/30">|</span>
            <a href="/reports" class="text-white/90 hover:text-ledger-lime no-underline">
              API docs
            </a>
          </div>
        </div>
      </footer>

      <ToastContainer />
    </div>
  );
};

export default Layout;
