use std::fs;
use std::io;
use std::path::Path;

pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn read_file_to_string(path: &str) -> Result<String, io::Error> {
    fs::read_to_string(path)
}

pub fn write_string_to_file(path: &str, content: &str) -> Result<(), io::Error> {
    fs::write(path, content)
}
