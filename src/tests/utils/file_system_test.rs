#[cfg(test)]
mod tests {
    use std::fs;
    use std::io;
    use std::path::Path;
    use tempfile::{tempdir, NamedTempFile};

    use crate::utils::file_exists;
    use crate::utils::read_file_to_string;
    use crate::utils::write_string_to_file;

    #[test]
    fn test_file_exists_with_existing_file() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();

        // Test that file_exists returns true for existing file
        assert!(file_exists(file_path));
    }

    #[test]
    fn test_file_exists_with_non_existing_file() {
        let non_existing_path = "/tmp/this_file_should_not_exist_12345.txt";

        // Ensure the file doesn't exist
        if Path::new(non_existing_path).exists() {
            fs::remove_file(non_existing_path).ok();
        }

        // Test that file_exists returns false for non-existing file
        assert!(!file_exists(non_existing_path));
    }

    #[test]
    fn test_file_exists_with_directory() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let dir_path = temp_dir.path().to_str().unwrap();

        // Test that file_exists returns true for existing directory
        assert!(file_exists(dir_path));
    }

    #[test]
    fn test_file_exists_with_empty_path() {
        // Test with empty string
        assert!(!file_exists(""));
    }

    #[test]
    fn test_file_exists_with_relative_path() {
        // Test with current directory (should exist)
        assert!(file_exists("."));
        assert!(file_exists("./"));
    }

    #[test]
    fn test_read_file_to_string_success() {
        // Create a temporary file with content
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();
        let test_content = "Hello, World!\nThis is a test file.";

        // Write content to the file
        fs::write(file_path, test_content).expect("Failed to write to temp file");

        // Test reading the file
        let result = read_file_to_string(file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content);
    }

    #[test]
    fn test_read_file_to_string_empty_file() {
        // Create a temporary empty file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();

        // Test reading empty file
        let result = read_file_to_string(file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_read_file_to_string_utf8_content() {
        // Create a temporary file with UTF-8 content
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();
        let test_content = "Hello, 🌍! Merhaba dünya! 你好世界! 🚀✨";

        // Write UTF-8 content to the file
        fs::write(file_path, test_content).expect("Failed to write to temp file");

        // Test reading the UTF-8 file
        let result = read_file_to_string(file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content);
    }

    #[test]
    fn test_read_file_to_string_non_existing_file() {
        let non_existing_path = "/tmp/this_file_should_not_exist_67890.txt";

        // Ensure the file doesn't exist
        if Path::new(non_existing_path).exists() {
            fs::remove_file(non_existing_path).ok();
        }

        // Test reading non-existing file
        let result = read_file_to_string(non_existing_path);
        assert!(result.is_err());

        // Check that it's the right kind of error
        match result.unwrap_err().kind() {
            io::ErrorKind::NotFound => (),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_read_file_to_string_directory() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let dir_path = temp_dir.path().to_str().unwrap();

        // Test reading a directory (should fail)
        let result = read_file_to_string(dir_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_string_to_file_success() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();
        let test_content = "Test content for writing";

        // Test writing to the file
        let result = write_string_to_file(file_path, test_content);
        assert!(result.is_ok());

        // Verify the content was written correctly
        let written_content = fs::read_to_string(file_path).expect("Failed to read written file");
        assert_eq!(written_content, test_content);
    }

    #[test]
    fn test_write_string_to_file_empty_content() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();

        // Test writing empty content
        let result = write_string_to_file(file_path, "");
        assert!(result.is_ok());

        // Verify the file is empty
        let written_content = fs::read_to_string(file_path).expect("Failed to read written file");
        assert_eq!(written_content, "");
    }

    #[test]
    fn test_write_string_to_file_utf8_content() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();
        let test_content = "UTF-8 test: 🎉 Türkçe karakter: ğüşıöç 中文测试";

        // Test writing UTF-8 content
        let result = write_string_to_file(file_path, test_content);
        assert!(result.is_ok());

        // Verify the UTF-8 content was written correctly
        let written_content = fs::read_to_string(file_path).expect("Failed to read written file");
        assert_eq!(written_content, test_content);
    }

    #[test]
    fn test_write_string_to_file_overwrite() {
        // Create a temporary file with initial content
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();
        let initial_content = "Initial content";
        let new_content = "New content that overwrites";

        // Write initial content
        fs::write(file_path, initial_content).expect("Failed to write initial content");

        // Test overwriting with new content
        let result = write_string_to_file(file_path, new_content);
        assert!(result.is_ok());

        // Verify the content was overwritten
        let written_content = fs::read_to_string(file_path).expect("Failed to read written file");
        assert_eq!(written_content, new_content);
        assert_ne!(written_content, initial_content);
    }

    #[test]
    fn test_write_string_to_file_new_file() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("new_test_file.txt");
        let file_path_str = file_path.to_str().unwrap();
        let test_content = "Content for new file";

        // Ensure the file doesn't exist initially
        assert!(!file_exists(file_path_str));

        // Test writing to a new file
        let result = write_string_to_file(file_path_str, test_content);
        assert!(result.is_ok());

        // Verify the file was created and content is correct
        assert!(file_exists(file_path_str));
        let written_content = fs::read_to_string(file_path_str).expect("Failed to read new file");
        assert_eq!(written_content, test_content);
    }

    #[test]
    fn test_write_string_to_file_invalid_path() {
        // Test writing to an invalid path (non-existent directory)
        let invalid_path = "/this/path/should/not/exist/test_file.txt";
        let test_content = "This should fail";

        let result = write_string_to_file(invalid_path, test_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_string_to_file_large_content() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();

        // Create large content (1MB of text)
        let large_content = "A".repeat(1024 * 1024);

        // Test writing large content
        let result = write_string_to_file(file_path, &large_content);
        assert!(result.is_ok());

        // Verify the large content was written correctly
        let written_content = fs::read_to_string(file_path).expect("Failed to read large file");
        assert_eq!(written_content, large_content);
        assert_eq!(written_content.len(), 1024 * 1024);
    }

    #[test]
    fn test_round_trip_file_operations() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();
        let original_content = "Round trip test content\nLine 2\nLine 3 with 🚀";

        // Test the complete round trip: write -> read -> verify
        let write_result = write_string_to_file(file_path, original_content);
        assert!(write_result.is_ok());

        let read_result = read_file_to_string(file_path);
        assert!(read_result.is_ok());

        let read_content = read_result.unwrap();
        assert_eq!(read_content, original_content);
    }

    #[test]
    fn test_multiple_operations_same_file() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();

        // Test multiple write operations
        for i in 0..5 {
            let content = format!("Content iteration {}", i);
            let write_result = write_string_to_file(file_path, &content);
            assert!(write_result.is_ok());

            let read_result = read_file_to_string(file_path);
            assert!(read_result.is_ok());
            assert_eq!(read_result.unwrap(), content);
        }
    }

    #[test]
    fn test_file_operations_with_special_characters() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();
        let special_content = "Special chars: !@#$%^&*()[]{}|\\:;\"'<>,.?/~`\n\t\r";

        // Test writing and reading special characters
        let write_result = write_string_to_file(file_path, special_content);
        assert!(write_result.is_ok());

        let read_result = read_file_to_string(file_path);
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), special_content);
    }

    #[test]
    fn test_path_edge_cases() {
        // Test with current directory
        assert!(file_exists("."));

        // Test with parent directory
        assert!(file_exists(".."));

        // Test with home directory shortcut (may not work in all environments)
        if cfg!(unix) {
            // On Unix systems, test some common paths
            assert!(file_exists("/"));
            assert!(file_exists("/tmp"));
        }
    }
}
