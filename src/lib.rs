use std::process::Command;

pub fn run_command(args: &[&str]) -> Option<Vec<u8>> {
    let output = Command::new(args[0])
        .args(&args[1..])
        .output()
        .ok()?;
    if output.status.success() {
        Some(output.stdout)
    } else {
        None
    }
}

pub fn has_mime_type(types: &str, mime: &str) -> bool {
    types.lines().any(|line| line == mime)
}

pub fn detect_clipboard_content_type(types: &str) -> ClipboardContentType {
    let is_image = has_mime_type(types, "image/png") 
        || has_mime_type(types, "image/jpeg") 
        || has_mime_type(types, "image/gif");
    let is_text = has_mime_type(types, "text/plain") 
        || has_mime_type(types, "text/plain;charset=utf-8") 
        || has_mime_type(types, "UTF8_STRING") 
        || has_mime_type(types, "TEXT") 
        || has_mime_type(types, "STRING");
    let is_file = has_mime_type(types, "text/uri-list");

    if is_file {
        ClipboardContentType::File
    } else if is_image {
        ClipboardContentType::Image
    } else if is_text {
        ClipboardContentType::Text
    } else {
        ClipboardContentType::Unsupported
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClipboardContentType {
    Text,
    Image,
    File,
    Unsupported,
}

pub fn get_image_format_from_types(types: &str) -> Option<&'static str> {
    if has_mime_type(types, "image/png") {
        Some("image/png")
    } else if has_mime_type(types, "image/jpeg") {
        Some("image/jpeg")
    } else if has_mime_type(types, "image/gif") {
        Some("image/gif")
    } else {
        None
    }
}

pub fn copy_image_to_clipboard(mime_type: &str, data: &[u8]) -> Result<(), String> {
    if !mime_type.starts_with("image/") {
        return Err("Invalid image MIME type".to_string());
    }
    
    if data.is_empty() {
        return Err("Empty image data".to_string());
    }
    
    use std::io::Write;
    let mut child = std::process::Command::new("wl-copy")
        .arg("--type")
        .arg(mime_type)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn wl-copy: {}", e))?;
    
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(data)
            .map_err(|e| format!("Failed to write image data: {}", e))?;
    }
    
    let exit_status = child.wait()
        .map_err(|e| format!("Failed to wait for wl-copy: {}", e))?;
    
    if exit_status.success() {
        Ok(())
    } else {
        Err("wl-copy command failed".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_mime_type_single_line() {
        let types = "text/plain";
        assert!(has_mime_type(types, "text/plain"));
        assert!(!has_mime_type(types, "image/png"));
    }

    #[test]
    fn test_has_mime_type_multiple_lines() {
        let types = "text/plain\nimage/png\ntext/html";
        assert!(has_mime_type(types, "text/plain"));
        assert!(has_mime_type(types, "image/png"));
        assert!(has_mime_type(types, "text/html"));
        assert!(!has_mime_type(types, "image/jpeg"));
    }

    #[test]
    fn test_has_mime_type_empty() {
        let types = "";
        assert!(!has_mime_type(types, "text/plain"));
    }

    #[test]
    fn test_detect_clipboard_content_type_text() {
        let types = "text/plain\nUTF8_STRING";
        assert_eq!(detect_clipboard_content_type(types), ClipboardContentType::Text);
    }

    #[test]
    fn test_detect_clipboard_content_type_image() {
        let types = "image/png\nimage/jpeg";
        assert_eq!(detect_clipboard_content_type(types), ClipboardContentType::Image);
    }

    #[test]
    fn test_detect_clipboard_content_type_file() {
        let types = "text/uri-list\ntext/plain";
        assert_eq!(detect_clipboard_content_type(types), ClipboardContentType::File);
    }

    #[test]
    fn test_detect_clipboard_content_type_unsupported() {
        let types = "application/octet-stream\napplication/pdf";
        assert_eq!(detect_clipboard_content_type(types), ClipboardContentType::Unsupported);
    }

    #[test]
    fn test_get_image_format_from_types() {
        assert_eq!(get_image_format_from_types("image/png"), Some("image/png"));
        assert_eq!(get_image_format_from_types("image/jpeg"), Some("image/jpeg"));
        assert_eq!(get_image_format_from_types("image/gif"), Some("image/gif"));
        assert_eq!(get_image_format_from_types("text/plain"), None);
    }

    #[test]
    fn test_get_image_format_priority() {
        // PNG should have priority when multiple formats are available
        let types = "image/jpeg\nimage/png\nimage/gif";
        assert_eq!(get_image_format_from_types(types), Some("image/png"));
    }

    #[test]
    fn test_copy_image_to_clipboard_validation() {
        // Test invalid MIME type
        let result = copy_image_to_clipboard("text/plain", &[1, 2, 3]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid image MIME type");
        
        // Test empty data
        let result = copy_image_to_clipboard("image/png", &[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty image data");
    }
}
