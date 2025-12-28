//! Wrapper over get-harness functionality.

use harness_locate::Harness;

/// Adapter for interacting with harnesses.
#[derive(Debug)]
pub struct HarnessAdapter {
    harness: Harness,
}

impl HarnessAdapter {
    /// Create a new harness adapter.
    pub fn new(harness: Harness) -> Self {
        Self { harness }
    }

    /// Get the underlying harness.
    pub fn harness(&self) -> &Harness {
        &self.harness
    }
}
