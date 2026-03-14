use crate::error::AppError;

/// Allowed MIME types for file uploads
pub const ALLOWED_TYPES: &[&str] = &[
    // Images
    "image/jpeg",
    "image/jpg",
    "image/png",
    "image/gif",
    "image/webp",
    "image/svg+xml",
    // Documents
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document", // .docx
    "application/vnd.oasis.opendocument.text", // .odt
    // Spreadsheets
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", // .xlsx
    "application/vnd.oasis.opendocument.spreadsheet", // .ods
    // Presentations
    "application/vnd.ms-powerpoint",
    "application/vnd.openxmlformats-officedocument.presentationml.presentation", // .pptx
    // Text
    "text/plain",
    "text/csv",
    "text/markdown",
    // Archives
    "application/zip",
    "application/x-rar-compressed",
    "application/x-7z-compressed",
    // Other
    "application/json",
    "application/xml",
];

/// Blocked file extensions (security risk)
pub const BLOCKED_EXTENSIONS: &[&str] = &[
    "exe", "bat", "cmd", "com", "scr", "pif", "msi", "vbs", "js", "jar", "app", "deb", "rpm",
    "dmg", "pkg", "sh", "bash", "ps1", "psm1",
];

/// Validate file MIME type against whitelist
pub fn validate_file_type(mime_type: &str) -> Result<(), AppError> {
    if ALLOWED_TYPES.contains(&mime_type) {
        Ok(())
    } else {
        Err(AppError::ValidationError(format!(
            "File type '{}' is not allowed. Allowed types: images, documents, spreadsheets, text files, archives",
            mime_type
        )))
    }
}

/// Validate file size against maximum allowed size
pub fn validate_file_size(size: usize, max_size: usize) -> Result<(), AppError> {
    if size > max_size {
        let size_mb = size as f64 / 1_048_576.0;
        let max_mb = max_size as f64 / 1_048_576.0;
        Err(AppError::ValidationError(format!(
            "File size {:.2}MB exceeds maximum allowed size of {:.2}MB",
            size_mb, max_mb
        )))
    } else if size == 0 {
        Err(AppError::ValidationError(
            "File is empty (0 bytes)".to_string(),
        ))
    } else {
        Ok(())
    }
}

/// Sanitize filename to remove dangerous characters
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            // Replace path separators and special characters
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            // Keep other characters
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Validate file extension against blocked list
pub fn validate_file_extension(filename: &str) -> Result<(), AppError> {
    if let Some(ext) = filename.rsplit('.').next() {
        let ext_lower = ext.to_lowercase();
        if BLOCKED_EXTENSIONS.contains(&ext_lower.as_str()) {
            return Err(AppError::ValidationError(format!(
                "File extension '.{}' is not allowed for security reasons",
                ext
            )));
        }
    }
    Ok(())
}

/// Comprehensive file validation
pub fn validate_upload(
    filename: &str,
    file_bytes: &[u8],
    mime_type: &str,
    max_size: usize,
) -> Result<String, AppError> {
    // 1. Validate file size
    validate_file_size(file_bytes.len(), max_size)?;

    // 2. Validate MIME type
    validate_file_type(mime_type)?;

    // 3. Sanitize filename
    let safe_filename = sanitize_filename(filename);

    // 4. Validate extension
    validate_file_extension(&safe_filename)?;

    // 5. Check filename is not empty after sanitization
    if safe_filename.is_empty() || safe_filename == "." {
        return Err(AppError::ValidationError(
            "Invalid filename".to_string(),
        ));
    }

    Ok(safe_filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_allowed_image_type() {
        assert!(validate_file_type("image/jpeg").is_ok());
        assert!(validate_file_type("image/png").is_ok());
    }

    #[test]
    fn test_validate_blocked_executable() {
        assert!(validate_file_type("application/x-executable").is_err());
        assert!(validate_file_type("application/x-msdownload").is_err());
    }

    #[test]
    fn test_validate_file_size_within_limit() {
        let max_size = 10 * 1024 * 1024; // 10MB
        assert!(validate_file_size(5_000_000, max_size).is_ok());
    }

    #[test]
    fn test_validate_file_size_exceeds_limit() {
        let max_size = 10 * 1024 * 1024; // 10MB
        assert!(validate_file_size(15_000_000, max_size).is_err());
    }

    #[test]
    fn test_validate_empty_file() {
        let max_size = 10 * 1024 * 1024;
        assert!(validate_file_size(0, max_size).is_err());
    }

    #[test]
    fn test_sanitize_filename_removes_special_chars() {
        assert_eq!(sanitize_filename("file/name.txt"), "file_name.txt");
        assert_eq!(sanitize_filename("test:file?.doc"), "test_file_.doc");
        assert_eq!(sanitize_filename("path\\to\\file.pdf"), "path_to_file.pdf");
    }

    #[test]
    fn test_sanitize_filename_preserves_normal() {
        assert_eq!(
            sanitize_filename("normal-file_name.pdf"),
            "normal-file_name.pdf"
        );
    }

    #[test]
    fn test_validate_blocked_extension() {
        assert!(validate_file_extension("malware.exe").is_err());
        assert!(validate_file_extension("script.bat").is_err());
        assert!(validate_file_extension("virus.vbs").is_err());
    }

    #[test]
    fn test_validate_allowed_extension() {
        assert!(validate_file_extension("document.pdf").is_ok());
        assert!(validate_file_extension("image.png").is_ok());
    }

    #[test]
    fn test_comprehensive_validation() {
        let data = vec![0u8; 1000]; // 1KB file
        let max_size = 10 * 1024 * 1024; // 10MB

        // Valid file
        assert!(validate_upload("test.pdf", &data, "application/pdf", max_size).is_ok());

        // Invalid MIME type
        assert!(validate_upload("test.exe", &data, "application/x-executable", max_size).is_err());

        // File too large
        let large_data = vec![0u8; 11 * 1024 * 1024];
        assert!(validate_upload("test.pdf", &large_data, "application/pdf", max_size).is_err());
    }
}
