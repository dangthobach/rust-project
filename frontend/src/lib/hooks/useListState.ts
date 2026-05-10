import { createMemo } from 'solid-js';
import { useSearchParams } from '@solidjs/router';

/**
 * Syncs list search/filter/pagination state with URL search params.
 * Because state lives in the URL, pressing browser Back restores the exact
 * search/page the user was on — no additional state lifting required.
 *
 * Usage:
 *   const { state, update, reset, setPage } = useListState({
 *     page: '1', limit: '20', search: '', status: ''
 *   });
 */
export function useListState<T extends Record<string, string>>(defaults: T) {
  const [params, setParams] = useSearchParams<T>();

  const state = createMemo(() => {
    const out = {} as T;
    for (const key in defaults) {
      out[key] = ((params[key] ?? defaults[key]) || defaults[key]) as T[typeof key];
    }
    return out;
  });

  /** Merge partial updates into current search params (preserves unrelated params). */
  function update(patch: Partial<T> & { page?: string }) {
    // Changing any filter resets to page 1
    const hasFilterChange = Object.keys(patch).some((k) => k !== 'page');
    setParams(
      hasFilterChange && !('page' in patch)
        ? { ...patch, page: '1' }
        : (patch as any),
    );
  }

  /** Reset all managed params back to defaults. */
  function reset() {
    setParams({ ...defaults } as any);
  }

  function setPage(n: number) {
    setParams({ page: String(n) } as any);
  }

  /** Derive numeric page (1-based). */
  const page = createMemo(() => Math.max(1, parseInt(state().page ?? '1', 10) || 1));
  const limit = createMemo(() => Math.max(1, parseInt(state().limit ?? '20', 10) || 20));

  return { state, update, reset, setPage, page, limit };
}
