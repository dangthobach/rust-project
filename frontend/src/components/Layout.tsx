import { Component, createSignal, Show, JSX } from 'solid-js';
import { A } from '@solidjs/router';
import { Button } from './ui';

const Layout: Component<{ children?: JSX.Element }> = (props) => {
  const [mobileMenuOpen, setMobileMenuOpen] = createSignal(false);

  return (
    <div class="min-h-screen bg-background">
      {/* Header */}
      <header class="border-b-5 border-black bg-white sticky top-0 z-40">
        <div class="container-brutal">
          <div class="flex items-center justify-between h-16">
            {/* Logo */}
            <A href="/" class="flex items-center gap-2 no-underline">
              <div class="w-10 h-10 bg-primary border-3 border-black shadow-brutal flex items-center justify-center">
                <span class="text-2xl font-bold">🎨</span>
              </div>
              <span class="text-heading-3 font-heading font-black uppercase hidden md:inline">
                Neo CRM
              </span>
            </A>

            {/* Desktop Navigation */}
            <nav class="hidden md:flex items-center gap-4">
              <A
                href="/"
                class="btn btn-ghost"
                activeClass="bg-primary text-black"
              >
                📊 Dashboard
              </A>
              <A
                href="/notifications"
                class="btn btn-ghost"
                activeClass="bg-primary text-black"
              >
                🔔 Notifications
              </A>
              <A
                href="/files"
                class="btn btn-ghost"
                activeClass="bg-primary text-black"
              >
                📁 Files
              </A>
            </nav>

            {/* User Menu */}
            <div class="flex items-center gap-2">
              <Button variant="secondary" size="sm" class="hidden md:inline-flex">
                👤 Profile
              </Button>
              
              {/* Mobile Menu Button */}
              <button
                class="md:hidden btn btn-ghost"
                onClick={() => setMobileMenuOpen(!mobileMenuOpen())}
              >
                {mobileMenuOpen() ? '✕' : '☰'}
              </button>
            </div>
          </div>

          {/* Mobile Menu */}
          <Show when={mobileMenuOpen()}>
            <nav class="md:hidden py-4 border-t-3 border-black">
              <div class="flex flex-col gap-2">
                <A
                  href="/"
                  class="btn btn-ghost justify-start"
                  activeClass="bg-primary text-black"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  📊 Dashboard
                </A>
                <A
                  href="/notifications"
                  class="btn btn-ghost justify-start"
                  activeClass="bg-primary text-black"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  🔔 Notifications
                </A>
                <A
                  href="/files"
                  class="btn btn-ghost justify-start"
                  activeClass="bg-primary text-black"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  📁 Files
                </A>
                <Button variant="secondary" size="sm" class="justify-start">
                  👤 Profile
                </Button>
              </div>
            </nav>
          </Show>
        </div>
      </header>

      {/* Main Content */}
      <main class="container-brutal py-8">
        {props.children}
      </main>

      {/* Footer */}
      <footer class="border-t-5 border-black bg-neutral-concrete py-6 mt-16">
        <div class="container-brutal text-center text-sm text-neutral-darkGray">
          <p class="font-heading font-bold uppercase">
            Neo-Brutalist CRM System © 2025
          </p>
          <p class="mt-1">Built with Solid.js & Rust</p>
        </div>
      </footer>
    </div>
  );
};

export default Layout;

