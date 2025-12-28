//! Harness integration for bridle.

#![allow(dead_code)]
#![allow(unused_imports)]

mod adapter;
mod display;

use std::path::PathBuf;

use get_harness::{InstallationStatus, McpServer, Scope};

use crate::error::Result;

pub use adapter::HarnessAdapter;
pub use display::DisplayInfo;

pub trait HarnessConfig {
    fn id(&self) -> &str;
    fn config_dir(&self) -> Result<PathBuf>;
    fn installation_status(&self) -> Result<InstallationStatus>;
    fn mcp_filename(&self) -> Option<String>;
    fn parse_mcp_servers(&self, content: &str) -> Result<Vec<String>>;
}

impl HarnessConfig for get_harness::Harness {
    fn id(&self) -> &'static str {
        match self.kind() {
            get_harness::HarnessKind::ClaudeCode => "claude-code",
            get_harness::HarnessKind::OpenCode => "opencode",
            get_harness::HarnessKind::Goose => "goose",
            _ => "unknown",
        }
    }

    fn config_dir(&self) -> Result<PathBuf> {
        Ok(self.config(&Scope::Global)?)
    }

    fn installation_status(&self) -> Result<InstallationStatus> {
        Ok(get_harness::Harness::installation_status(self)?)
    }

    fn mcp_filename(&self) -> Option<String> {
        self.mcp(&Scope::Global)
            .ok()
            .flatten()
            .map(|r| r.file)
            .and_then(|f| f.file_name().map(|n| n.to_os_string()))
            .and_then(|n| n.into_string().ok())
    }

    fn parse_mcp_servers(&self, content: &str) -> Result<Vec<String>> {
        let parsed: serde_json::Value = serde_json::from_str(content)?;
        let servers: std::collections::HashMap<String, McpServer> =
            self.parse_mcp_config(&parsed)?;
        let mut names: Vec<String> = servers.keys().cloned().collect();
        names.sort();
        Ok(names)
    }
}
