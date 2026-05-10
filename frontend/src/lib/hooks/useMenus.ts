import { createQuery } from '@tanstack/solid-query';
import { api } from '~/lib/api';

export const menuKeys = {
  myMenus: ['menus', 'my-menus'] as const,
};

/** Fetches the menu tree for the current user based on their permissions. */
export function useMyMenus() {
  return createQuery(() => ({
    queryKey: menuKeys.myMenus,
    queryFn: () => api.getMyMenus(),
    staleTime: 5 * 60 * 1000, // 5 min — menus change rarely
  }));
}
