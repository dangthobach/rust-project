use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::core::cqrs::Command;
use crate::domains::rbac::handlers::{PermissionDto, RoleDto};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRoleCommand {
    #[validate(length(min = 1, max = 64))]
    pub slug: String,
    pub description: Option<String>,
}
impl Command for CreateRoleCommand {
    type Response = RoleDto;
    fn command_name(&self) -> &'static str { "CreateRole" }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateRoleCommand {
    #[validate(length(min = 1))]
    pub role_id: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}
impl Command for UpdateRoleCommand {
    type Response = RoleDto;
    fn command_name(&self) -> &'static str { "UpdateRole" }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DeleteRoleCommand {
    #[validate(length(min = 1))]
    pub role_id: String,
}
impl Command for DeleteRoleCommand {
    type Response = ();
    fn command_name(&self) -> &'static str { "DeleteRole" }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePermissionCommand {
    #[validate(length(min = 1, max = 128))]
    pub code: String,
    pub description: Option<String>,
}
impl Command for CreatePermissionCommand {
    type Response = PermissionDto;
    fn command_name(&self) -> &'static str { "CreatePermission" }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdatePermissionCommand {
    #[validate(length(min = 1))]
    pub code: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}
impl Command for UpdatePermissionCommand {
    type Response = PermissionDto;
    fn command_name(&self) -> &'static str { "UpdatePermission" }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DeletePermissionCommand {
    #[validate(length(min = 1))]
    pub code: String,
}
impl Command for DeletePermissionCommand {
    type Response = ();
    fn command_name(&self) -> &'static str { "DeletePermission" }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AssignPermissionToRoleCommand {
    #[validate(length(min = 1))]
    pub role_id: String,
    #[validate(length(min = 1))]
    pub code: String,
}
impl Command for AssignPermissionToRoleCommand {
    type Response = ();
    fn command_name(&self) -> &'static str { "AssignPermissionToRole" }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RevokePermissionFromRoleCommand {
    #[validate(length(min = 1))]
    pub role_id: String,
    #[validate(length(min = 1))]
    pub code: String,
}
impl Command for RevokePermissionFromRoleCommand {
    type Response = ();
    fn command_name(&self) -> &'static str { "RevokePermissionFromRole" }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AssignRoleToUserCommand {
    #[validate(length(min = 1))]
    pub user_id: String,
    #[validate(length(min = 1))]
    pub role_id: String,
}
impl Command for AssignRoleToUserCommand {
    type Response = ();
    fn command_name(&self) -> &'static str { "AssignRoleToUser" }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RevokeRoleFromUserCommand {
    #[validate(length(min = 1))]
    pub user_id: String,
    #[validate(length(min = 1))]
    pub role_id: String,
}
impl Command for RevokeRoleFromUserCommand {
    type Response = ();
    fn command_name(&self) -> &'static str { "RevokeRoleFromUser" }
}
