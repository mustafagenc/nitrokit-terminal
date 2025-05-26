pub mod release_notes;
pub mod dependency_update;

pub use release_notes::generate_release_notes;
pub use dependency_update::update_dependencies;