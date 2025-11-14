use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Value Object trait
/// Value Objects không có identity, được so sánh bằng value
pub trait ValueObject: Clone + PartialEq + Eq + Debug + Send + Sync {
    /// Validate value object
    fn validate(&self) -> Result<(), String>;
}

/// Example: File Path value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilePath {
    path: String,
}

impl FilePath {
    pub fn new(path: String) -> Result<Self, String> {
        let path = path.trim().to_string();
        if path.is_empty() {
            return Err("Path cannot be empty".to_string());
        }
        if !path.starts_with('/') {
            return Err("Path must start with /".to_string());
        }
        Ok(Self { path })
    }

    pub fn as_str(&self) -> &str {
        &self.path
    }

    pub fn parent(&self) -> Option<Self> {
        let parent = self.path.rsplitn(2, '/').nth(1)?;
        if parent.is_empty() {
            Some(Self::new("/".to_string()).unwrap())
        } else {
            Self::new(parent.to_string()).ok()
        }
    }

    pub fn join(&self, name: &str) -> Result<Self, String> {
        let path = if self.path.ends_with('/') {
            format!("{}{}", self.path, name)
        } else {
            format!("{}/{}", self.path, name)
        };
        Self::new(path)
    }
}

impl ValueObject for FilePath {
    fn validate(&self) -> Result<(), String> {
        if self.path.is_empty() {
            return Err("Path cannot be empty".to_string());
        }
        if !self.path.starts_with('/') {
            return Err("Path must start with /".to_string());
        }
        Ok(())
    }
}

