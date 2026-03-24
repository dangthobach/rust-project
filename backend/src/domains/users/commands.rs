use crate::core::cqrs::Command;
use crate::models::User;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Register User Command (Sign up)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterUserCommand {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 6))]
    pub password: String,
    
    #[validate(length(min = 2, max = 255))]
    pub full_name: String,
    pub role: Option<String>,
    pub actor_id: Option<String>,
}

impl Command for RegisterUserCommand {
    type Response = User;

    fn command_name(&self) -> &'static str {
        "RegisterUser"
    }
}

/// Update User Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUserCommand {
    #[validate(length(min = 1))]
    pub id: String,
    
    #[validate(email)]
    pub email: Option<String>,
    
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: Option<String>,
    pub actor_id: Option<String>,
}

impl Command for UpdateUserCommand {
    type Response = User;

    fn command_name(&self) -> &'static str {
        "UpdateUser"
    }
}

/// Change Password Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ChangePasswordCommand {
    #[validate(length(min = 1))]
    pub user_id: String,
    
    #[validate(length(min = 6))]
    pub old_password: String,
    
    #[validate(length(min = 6))]
    pub new_password: String,
    pub actor_id: Option<String>,
}

impl Command for ChangePasswordCommand {
    type Response = ();

    fn command_name(&self) -> &'static str {
        "ChangePassword"
    }
}

/// Delete User Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DeleteUserCommand {
    #[validate(length(min = 1))]
    pub id: String,
    pub actor_id: Option<String>,
}

impl Command for DeleteUserCommand {
    type Response = ();

    fn command_name(&self) -> &'static str {
        "DeleteUser"
    }
}
