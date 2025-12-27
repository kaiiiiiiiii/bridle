//! CLI subcommand definitions.

use clap::Subcommand;

/// Available CLI subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show status of all harnesses.
    Status,

    /// List available profiles.
    List,

    /// Show details of a specific profile.
    Show {
        /// Profile name to show.
        name: String,
    },

    /// Apply a profile (activate its configuration).
    Apply {
        /// Profile name to apply.
        name: String,
    },
}
