import { Component, createSignal, Show, JSX } from 'solid-js';
import { A } from '@solidjs/router';
import { Button, Badge } from './ui';

const Layout: Component<{ children?: JSX.Element }> = (props) => {
  const [mobileMenuOpen, setMobileMenuOpen] = createSignal(false);
  const [currentTime, setCurrentTime] = createSignal(new Date().toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' }));

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
            <div class="flex items-center justify-between h-10 text-xs font-heading font-bold uppercase">
              <div class="flex items-center gap-6">
                <span class="flex items-center gap-2">
                  <span class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></span>
                  System Online
                </span>
                <span class="hidden sm:inline">🕐 {currentTime()}</span>
                <span class="hidden md:inline">📅 {new Date().toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' })}</span>
              </div>
              <div class="flex items-center gap-4">
                <span class="hidden sm:flex items-center gap-2">
                  <span class="text-accent-yellow">⚡</span>
                  <span class="text-primary">Fast Mode</span>
                </span>
                <Badge variant="success" class="hidden md:inline-flex border-2 border-primary">
                  v2.0
                </Badge>
              </div>
            </div>
          </div>
        </div>

        {/* Main Navigation Bar */}
        <div class="container-brutal">
          <div class="flex items-center justify-between h-20">
            {/* Logo with Animation */}
            <A href="/" class="flex items-center gap-3 no-underline group">
              <div class="relative">
                <div class="w-14 h-14 bg-black border-4 border-black shadow-brutal-lg flex items-center justify-center transform group-hover:rotate-12 group-hover:-translate-y-1 transition-all duration-200">
                  <span class="text-3xl">🎨</span>
                </div>
                <div class="absolute -top-1 -right-1 w-4 h-4 bg-accent-yellow border-2 border-black rotate-45"></div>
              </div>
              <div class="hidden md:block">
                <div class="text-2xl font-heading font-black uppercase text-black text-shadow-brutal leading-tight">
                  NEO CRM
                </div>
                <div class="text-xs font-body font-bold text-black uppercase tracking-wider">
                  Brutalist Design System
                </div>
              </div>
            </A>

            {/* Desktop Navigation with Enhanced Style */}
            <nav class="hidden lg:flex items-center gap-2">
              <A
                href="/"
                class="px-6 py-3 font-heading font-bold uppercase border-4 border-black bg-white text-black shadow-brutal transition-all duration-150 hover:-translate-x-1 hover:-translate-y-1 hover:shadow-brutal-lg hover:bg-primary"
                activeClass="bg-primary shadow-brutal-lg -translate-x-1 -translate-y-1"
              >
                <span class="flex items-center gap-2">
                  <span class="text-xl">📊</span>
                  <span>Dashboard</span>
                </span>
              </A>
              <A
                href="/notifications"
                class="relative px-6 py-3 font-heading font-bold uppercase border-4 border-black bg-white text-black shadow-brutal transition-all duration-150 hover:-translate-x-1 hover:-translate-y-1 hover:shadow-brutal-lg hover:bg-accent-yellow"
                activeClass="bg-accent-yellow shadow-brutal-lg -translate-x-1 -translate-y-1"
              >
                <span class="flex items-center gap-2">
                  <span class="text-xl">🔔</span>
                  <span>Notifications</span>
                  <Badge variant="error" class="absolute -top-2 -right-2 w-6 h-6 flex items-center justify-center text-xs border-2">
                    12
                  </Badge>
                </span>
              </A>
              <A
                href="/files"
                class="px-6 py-3 font-heading font-bold uppercase border-4 border-black bg-white text-black shadow-brutal transition-all duration-150 hover:-translate-x-1 hover:-translate-y-1 hover:shadow-brutal-lg hover:bg-secondary hover:text-white"
                activeClass="bg-secondary text-white shadow-brutal-lg -translate-x-1 -translate-y-1"
              >
                <span class="flex items-center gap-2">
                  <span class="text-xl">📁</span>
                  <span>Files</span>
                </span>
              </A>
            </nav>

            {/* User Menu with Stats */}
            <div class="flex items-center gap-3">
              {/* Quick Stats */}
              <div class="hidden xl:flex items-center gap-2 px-4 py-2 bg-white border-4 border-black shadow-brutal">
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
              <Button variant="secondary" size="md" class="hidden md:inline-flex gap-2 shadow-brutal-lg">
                <span class="text-lg">👤</span>
                <span class="font-black">Profile</span>
              </Button>
              
              {/* Mobile Menu Button */}
              <button
                class="lg:hidden px-4 py-3 font-heading font-black text-2xl border-4 border-black bg-white shadow-brutal hover:-translate-y-1 transition-all"
                onClick={() => setMobileMenuOpen(!mobileMenuOpen())}
              >
                {mobileMenuOpen() ? '✕' : '☰'}
              </button>
            </div>
          </div>

          {/* Mobile Menu */}
          <Show when={mobileMenuOpen()}>
            <nav class="lg:hidden py-6 border-t-5 border-black bg-white">
              <div class="flex flex-col gap-3">
                <A
                  href="/"
                  class="px-6 py-4 font-heading font-bold uppercase border-4 border-black bg-white text-black shadow-brutal hover:-translate-y-1 transition-all"
                  activeClass="bg-primary shadow-brutal-lg"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  <span class="flex items-center gap-3">
                    <span class="text-2xl">📊</span>
                    <span>Dashboard</span>
                  </span>
                </A>
                <A
                  href="/notifications"
                  class="px-6 py-4 font-heading font-bold uppercase border-4 border-black bg-white text-black shadow-brutal hover:-translate-y-1 transition-all"
                  activeClass="bg-accent-yellow shadow-brutal-lg"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  <span class="flex items-center gap-3">
                    <span class="text-2xl">🔔</span>
                    <span>Notifications</span>
                    <Badge variant="error" class="ml-auto">12</Badge>
                  </span>
                </A>
                <A
                  href="/files"
                  class="px-6 py-4 font-heading font-bold uppercase border-4 border-black bg-white text-black shadow-brutal hover:-translate-y-1 transition-all"
                  activeClass="bg-secondary text-white shadow-brutal-lg"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  <span class="flex items-center gap-3">
                    <span class="text-2xl">📁</span>
                    <span>Files</span>
                  </span>
                </A>
                <div class="px-6 py-3 mt-3 border-t-4 border-black">
                  <Button variant="secondary" size="md" class="w-full justify-center gap-2">
                    <span class="text-lg">👤</span>
                    <span class="font-black">Profile</span>
                  </Button>
                </div>
              </div>
            </nav>
          </Show>
        </div>
      </header>

      {/* Main Content */}
      <main class="container-brutal py-8">
        {props.children}
      </main>

      {/* Enhanced Footer */}
      <footer class="border-t-8 border-black bg-black text-primary py-8 mt-16">
        <div class="container-brutal">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
            <div>
              <h3 class="font-heading font-black uppercase text-lg mb-3 text-accent-yellow">
                Neo CRM System
              </h3>
              <p class="text-sm text-neutral-concrete">
                A brutally efficient customer relationship management system built with modern tech.
              </p>
            </div>
            <div>
              <h3 class="font-heading font-black uppercase text-lg mb-3 text-accent-yellow">
                Quick Links
              </h3>
              <ul class="space-y-2 text-sm">
                <li><A href="/" class="hover:text-accent-yellow transition-colors">Dashboard</A></li>
                <li><A href="/notifications" class="hover:text-accent-yellow transition-colors">Notifications</A></li>
                <li><A href="/files" class="hover:text-accent-yellow transition-colors">Files</A></li>
              </ul>
            </div>
            <div>
              <h3 class="font-heading font-black uppercase text-lg mb-3 text-accent-yellow">
                Tech Stack
              </h3>
              <div class="flex flex-wrap gap-2">
                <Badge variant="primary" class="border-primary">Solid.js</Badge>
                <Badge variant="secondary" class="border-secondary">Rust</Badge>
                <Badge variant="success" class="border-green-400">PostgreSQL</Badge>
              </div>
            </div>
          </div>
          <div class="border-t-4 border-primary pt-6 text-center">
            <p class="font-heading font-bold uppercase text-sm">
              Neo-Brutalist CRM System © 2025 - Designed with ⚡ by Brutal Devs
            </p>
            <p class="mt-2 text-xs text-neutral-concrete">
              Made with Solid.js, Rust, and a lot of bold borders
            </p>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default Layout;

