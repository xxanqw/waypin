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
    fn test_has_mime_type_whitespace() {
        let types = " \n \t \n ";
        assert!(!has_mime_type(types, "text/plain"));
        assert!(has_mime_type(types, " "));
    }

    #[test]
    fn test_has_mime_type_case_sensitive() {
        let types = "text/plain";
        assert!(has_mime_type(types, "text/plain"));
        assert!(!has_mime_type(types, "TEXT/PLAIN"));
        assert!(!has_mime_type(types, "Text/Plain"));
    }

    #[test]
    fn test_has_mime_type_partial_match() {
        let types = "application/json";
        assert!(has_mime_type(types, "application/json"));
        assert!(!has_mime_type(types, "application"));
        assert!(!has_mime_type(types, "json"));
    }

    #[test]
    fn test_detect_clipboard_content_type_text() {
        let types = "text/plain\nUTF8_STRING";
        assert_eq!(detect_clipboard_content_type(types), ClipboardContentType::Text);
    }

    #[test]
    fn test_detect_clipboard_content_type_text_variants() {
        assert_eq!(detect_clipboard_content_type("text/plain"), ClipboardContentType::Text);
        assert_eq!(detect_clipboard_content_type("text/plain;charset=utf-8"), ClipboardContentType::Text);
        assert_eq!(detect_clipboard_content_type("UTF8_STRING"), ClipboardContentType::Text);
        assert_eq!(detect_clipboard_content_type("TEXT"), ClipboardContentType::Text);
        assert_eq!(detect_clipboard_content_type("STRING"), ClipboardContentType::Text);
    }

    #[test]
    fn test_detect_clipboard_content_type_image() {
        let types = "image/png\nimage/jpeg";
        assert_eq!(detect_clipboard_content_type(types), ClipboardContentType::Image);
    }

    #[test]
    fn test_detect_clipboard_content_type_image_variants() {
        assert_eq!(detect_clipboard_content_type("image/png"), ClipboardContentType::Image);
        assert_eq!(detect_clipboard_content_type("image/jpeg"), ClipboardContentType::Image);
        assert_eq!(detect_clipboard_content_type("image/gif"), ClipboardContentType::Image);
    }

    #[test]
    fn test_detect_clipboard_content_type_file() {
        let types = "text/uri-list\ntext/plain";
        assert_eq!(detect_clipboard_content_type(types), ClipboardContentType::File);
    }

    #[test]
    fn test_detect_clipboard_content_type_priority() {
        // File should have highest priority
        let types = "text/uri-list\nimage/png\ntext/plain";
        assert_eq!(detect_clipboard_content_type(types), ClipboardContentType::File);
        
        // Image should have priority over text
        let types = "image/png\ntext/plain";
        assert_eq!(detect_clipboard_content_type(types), ClipboardContentType::Image);
    }

    #[test]
    fn test_detect_clipboard_content_type_unsupported() {
        let types = "application/octet-stream\napplication/pdf";
        assert_eq!(detect_clipboard_content_type(types), ClipboardContentType::Unsupported);
    }

    #[test]
    fn test_detect_clipboard_content_type_empty() {
        assert_eq!(detect_clipboard_content_type(""), ClipboardContentType::Unsupported);
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
        
        // JPEG should be second priority
        let types = "image/jpeg\nimage/gif";
        assert_eq!(get_image_format_from_types(types), Some("image/jpeg"));
        
        // GIF should be last priority
        let types = "image/gif";
        assert_eq!(get_image_format_from_types(types), Some("image/gif"));
    }

    #[test]
    fn test_get_image_format_no_images() {
        assert_eq!(get_image_format_from_types("text/plain\napplication/json"), None);
        assert_eq!(get_image_format_from_types(""), None);
    }

    #[test]
    fn test_get_image_format_mixed_content() {
        let types = "text/plain\nimage/png\napplication/json";
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
        
        // Test invalid image MIME types
        let result = copy_image_to_clipboard("application/octet-stream", &[1, 2, 3]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid image MIME type");
        
        let result = copy_image_to_clipboard("video/mp4", &[1, 2, 3]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid image MIME type");
    }

    #[test]
    fn test_copy_image_to_clipboard_valid_mime_types() {
        // Note: These tests only validate the input parameters, not the actual command execution
        // since wl-copy might not be available in test environment
        
        let test_data = vec![1, 2, 3, 4, 5];
        
        // Test that valid MIME types don't fail validation
        let mime_types = vec!["image/png", "image/jpeg", "image/gif", "image/bmp", "image/webp"];
        
        for mime_type in mime_types {
            let result = copy_image_to_clipboard(mime_type, &test_data);
            // The result might be an error due to wl-copy not being available,
            // but it shouldn't be a validation error
            if let Err(err) = result {
                assert!(!err.contains("Invalid image MIME type"));
                assert!(!err.contains("Empty image data"));
            }
        }
    }

    #[test]
    fn test_clipboard_content_type_clone() {
        let content_type = ClipboardContentType::Image;
        let cloned = content_type.clone();
        assert_eq!(content_type, cloned);
    }

    #[test]
    fn test_clipboard_content_type_debug() {
        let content_type = ClipboardContentType::Text;
        let debug_str = format!("{:?}", content_type);
        assert_eq!(debug_str, "Text");
    }

    #[test]
    fn test_run_command_edge_cases() {
        // Test with single arg (command only, no additional args)
        let _result = run_command(&["echo"]);
        // Should work fine - echo with no args
        
        // Test with non-existent command
        let result = run_command(&["this_command_does_not_exist_12345"]);
        assert!(result.is_none());
    }

    #[test]
    fn test_run_command_success() {
        // Test with a command that should exist on most systems
        let result = run_command(&["echo", "test"]);
        if let Some(output) = result {
            assert_eq!(String::from_utf8_lossy(&output).trim(), "test");
        }
        // Note: We don't assert success here since the test environment might vary
    }

    #[test]
    fn test_multiple_mime_types_complex() {
        let complex_types = "text/plain\ntext/plain;charset=utf-8\nimage/png\nimage/jpeg\ntext/uri-list\napplication/x-kde-cutselection\nUTF8_STRING";
        
        // Should detect as file due to text/uri-list priority
        assert_eq!(detect_clipboard_content_type(complex_types), ClipboardContentType::File);
        
        // Should get PNG format due to priority
        assert_eq!(get_image_format_from_types(complex_types), Some("image/png"));
        
        // Should detect individual types correctly
        assert!(has_mime_type(complex_types, "text/plain"));
        assert!(has_mime_type(complex_types, "image/png"));
        assert!(has_mime_type(complex_types, "text/uri-list"));
        assert!(!has_mime_type(complex_types, "image/gif"));
    }
}
