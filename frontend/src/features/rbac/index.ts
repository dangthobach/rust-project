// Public API of the RBAC feature module
// Import pages/hooks through here to maintain clean boundaries

export { default as RoleList } from './pages/RoleList';
export { default as RoleDetail } from './pages/RoleDetail';
export { default as RoleCreate } from './pages/RoleCreate';
export { default as RoleEdit } from './pages/RoleEdit';
export { default as PermissionMatrix } from './pages/PermissionMatrix';
export { default as UserRoleList } from './pages/UserRoleList';

export {
  useRolesPaged,
  useRole,
  usePermissionsPaged,
  usePermissionMatrix,
  useUsersWithRoles,
  useCreateRole,
  useUpdateRole,
  useDeleteRole,
  useDeleteRolesBulk,
  useUpdateRolePermissions,
  useAssignUserRole,
  useBulkAssignUserRole,
  rbacKeys,
} from './hooks';
