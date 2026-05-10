// Re-export from feature module for backward compatibility with existing pages
export {
  useRolesPaged,
  usePermissionsPaged,
} from '~/features/rbac/hooks';
export type { CorePaginatedResponse, RoleDto, PermissionDto } from '~/lib/api';

