pub mod git;
pub mod file_system;
pub mod formatting;
pub mod logging;

// Re-export commonly used functions
pub use logging::{log_info, log_warning, log_error, log_success};
pub use formatting::{format_date, format_commit_type, format_release_notes_markdown};
pub use file_system::{file_exists, read_file_to_string, write_string_to_file, find_project_files};
pub use git::{get_repository, get_latest_commits, get_commit_message};