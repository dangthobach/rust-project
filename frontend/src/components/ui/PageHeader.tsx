import { Component, JSX, For, Show } from 'solid-js';
import { A } from '@solidjs/router';

export interface Breadcrumb {
  label: string;
  href?: string;
}

interface PageHeaderProps {
  title: string;
  description?: string;
  breadcrumbs?: Breadcrumb[];
  actions?: JSX.Element;
}

export const PageHeader: Component<PageHeaderProps> = (props) => {
  return (
    <div class="mb-6">
      <Show when={props.breadcrumbs && props.breadcrumbs.length > 0}>
        <nav class="mb-2 flex items-center gap-1 font-mono text-[10px] uppercase tracking-widest text-neutral-darkGray">
          <For each={props.breadcrumbs}>
            {(crumb, i) => (
              <>
                <Show
                  when={crumb.href}
                  fallback={
                    <span
                      class={
                        i() === (props.breadcrumbs?.length ?? 0) - 1
                          ? 'font-black text-black'
                          : ''
                      }
                    >
                      {crumb.label}
                    </span>
                  }
                >
                  <A
                    href={crumb.href!}
                    class="underline decoration-dotted underline-offset-2 hover:text-black no-underline"
                  >
                    {crumb.label}
                  </A>
                </Show>
                <Show when={i() < (props.breadcrumbs?.length ?? 0) - 1}>
                  <span>/</span>
                </Show>
              </>
            )}
          </For>
        </nav>
      </Show>

      <div class="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 class="font-heading text-2xl font-black uppercase tracking-tight text-shadow-brutal sm:text-heading-1">
            {props.title}
          </h1>
          <Show when={props.description}>
            <p class="mt-1 font-mono text-xs font-semibold uppercase tracking-wide text-neutral-darkGray">
              {props.description}
            </p>
          </Show>
        </div>
        <Show when={props.actions}>
          <div class="flex flex-wrap items-center gap-2">{props.actions}</div>
        </Show>
      </div>
    </div>
  );
};
