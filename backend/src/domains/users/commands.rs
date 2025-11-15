use crate::core::cqrs::Command;
use crate::models::User;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Register User Command (Sign up)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterUserCommand {
    #[validate(length(min = 3, max = 100))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 6))]
    pub password: String,
    
    pub full_name: Option<String>,
    pub role: Option<String>,
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
    pub id: i32,
    
    #[validate(email)]
    pub email: Option<String>,
    
    pub full_name: Option<String>,
    pub role: Option<String>,
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
    pub user_id: i32,
    
    #[validate(length(min = 6))]
    pub old_password: String,
    
    #[validate(length(min = 6))]
    pub new_password: String,
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
    pub id: i32,
}

impl Command for DeleteUserCommand {
    type Response = ();

    fn command_name(&self) -> &'static str {
        "DeleteUser"
    }
}
