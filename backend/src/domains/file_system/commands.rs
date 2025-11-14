use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::core::cqrs::Command;

/// Create File Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateFileCommand {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub size: i64,
    pub mime_type: String,
    pub owner_id: Uuid, // User creating the file
}

impl Command for CreateFileCommand {
    type Response = Uuid;

    fn command_name(&self) -> &'static str {
        "create_file"
    }
}

/// Move File Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MoveFileCommand {
    pub file_id: Uuid,
    pub new_parent_id: Option<Uuid>,
    pub moved_by: Uuid, // User performing the move
}

impl Command for MoveFileCommand {
    type Response = ();

    fn command_name(&self) -> &'static str {
        "move_file"
    }
}

/// Delete File Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DeleteFileCommand {
    pub file_id: Uuid,
    pub deleted_by: Uuid, // User performing the delete
}

impl Command for DeleteFileCommand {
    type Response = ();

    fn command_name(&self) -> &'static str {
        "delete_file"
    }
}

/// Rename File Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RenameFileCommand {
    pub file_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub new_name: String,
    pub renamed_by: Uuid, // User performing the rename
}

impl Command for RenameFileCommand {
    type Response = ();

    fn command_name(&self) -> &'static str {
        "rename_file"
    }
}

/// Set File Permissions Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SetFilePermissionsCommand {
    pub file_id: Uuid,
    pub acl: Vec<crate::core::shared::AccessControlEntry>,
}

impl Command for SetFilePermissionsCommand {
    type Response = ();

    fn command_name(&self) -> &'static str {
        "set_file_permissions"
    }
}

/// Create Folder Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateFolderCommand {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub owner_id: Uuid, // User creating the folder
}

impl Command for CreateFolderCommand {
    type Response = Uuid;

    fn command_name(&self) -> &'static str {
        "create_folder"
    }
}

/// Move Folder Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MoveFolderCommand {
    pub folder_id: Uuid,
    pub new_parent_id: Option<Uuid>,
}

impl Command for MoveFolderCommand {
    type Response = ();

    fn command_name(&self) -> &'static str {
        "move_folder"
    }
}

/// Delete Folder Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DeleteFolderCommand {
    pub folder_id: Uuid,
}

impl Command for DeleteFolderCommand {
    type Response = ();

    fn command_name(&self) -> &'static str {
        "delete_folder"
    }
}

/// Rename Folder Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RenameFolderCommand {
    pub folder_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub new_name: String,
}

impl Command for RenameFolderCommand {
    type Response = ();

    fn command_name(&self) -> &'static str {
        "rename_folder"
    }
}

