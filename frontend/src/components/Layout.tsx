import { Component, createSignal, Show, JSX } from 'solid-js';
import { A } from '@solidjs/router';
import { Badge } from './ui';
import ToastContainer from './ToastContainer';

const Layout: Component<{ children?: JSX.Element }> = (props) => {
  const [mobileMenuOpen, setMobileMenuOpen] = createSignal(false);
  const [adminMenuOpen, setAdminMenuOpen] = createSignal(false);
  const [currentTime, setCurrentTime] = createSignal(new Date().toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' }));
  const isAdmin = () => false; // Auth deferred (Keycloak PKCE)

  // Update time every minute
  setInterval(() => {
    setCurrentTime(new Date().toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' }));
  }, 60000);

  return (
    <div class="min-h-screen bg-background">
      {/* Enhanced Header with Neo-Brutalism */}
      <header class="border-b-8 border-black bg-gradient-to-r from-primary via-accent-yellow to-secondary sticky top-0 z-50 shadow-brutal-lg">
        {/* Top Info Bar */}
        <div class="bg-black text-primary border-b-4 border-black">
          <div class="container-brutal">
            <div class="flex items-center justify-between h-8 text-xs font-heading font-bold uppercase">
              <div class="flex items-center gap-3 md:gap-6">
                <span class="flex items-center gap-1 md:gap-2">
                  <span class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></span>
                  <span class="hidden sm:inline">System</span> Online
                </span>
                <span class="hidden md:inline">🕐 {currentTime()}</span>
                <span class="hidden lg:inline">📅 {new Date().toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' })}</span>
              </div>
              <div class="flex items-center gap-2 md:gap-4">
                <span class="flex items-center gap-1 md:gap-2">
                  <span class="text-accent-yellow">⚡</span>
                  <span class="text-primary hidden sm:inline">Fast Mode</span>
                </span>
                <Badge variant="success" class="text-xs border-2 border-primary px-1 md:px-2">
                  v2.0
                </Badge>
              </div>
            </div>
          </div>
        </div>

        {/* Main Navigation Bar */}
        <div class="container-brutal">
          <div class="flex items-center justify-between h-16 md:h-20">
            {/* Logo with Animation */}
            <A href="/" class="flex items-center gap-2 md:gap-3 no-underline group">
              <div class="relative">
                <div class="w-10 h-10 md:w-14 md:h-14 bg-black border-3 md:border-4 border-black shadow-brutal flex items-center justify-center transform group-hover:rotate-12 group-hover:-translate-y-1 transition-all duration-200">
                  <span class="text-xl md:text-3xl">🎨</span>
                </div>
                <div class="absolute -top-1 -right-1 w-3 h-3 md:w-4 md:h-4 bg-accent-yellow border-2 border-black rotate-45"></div>
              </div>
              <div class="hidden sm:block">
                <div class="text-lg md:text-2xl font-heading font-black uppercase text-black text-shadow-brutal leading-tight">
                  NEO CRM
                </div>
                <div class="text-[10px] md:text-xs font-body font-bold text-black uppercase tracking-wider hidden md:block">
                  Brutalist Design
                </div>
              </div>
            </A>

            {/* Desktop Navigation with Compact Style */}
            <nav class="hidden lg:flex items-center gap-1 xl:gap-2">
              <A
                href="/"
                class="px-3 xl:px-4 py-2 text-sm xl:text-base font-heading font-bold uppercase border-3 border-black bg-white text-black shadow-brutal transition-all duration-150 hover:-translate-x-0.5 hover:-translate-y-0.5 hover:shadow-brutal-lg hover:bg-primary"
                activeClass="bg-primary shadow-brutal-lg -translate-x-0.5 -translate-y-0.5"
              >
                <span class="flex items-center gap-1 xl:gap-2">
                  <span class="text-base xl:text-xl">📊</span>
                  <span class="hidden xl:inline">Dashboard</span>
                </span>
              </A>
              <A
                href="/clients"
                class="px-3 xl:px-4 py-2 text-sm xl:text-base font-heading font-bold uppercase border-3 border-black bg-white text-black shadow-brutal transition-all duration-150 hover:-translate-x-0.5 hover:-translate-y-0.5 hover:shadow-brutal-lg hover:bg-green-400"
                activeClass="bg-green-400 shadow-brutal-lg -translate-x-0.5 -translate-y-0.5"
              >
                <span class="flex items-center gap-1 xl:gap-2">
                  <span class="text-base xl:text-xl">👥</span>
                  <span class="hidden xl:inline">Clients</span>
                </span>
              </A>
              <A
                href="/tasks"
                class="px-3 xl:px-4 py-2 text-sm xl:text-base font-heading font-bold uppercase border-3 border-black bg-white text-black shadow-brutal transition-all duration-150 hover:-translate-x-0.5 hover:-translate-y-0.5 hover:shadow-brutal-lg hover:bg-blue-400"
                activeClass="bg-blue-400 shadow-brutal-lg -translate-x-0.5 -translate-y-0.5"
              >
                <span class="flex items-center gap-1 xl:gap-2">
                  <span class="text-base xl:text-xl">📋</span>
                  <span class="hidden xl:inline">Tasks</span>
                </span>
              </A>
              <A
                href="/notifications"
                class="relative px-3 xl:px-4 py-2 text-sm xl:text-base font-heading font-bold uppercase border-3 border-black bg-white text-black shadow-brutal transition-all duration-150 hover:-translate-x-0.5 hover:-translate-y-0.5 hover:shadow-brutal-lg hover:bg-accent-yellow"
                activeClass="bg-accent-yellow shadow-brutal-lg -translate-x-0.5 -translate-y-0.5"
              >
                <span class="flex items-center gap-1 xl:gap-2">
                  <span class="text-base xl:text-xl">🔔</span>
                  <span class="hidden xl:inline">Notifications</span>
                  <Badge variant="error" class="absolute -top-1 -right-1 w-5 h-5 flex items-center justify-center text-[10px] border-2">
                    12
                  </Badge>
                </span>
              </A>
              <A
                href="/reports"
                class="px-3 xl:px-4 py-2 text-sm xl:text-base font-heading font-bold uppercase border-3 border-black bg-white text-black shadow-brutal transition-all duration-150 hover:-translate-x-0.5 hover:-translate-y-0.5 hover:shadow-brutal-lg hover:bg-accent-yellow"
                activeClass="bg-accent-yellow shadow-brutal-lg -translate-x-0.5 -translate-y-0.5"
              >
                <span class="flex items-center gap-1 xl:gap-2">
                  <span class="text-base xl:text-xl">🧾</span>
                  <span class="hidden xl:inline">Reports</span>
                </span>
              </A>
              <A
                href="/files"
                class="px-3 xl:px-4 py-2 text-sm xl:text-base font-heading font-bold uppercase border-3 border-black bg-white text-black shadow-brutal transition-all duration-150 hover:-translate-x-0.5 hover:-translate-y-0.5 hover:shadow-brutal-lg hover:bg-secondary hover:text-white"
                activeClass="bg-secondary text-white shadow-brutal-lg -translate-x-0.5 -translate-y-0.5"
              >
                <span class="flex items-center gap-1 xl:gap-2">
                  <span class="text-base xl:text-xl">📁</span>
                  <span class="hidden xl:inline">Files</span>
                </span>
              </A>

              {/* User Profile Link */}
              <A
                href="/profile"
                class="px-3 xl:px-4 py-2 text-sm xl:text-base font-heading font-bold uppercase border-3 border-black bg-white text-black shadow-brutal transition-all duration-150 hover:-translate-x-0.5 hover:-translate-y-0.5 hover:shadow-brutal-lg hover:bg-purple-400"
                activeClass="bg-purple-400 shadow-brutal-lg -translate-x-0.5 -translate-y-0.5"
              >
                <span class="flex items-center gap-1 xl:gap-2">
                  <span class="text-base xl:text-xl">👤</span>
                  <span class="hidden xl:inline">Profile</span>
                </span>
              </A>

              {/* Admin Dropdown Menu */}
              <Show when={isAdmin()}>
                <div class="relative">
                  <button
                    onClick={() => setAdminMenuOpen(!adminMenuOpen())}
                    class="px-3 xl:px-4 py-2 text-sm xl:text-base font-heading font-bold uppercase border-3 border-black bg-red-400 text-black shadow-brutal transition-all duration-150 hover:-translate-x-0.5 hover:-translate-y-0.5 hover:shadow-brutal-lg"
                  >
                    <span class="flex items-center gap-1 xl:gap-2">
                      <span class="text-base xl:text-xl">⚙️</span>
                      <span class="hidden xl:inline">Admin</span>
                      <span class="text-xs">{adminMenuOpen() ? '▲' : '▼'}</span>
                    </span>
                  </button>

                  {/* Admin Dropdown */}
                  <Show when={adminMenuOpen()}>
                    <div class="absolute right-0 top-full mt-2 w-48 bg-white border-4 border-black shadow-brutal-lg z-50">
                      <A
                        href="/admin-dashboard"
                        class="block px-4 py-3 font-heading font-bold uppercase text-sm border-b-3 border-black hover:bg-red-400 transition-colors"
                        onClick={() => setAdminMenuOpen(false)}
                      >
                        <span class="flex items-center gap-2">
                          <span>📊</span>
                          <span>Dashboard</span>
                        </span>
                      </A>
                      <A
                        href="/analytics"
                        class="block px-4 py-3 font-heading font-bold uppercase text-sm border-b-3 border-black hover:bg-purple-400 transition-colors"
                        onClick={() => setAdminMenuOpen(false)}
                      >
                        <span class="flex items-center gap-2">
                          <span>📈</span>
                          <span>Analytics</span>
                        </span>
                      </A>
                      <A
                        href="/users-management"
                        class="block px-4 py-3 font-heading font-bold uppercase text-sm hover:bg-orange-400 transition-colors"
                        onClick={() => setAdminMenuOpen(false)}
                      >
                        <span class="flex items-center gap-2">
                          <span>👥</span>
                          <span>Users</span>
                        </span>
                      </A>
                    </div>
                  </Show>
                </div>
              </Show>
            </nav>

            {/* User Menu with Stats */}
            <div class="flex items-center gap-2 md:gap-3">
              {/* Quick Stats */}
              <div class="hidden xl:flex items-center gap-2 px-3 py-2 bg-white border-3 border-black shadow-brutal">
                <div class="flex items-center gap-1 text-xs font-bold">
                  <span>👥</span>
                  <span class="text-black">42</span>
                </div>
                <div class="w-px h-4 bg-black"></div>
                <div class="flex items-center gap-1 text-xs font-bold">
                  <span>✅</span>
                  <span class="text-black">156</span>
                </div>
              </div>

              {/* Profile Button */}
              <A
                href="/profile"
                class="hidden md:inline-flex gap-2 px-3 md:px-4 py-2 text-sm font-heading font-black uppercase border-3 border-black bg-secondary text-white shadow-brutal hover:-translate-y-1 transition-all"
              >
                <span class="text-base">👤</span>
                <span class="hidden lg:inline">Profile</span>
              </A>

              {/* Mobile Menu Button */}
              <button
                class="lg:hidden px-3 md:px-4 py-2 font-heading font-black text-xl border-3 border-black bg-white shadow-brutal hover:-translate-y-1 transition-all"
                onClick={() => setMobileMenuOpen(!mobileMenuOpen())}
              >
                {mobileMenuOpen() ? '✕' : '☰'}
              </button>
            </div>
          </div>

          {/* Mobile Menu */}
          <Show when={mobileMenuOpen()}>
            <nav class="lg:hidden py-4 border-t-4 border-black bg-white">
              <div class="flex flex-col gap-2">
                <A
                  href="/"
                  class="px-4 py-3 font-heading font-bold uppercase text-sm border-3 border-black bg-white text-black shadow-brutal hover:-translate-y-1 transition-all"
                  activeClass="bg-primary shadow-brutal-lg"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  <span class="flex items-center gap-3">
                    <span class="text-xl">📊</span>
                    <span>Dashboard</span>
                  </span>
                </A>
                <A
                  href="/clients"
                  class="px-4 py-3 font-heading font-bold uppercase text-sm border-3 border-black bg-white text-black shadow-brutal hover:-translate-y-1 transition-all"
                  activeClass="bg-green-400 shadow-brutal-lg"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  <span class="flex items-center gap-3">
                    <span class="text-xl">👥</span>
                    <span>Clients</span>
                  </span>
                </A>
                <A
                  href="/tasks"
                  class="px-4 py-3 font-heading font-bold uppercase text-sm border-3 border-black bg-white text-black shadow-brutal hover:-translate-y-1 transition-all"
                  activeClass="bg-blue-400 shadow-brutal-lg"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  <span class="flex items-center gap-3">
                    <span class="text-xl">📋</span>
                    <span>Tasks</span>
                  </span>
                </A>
                <A
                  href="/notifications"
                  class="px-4 py-3 font-heading font-bold uppercase text-sm border-3 border-black bg-white text-black shadow-brutal hover:-translate-y-1 transition-all"
                  activeClass="bg-accent-yellow shadow-brutal-lg"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  <span class="flex items-center gap-3">
                    <span class="text-xl">🔔</span>
                    <span>Notifications</span>
                    <Badge variant="error" class="ml-auto text-xs">12</Badge>
                  </span>
                </A>
                <A
                  href="/files"
                  class="px-4 py-3 font-heading font-bold uppercase text-sm border-3 border-black bg-white text-black shadow-brutal hover:-translate-y-1 transition-all"
                  activeClass="bg-secondary text-white shadow-brutal-lg"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  <span class="flex items-center gap-3">
                    <span class="text-xl">📁</span>
                    <span>Files</span>
                  </span>
                </A>
                <A
                  href="/profile"
                  class="px-4 py-3 font-heading font-bold uppercase text-sm border-3 border-black bg-white text-black shadow-brutal hover:-translate-y-1 transition-all"
                  activeClass="bg-purple-400 shadow-brutal-lg"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  <span class="flex items-center gap-3">
                    <span class="text-xl">👤</span>
                    <span>Profile</span>
                  </span>
                </A>
                <Show when={isAdmin()}>
                  <div class="border-t-3 border-black pt-2 mt-2">
                    <div class="px-4 py-2 text-xs font-heading font-black uppercase text-neutral-darkGray">
                      Admin Section
                    </div>
                    <A
                      href="/admin-dashboard"
                      class="px-4 py-3 font-heading font-bold uppercase text-sm border-3 border-black bg-white text-black shadow-brutal hover:-translate-y-1 transition-all"
                      activeClass="bg-red-400 shadow-brutal-lg"
                      onClick={() => setMobileMenuOpen(false)}
                    >
                      <span class="flex items-center gap-3">
                        <span class="text-xl">📊</span>
                        <span>Admin Dashboard</span>
                      </span>
                    </A>
                    <A
                      href="/analytics"
                      class="px-4 py-3 font-heading font-bold uppercase text-sm border-3 border-black bg-white text-black shadow-brutal hover:-translate-y-1 transition-all"
                      activeClass="bg-purple-400 shadow-brutal-lg"
                      onClick={() => setMobileMenuOpen(false)}
                    >
                      <span class="flex items-center gap-3">
                        <span class="text-xl">📈</span>
                        <span>Analytics</span>
                      </span>
                    </A>
                    <A
                      href="/users-management"
                      class="px-4 py-3 font-heading font-bold uppercase text-sm border-3 border-black bg-white text-black shadow-brutal hover:-translate-y-1 transition-all"
                      activeClass="bg-orange-400 shadow-brutal-lg"
                      onClick={() => setMobileMenuOpen(false)}
                    >
                      <span class="flex items-center gap-3">
                        <span class="text-xl">👥</span>
                        <span>User Management</span>
                      </span>
                    </A>
                  </div>
                </Show>
              </div>
            </nav>
          </Show>
        </div>
      </header>

      {/* Main Content */}
      <main class="container-brutal py-6 md:py-8">
        {props.children}
      </main>

      {/* Enhanced Footer */}
      <footer class="border-t-8 border-black bg-black text-primary py-6 md:py-8 mt-16">
        <div class="container-brutal">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4 md:gap-6 mb-4 md:mb-6">
            <div>
              <h3 class="font-heading font-black uppercase text-base md:text-lg mb-2 md:mb-3 text-accent-yellow">
                Neo CRM System
              </h3>
              <p class="text-xs md:text-sm text-neutral-concrete">
                A brutally efficient customer relationship management system built with modern tech.
              </p>
            </div>
            <div>
              <h3 class="font-heading font-black uppercase text-base md:text-lg mb-2 md:mb-3 text-accent-yellow">
                Quick Links
              </h3>
              <ul class="space-y-1 md:space-y-2 text-xs md:text-sm">
                <li><A href="/" class="hover:text-accent-yellow transition-colors">Dashboard</A></li>
                <li><A href="/notifications" class="hover:text-accent-yellow transition-colors">Notifications</A></li>
                <li><A href="/files" class="hover:text-accent-yellow transition-colors">Files</A></li>
              </ul>
            </div>
            <div>
              <h3 class="font-heading font-black uppercase text-base md:text-lg mb-2 md:mb-3 text-accent-yellow">
                Tech Stack
              </h3>
              <div class="flex flex-wrap gap-2">
                <Badge variant="primary" class="text-xs border-primary">Solid.js</Badge>
                <Badge variant="secondary" class="text-xs border-secondary">Rust</Badge>
                <Badge variant="success" class="text-xs border-green-400">SQLite</Badge>
              </div>
            </div>
          </div>
          <div class="border-t-4 border-primary pt-4 md:pt-6 text-center">
            <p class="font-heading font-bold uppercase text-xs md:text-sm">
              Neo-Brutalist CRM System © 2025 - Designed with ⚡ by Brutal Devs
            </p>
            <p class="mt-2 text-[10px] md:text-xs text-neutral-concrete">
              Made with Solid.js, Rust, and a lot of bold borders
            </p>
          </div>
        </div>
      </footer>
      <ToastContainer />
    </div>
  );
};

export default Layout;
