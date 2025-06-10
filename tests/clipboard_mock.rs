use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MockClipboard {
    content: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    types: Arc<Mutex<Vec<String>>>,
}

impl MockClipboard {
    pub fn new() -> Self {
        Self {
            content: Arc::new(Mutex::new(HashMap::new())),
            types: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn set_text(&self, text: &str) {
        let mut content = self.content.lock().unwrap();
        let mut types = self.types.lock().unwrap();
        
        content.insert("text/plain".to_string(), text.as_bytes().to_vec());
        types.clear();
        types.push("text/plain".to_string());
        types.push("UTF8_STRING".to_string());
    }
    
    pub fn set_image(&self, format: &str, data: Vec<u8>) {
        let mut content = self.content.lock().unwrap();
        let mut types = self.types.lock().unwrap();
        
        content.insert(format.to_string(), data);
        types.clear();
        types.push(format.to_string());
    }
    
    pub fn set_file_list(&self, files: Vec<&str>) {
        let mut content = self.content.lock().unwrap();
        let mut types = self.types.lock().unwrap();
        
        let file_data = files.join("\n");
        content.insert("text/uri-list".to_string(), file_data.as_bytes().to_vec());
        types.clear();
        types.push("text/uri-list".to_string());
    }
    
    pub fn copy_image(&self, mime_type: &str, data: &[u8]) -> Result<(), String> {
        // Simulate the wl-copy operation
        if !mime_type.starts_with("image/") {
            return Err("Invalid image MIME type".to_string());
        }
        
        if data.is_empty() {
            return Err("Empty image data".to_string());
        }
        
        // Update clipboard content
        let mut content = self.content.lock().unwrap();
        let mut types = self.types.lock().unwrap();
        
        content.insert(mime_type.to_string(), data.to_vec());
        types.clear();
        types.push(mime_type.to_string());
        
        Ok(())
    }
    
    pub fn get_types(&self) -> Vec<String> {
        self.types.lock().unwrap().clone()
    }
    
    pub fn get_content(&self, mime_type: &str) -> Option<Vec<u8>> {
        self.content.lock().unwrap().get(mime_type).cloned()
    }
    
    pub fn clear(&self) {
        self.content.lock().unwrap().clear();
        self.types.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_clipboard_text() {
        let clipboard = MockClipboard::new();
        clipboard.set_text("Hello, World!");
        
        let types = clipboard.get_types();
        assert!(types.contains(&"text/plain".to_string()));
        
        let content = clipboard.get_content("text/plain").unwrap();
        assert_eq!(String::from_utf8(content).unwrap(), "Hello, World!");
    }
    
    #[test]
    fn test_mock_clipboard_image() {
        let clipboard = MockClipboard::new();
        let png_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
        clipboard.set_image("image/png", png_data.clone());
        
        let types = clipboard.get_types();
        assert!(types.contains(&"image/png".to_string()));
        
        let content = clipboard.get_content("image/png").unwrap();
        assert_eq!(content, png_data);
    }
    
    #[test]
    fn test_mock_clipboard_clear() {
        let clipboard = MockClipboard::new();
        clipboard.set_text("test");
        clipboard.clear();
        
        assert!(clipboard.get_types().is_empty());
        assert!(clipboard.get_content("text/plain").is_none());
    }
    
    #[test]
    fn test_mock_clipboard_copy_image() {
        let clipboard = MockClipboard::new();
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
        
        // Test successful copy
        let result = clipboard.copy_image("image/png", &png_data);
        assert!(result.is_ok());
        
        let types = clipboard.get_types();
        assert!(types.contains(&"image/png".to_string()));
        
        let content = clipboard.get_content("image/png").unwrap();
        assert_eq!(content, png_data);
    }
    
    #[test]
    fn test_mock_clipboard_copy_image_invalid_mime() {
        let clipboard = MockClipboard::new();
        let data = vec![1, 2, 3, 4];
        
        let result = clipboard.copy_image("text/plain", &data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid image MIME type");
    }
    
    #[test]
    fn test_mock_clipboard_copy_image_empty_data() {
        let clipboard = MockClipboard::new();
        let empty_data = vec![];
        
        let result = clipboard.copy_image("image/png", &empty_data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty image data");
    }
    
    #[test]
    fn test_mock_clipboard_copy_different_formats() {
        let clipboard = MockClipboard::new();
        let formats = vec![
            ("image/png", vec![0x89, 0x50, 0x4E, 0x47]),
            ("image/jpeg", vec![0xFF, 0xD8, 0xFF, 0xE0]),
            ("image/gif", vec![0x47, 0x49, 0x46, 0x38]),
        ];
        
        for (mime_type, data) in formats {
            let result = clipboard.copy_image(mime_type, &data);
            assert!(result.is_ok(), "Failed to copy {}", mime_type);
            
            let retrieved_data = clipboard.get_content(mime_type).unwrap();
            assert_eq!(retrieved_data, data);
        }
    }
}
