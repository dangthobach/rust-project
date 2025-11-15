use crate::core::cqrs::Command;
use crate::models::Client;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Create Client Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateClientCommand {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 1, max = 50))]
    pub phone: String,
    
    #[validate(length(max = 500))]
    pub address: Option<String>,
    
    pub company: Option<String>,
    pub status: Option<String>,
}

impl Command for CreateClientCommand {
    type Response = Client;

    fn command_name(&self) -> &'static str {
        "CreateClient"
    }
}

/// Update Client Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateClientCommand {
    pub id: i32,
    
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    
    #[validate(email)]
    pub email: Option<String>,
    
    #[validate(length(min = 1, max = 50))]
    pub phone: Option<String>,
    
    #[validate(length(max = 500))]
    pub address: Option<String>,
    
    pub company: Option<String>,
    pub status: Option<String>,
}

impl Command for UpdateClientCommand {
    type Response = Client;

    fn command_name(&self) -> &'static str {
        "UpdateClient"
    }
}

/// Delete Client Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DeleteClientCommand {
    pub id: i32,
}

impl Command for DeleteClientCommand {
    type Response = ();

    fn command_name(&self) -> &'static str {
        "DeleteClient"
    }
}
