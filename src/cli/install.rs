//! CLI install command implementation.

use std::io::IsTerminal;

use color_eyre::eyre::{eyre, Result};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{GroupMultiSelect, MultiSelect};

use crate::config::{BridleConfig, ProfileManager};
use crate::harness::HarnessConfig;
use crate::install::discovery::{discover_skills, DiscoveryError};
use crate::install::installer::install_skills;
use crate::install::{InstallOptions, InstallTarget};

pub fn run(source: &str, force: bool) -> Result<()> {
    if !std::io::stdin().is_terminal() {
        return Err(eyre!(
            "Interactive mode requires a terminal. Use --help for non-interactive options."
        ));
    }

    let url = normalize_source(source);

    eprintln!("Discovering skills from {}...", url);

    let discovery = discover_skills(&url).map_err(|e| match e {
        DiscoveryError::InvalidUrl(msg) => eyre!("Invalid URL: {}", msg),
        DiscoveryError::FetchError(e) => eyre!("Failed to fetch repository: {}", e),
        DiscoveryError::NoSkillsFound => eyre!("No skills found in repository"),
    })?;

    if discovery.skills.is_empty() && discovery.mcp_servers.is_empty() {
        eprintln!("No skills or MCP servers found in {}", url);
        return Ok(());
    }

    eprintln!(
        "Found {} skill(s) and {} MCP server(s) from {}/{}",
        discovery.skills.len(),
        discovery.mcp_servers.len(),
        discovery.source.owner,
        discovery.source.repo
    );

    let skill_names: Vec<&str> = discovery.skills.iter().map(|s| s.name.as_str()).collect();

    let Some(selected_indices) = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select skills to install (Esc to cancel)")
        .items(&skill_names)
        .defaults(&vec![true; skill_names.len()])
        .interact_opt()?
    else {
        eprintln!("Cancelled");
        return Ok(());
    };

    if selected_indices.is_empty() {
        eprintln!("No skills selected");
        return Ok(());
    }

    let selected_skills: Vec<_> = selected_indices
        .iter()
        .map(|&i| discovery.skills[i].clone())
        .collect();

    let targets = select_targets()?;

    if targets.is_empty() {
        eprintln!("No targets selected");
        return Ok(());
    }

    let options = InstallOptions { force };

    for target in &targets {
        eprintln!("\nInstalling to {}/{}...", target.harness, target.profile);

        let report = install_skills(&selected_skills, target, &options);

        for success in &report.installed {
            eprintln!("  + Installed: {}", success.skill);
        }

        for skip in &report.skipped {
            eprintln!("  = Skipped: {} (already exists)", skip.skill);
        }

        for error in &report.errors {
            eprintln!("  ! Error installing {}: {}", error.skill, error.error);
        }
    }

    eprintln!("\nDone!");
    Ok(())
}

fn normalize_source(source: &str) -> String {
    if source.starts_with("http://") || source.starts_with("https://") {
        source.to_string()
    } else if source.contains('/') && !source.contains(':') {
        format!("https://github.com/{}", source)
    } else {
        source.to_string()
    }
}

fn select_targets() -> Result<Vec<InstallTarget>> {
    use harness_locate::{Harness, HarnessKind};

    let config = BridleConfig::load()?;
    let profiles_dir = BridleConfig::profiles_dir()?;
    let manager = ProfileManager::new(profiles_dir);

    let harness_kinds = [
        HarnessKind::OpenCode,
        HarnessKind::ClaudeCode,
        HarnessKind::Goose,
    ];

    let mut groups: Vec<(String, Vec<String>, Vec<InstallTarget>)> = Vec::new();

    for kind in &harness_kinds {
        let Ok(harness) = Harness::locate(*kind) else {
            continue;
        };
        let harness_id = harness.id();
        let Ok(profiles) = manager.list_profiles(&harness) else {
            continue;
        };

        if profiles.is_empty() {
            continue;
        }

        let active_profile = config.active_profile_for(harness_id);
        let mut labels = Vec::new();
        let mut targets = Vec::new();

        for profile in profiles {
            let is_active = active_profile == Some(profile.as_str());
            let label = if is_active {
                format!("{} (active)", profile)
            } else {
                profile.to_string()
            };
            labels.push(label);
            targets.push(InstallTarget {
                harness: harness_id.to_string(),
                profile,
            });
        }

        groups.push((harness_id.to_string(), labels, targets));
    }

    if groups.is_empty() {
        return Err(eyre!(
            "No profiles found. Create a profile first with: bridle profile create <harness> <name>"
        ));
    }

    let defaults: Vec<Vec<bool>> = groups
        .iter()
        .map(|(_, labels, _)| {
            labels
                .iter()
                .map(|label| label.contains("(active)"))
                .collect()
        })
        .collect();

    let theme = ColorfulTheme::default();
    let mut group_select = GroupMultiSelect::new()
        .with_theme(&theme)
        .with_prompt("Select target profiles (Esc to cancel)")
        .defaults(defaults);

    for (harness_id, labels, _) in &groups {
        let label_refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
        group_select = group_select.group(harness_id, label_refs);
    }

    let Some(selections) = group_select.interact_opt()? else {
        return Ok(Vec::new());
    };

    let mut selected_targets = Vec::new();
    for (group_idx, indices) in selections.iter().enumerate() {
        let (_, _, targets) = &groups[group_idx];
        for &item_idx in indices {
            selected_targets.push(targets[item_idx].clone());
        }
    }

    Ok(selected_targets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_source_handles_shorthand() {
        assert_eq!(
            normalize_source("owner/repo"),
            "https://github.com/owner/repo"
        );
    }

    #[test]
    fn normalize_source_preserves_full_url() {
        let url = "https://github.com/owner/repo";
        assert_eq!(normalize_source(url), url);
    }

    #[test]
    fn normalize_source_preserves_http() {
        let url = "http://example.com/repo";
        assert_eq!(normalize_source(url), url);
    }
}
