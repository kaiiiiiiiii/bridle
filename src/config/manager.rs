//! Profile management.

use std::path::PathBuf;

/// Manages bridle profiles.
#[derive(Debug)]
pub struct ProfileManager {
    /// Directory containing profile files.
    profiles_dir: PathBuf,
}

impl ProfileManager {
    /// Create a new profile manager.
    pub fn new(profiles_dir: PathBuf) -> Self {
        Self { profiles_dir }
    }

    /// Get the profiles directory path.
    pub fn profiles_dir(&self) -> &PathBuf {
        &self.profiles_dir
    }
}
