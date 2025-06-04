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
}
