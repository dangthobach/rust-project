use serde::{Deserialize, Serialize};

use crate::core::cqrs::Query;
use crate::domains::rbac::handlers::{PermissionDto, RoleDto};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRolesQuery;
impl Query for ListRolesQuery {
    type Response = Vec<RoleDto>;
    fn query_name(&self) -> &'static str { "ListRoles" }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPermissionsQuery;
impl Query for ListPermissionsQuery {
    type Response = Vec<PermissionDto>;
    fn query_name(&self) -> &'static str { "ListPermissions" }
}
