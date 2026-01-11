# Cross-Harness Profile Copy

## Summary

Add the ability to copy profiles between different harnesses (e.g., OpenCode to Claude Code, Claude Code to Goose). The feature transforms configurations to match target harness formats while preserving functionality, with user control over what gets copied.

## Motivation

- **Try new harnesses**: Quickly test a new AI assistant with your existing configuration
- **Team standardization**: Share a common configuration across teams using different tools
- **Migration**: Move to a different harness without manual reconfiguration
- **Backup/restore**: Copy working configs between harnesses as a form of redundancy

## Architecture

### Copy Flow

```
Source Profile                    Target Profile
(OpenCode)                        (Claude Code)
                                  
~/.config/bridle/profiles/        ~/.config/bridle/profiles/
  opencode/                         claude-code/
    work/                             work-copy/
      opencode.jsonc     ──────►        .mcp.json
      skill/             ──────►        skills/
      command/           ──────►        (not copied - unsupported)
      agent/             ──────►        (not copied - unsupported)
```

### Transformation Pipeline

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Read Source    │────►│  Normalize to   │────►│  Write Target   │
│  (harness fmt)  │     │  Canonical Fmt  │     │  (harness fmt)  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
     JSONC/JSON              Internal              JSON/YAML
     YAML                    structs               JSONC
```

### Canonical Internal Format

All configurations pass through a harness-agnostic intermediate representation:

```rust
pub struct CanonicalProfile {
    pub mcp_servers: Vec<CanonicalMcpServer>,
    pub skills: Vec<CanonicalSkill>,
    pub commands: Vec<CanonicalCommand>,
    pub agents: Vec<CanonicalAgent>,
    pub model: Option<String>,
    pub theme: Option<String>,
}

pub struct CanonicalMcpServer {
    pub name: String,
    pub enabled: bool,
    pub server_type: McpServerType,  // Stdio, Sse, Http, StreamableHttp
    pub command: Option<String>,
    pub args: Vec<String>,
    pub url: Option<String>,
    pub env: HashMap<String, String>,
}

pub struct CanonicalSkill {
    pub name: String,
    pub description: String,
    pub content: String,  // Full skill file content
}

pub struct CanonicalAgent {
    pub identifier: String,
    pub when_to_use: String,
    pub system_prompt: String,
    pub tools: Vec<String>,
}

pub struct CanonicalCommand {
    pub name: String,
    pub description: String,
    pub content: String,
}
```

## Harness Compatibility Matrix

### MCP Servers

| Source ↓ / Target → | Claude Code | OpenCode | Goose | AMP Code |
|---------------------|-------------|----------|-------|----------|
| **Claude Code**     | -           | ✓        | ✓     | ✓        |
| **OpenCode**        | ✓           | -        | ✓     | ✓        |
| **Goose**           | ✓           | ✓        | -     | ✓        |
| **AMP Code**        | ✓           | ✓        | ✓     | -        |

All MCP server types (stdio, sse, http) are portable across harnesses.

### Skills

| Source ↓ / Target → | Claude Code | OpenCode | Goose | AMP Code |
|---------------------|-------------|----------|-------|----------|
| **Claude Code**     | -           | ✓*       | ✗     | ✓        |
| **OpenCode**        | ✓           | -        | ✗     | ✓        |
| **Goose**           | n/a         | n/a      | -     | n/a      |
| **AMP Code**        | ✓           | ✓*       | ✗     | -        |

`*` = Requires name sanitization (lowercase, hyphenated)

### Agents

| Source ↓ / Target → | Claude Code | OpenCode | Goose | AMP Code |
|---------------------|-------------|----------|-------|----------|
| **Claude Code**     | -           | ✓*       | ✗     | ✗        |
| **OpenCode**        | ✓           | -        | ✗     | ✗        |
| **Goose**           | n/a         | n/a      | -     | n/a      |
| **AMP Code**        | n/a         | n/a      | n/a   | -        |

`*` = Requires field transformation (tools format, color field)

### Commands

| Source ↓ / Target → | Claude Code | OpenCode | Goose | AMP Code |
|---------------------|-------------|----------|-------|----------|
| **Claude Code**     | -           | ✓*       | ✗     | ✓        |
| **OpenCode**        | ✓           | -        | ✗     | ✓        |
| **Goose**           | n/a         | n/a      | -     | n/a      |
| **AMP Code**        | ✓           | ✓*       | ✗     | -        |

`*` = Requires name sanitization

## CLI Commands

### Primary Command

```bash
bridle profile copy <source-harness> <source-profile> <target-harness> [target-profile]
```

### Examples

```bash
# Copy entire profile (all supported resources)
bridle profile copy opencode work claude-code work-copy

# Copy to same-named profile in target harness
bridle profile copy opencode work claude-code
# Creates: claude-code/work

# Copy only MCP servers
bridle profile copy opencode work claude-code --mcp-only

# Copy MCP + skills (no agents/commands)
bridle profile copy opencode work claude-code --include mcp,skills

# Exclude specific resource types
bridle profile copy opencode work claude-code --exclude agents,commands

# Force overwrite existing profile
bridle profile copy opencode work claude-code work --force

# Dry run - show what would be copied
bridle profile copy opencode work claude-code --dry-run

# Interactive mode - select what to copy
bridle profile copy opencode work claude-code --interactive
```

### Command Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `source-harness` | Yes | Source harness ID (opencode, claude-code, goose, amp-code) |
| `source-profile` | Yes | Name of the profile to copy from |
| `target-harness` | Yes | Target harness ID |
| `target-profile` | No | Name for the new profile (defaults to source name) |

### Command Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--mcp-only` | `-m` | Copy only MCP server configurations |
| `--include <types>` | `-i` | Comma-separated list: mcp,skills,agents,commands,settings |
| `--exclude <types>` | `-e` | Comma-separated list of types to skip |
| `--force` | `-f` | Overwrite target profile if it exists |
| `--dry-run` | `-n` | Show what would be copied without making changes |
| `--interactive` | | Interactive selection of resources to copy |
| `--no-transform` | | Skip transformations (copy raw, may break target) |

### Output Examples

**Standard copy:**
```
Copying profile: opencode/work → claude-code/work-copy

  MCP Servers:
    ✓ github (stdio)
    ✓ filesystem (stdio)
    ✓ postgres (sse)

  Skills:
    ✓ refactor-helper → refactor-helper
    ✓ Code Review Guide → code-review-guide (name sanitized)

  Agents:
    ⚠ code-reviewer (transformed: tools format adjusted)

  Commands:
    ✗ Skipped (not supported by claude-code)

  Settings:
    ✓ model: claude-3.5-sonnet
    ⚠ theme: Skipped (not portable)

Profile copied successfully!
Run 'bridle profile switch claude-code work-copy' to activate.
```

**Dry run:**
```
[DRY RUN] Would copy profile: opencode/work → claude-code/work-copy

  Would copy:
    • 3 MCP servers
    • 2 skills (1 requires name sanitization)
    • 1 agent (requires transformation)

  Would skip:
    • 2 commands (unsupported by target)
    • theme setting (not portable)

  Target profile does not exist, would be created.

No changes made. Remove --dry-run to execute.
```

**Interactive mode:**
```
Copying profile: opencode/work → claude-code/work-copy

Select resources to copy:

  MCP Servers:
    [x] github
    [x] filesystem
    [ ] postgres (disabled in source)

  Skills:
    [x] refactor-helper
    [x] Code Review Guide → code-review-guide

  Agents:
    [x] code-reviewer (will transform tools format)

  Settings:
    [x] model
    [ ] theme (not portable)

  [Space] Toggle  [a] All  [n] None  [Enter] Confirm  [Esc] Cancel
```

## Module Structure

```
src/
├── copy/
│   ├── mod.rs              # Public API: copy_profile()
│   ├── canonical.rs        # Canonical format types
│   ├── extract.rs          # Extract from source profile
│   ├── transform.rs        # Transform between formats
│   ├── write.rs            # Write to target profile
│   └── compatibility.rs    # Compatibility checks
├── cli/
│   └── profile.rs          # Add copy subcommand (existing file)
```

## Implementation Details

### Extraction (Source → Canonical)

```rust
pub fn extract_canonical(
    source_harness: &dyn HarnessConfig,
    profile_path: &Path,
) -> Result<CanonicalProfile> {
    let mcp_servers = extract_mcp_servers(source_harness, profile_path)?;
    let skills = extract_skills(source_harness, profile_path)?;
    let agents = extract_agents(source_harness, profile_path)?;
    let commands = extract_commands(source_harness, profile_path)?;
    let settings = extract_settings(source_harness, profile_path)?;

    Ok(CanonicalProfile {
        mcp_servers,
        skills,
        agents,
        commands,
        model: settings.model,
        theme: settings.theme,
    })
}
```

### MCP Server Extraction

Leverages existing `read_mcp_config()` from `src/install/mcp_config.rs`:

```rust
fn extract_mcp_servers(
    harness: &dyn HarnessConfig,
    profile_path: &Path,
) -> Result<Vec<CanonicalMcpServer>> {
    let config_path = get_profile_config_path(profile_path, harness.kind());
    let servers = read_mcp_config(&config_path, harness.kind())?;

    servers
        .into_iter()
        .map(|(name, value)| CanonicalMcpServer::from_json(&name, &value))
        .collect()
}
```

### Transformation (Canonical → Target Format)

```rust
pub fn transform_for_harness(
    canonical: &CanonicalProfile,
    target_harness: HarnessKind,
    options: &CopyOptions,
) -> Result<TransformResult> {
    let mut result = TransformResult::new();

    // MCP servers - always portable
    for server in &canonical.mcp_servers {
        result.mcp_servers.push(transform_mcp_server(server, target_harness)?);
    }

    // Skills - may require name sanitization
    for skill in &canonical.skills {
        match transform_skill(skill, target_harness) {
            Ok(transformed) => result.skills.push(transformed),
            Err(e) => result.warnings.push(format!("Skill '{}': {}", skill.name, e)),
        }
    }

    // Agents - check compatibility
    if supports_agents(target_harness) {
        for agent in &canonical.agents {
            match transform_agent(agent, target_harness) {
                Ok(transformed) => result.agents.push(transformed),
                Err(e) => result.warnings.push(format!("Agent '{}': {}", agent.identifier, e)),
            }
        }
    } else {
        result.skipped.push("agents (not supported by target harness)".into());
    }

    // Commands - check compatibility
    if supports_commands(target_harness) {
        for cmd in &canonical.commands {
            match transform_command(cmd, target_harness) {
                Ok(transformed) => result.commands.push(transformed),
                Err(e) => result.warnings.push(format!("Command '{}': {}", cmd.name, e)),
            }
        }
    } else {
        result.skipped.push("commands (not supported by target harness)".into());
    }

    Ok(result)
}
```

### Name Sanitization for OpenCode

Uses existing `sanitize_name_for_opencode()` from `src/install/installer.rs`:

```rust
fn transform_skill_for_opencode(skill: &CanonicalSkill) -> Result<TransformedSkill> {
    let sanitized_name = sanitize_name_for_opencode(&skill.name);
    let transformed_content = transform_skill_frontmatter(&skill.content, &sanitized_name)?;

    Ok(TransformedSkill {
        name: sanitized_name,
        content: transformed_content,
        original_name: if skill.name != sanitized_name {
            Some(skill.name.clone())
        } else {
            None
        },
    })
}
```

### Writing to Target

Leverages existing `write_mcp_config()` from `src/install/mcp_config.rs`:

```rust
pub fn write_to_profile(
    target_harness: &dyn HarnessConfig,
    profile_path: &Path,
    transformed: &TransformResult,
) -> Result<()> {
    // Create profile directory
    fs::create_dir_all(profile_path)?;

    // Write MCP servers
    let config_path = get_profile_config_path(profile_path, target_harness.kind());
    let mcp_map = transformed.mcp_servers
        .iter()
        .map(|s| (s.name.clone(), s.to_json()))
        .collect();
    write_mcp_config(&config_path, &mcp_map, target_harness.kind())?;

    // Write skills
    let skills_dir = get_skills_dir(profile_path, target_harness.kind());
    for skill in &transformed.skills {
        write_skill(&skills_dir, skill)?;
    }

    // Write agents (if supported)
    if !transformed.agents.is_empty() {
        let agents_dir = get_agents_dir(profile_path, target_harness.kind());
        for agent in &transformed.agents {
            write_agent(&agents_dir, agent, target_harness.kind())?;
        }
    }

    Ok(())
}
```

## TUI Integration

### Profile Context Menu

Add "Copy to..." option in profile actions:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Profile: opencode/work                                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Actions:                                                                   │
│    [S]witch to this profile                                                │
│    [E]dit profile                                                          │
│    [C]opy to another harness...                                            │
│    [D]elete profile                                                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Copy Wizard Flow

**Step 1: Select Target Harness**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Copy Profile: opencode/work (1/3)                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Select target harness:                                                     │
│                                                                             │
│    ▶ Claude Code    ~/.claude/                                             │
│      Goose          ~/.config/goose/                                       │
│      AMP Code       ~/.amp/                                                │
│                                                                             │
│  Source profile contains:                                                   │
│    • 5 MCP servers                                                         │
│    • 3 skills                                                              │
│    • 2 agents                                                              │
│    • 1 command                                                             │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  [↑/↓] Navigate  [Enter] Select  [Esc] Cancel                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Step 2: Select Resources**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Copy Profile: opencode/work (2/3)                        │
│                    Target: claude-code                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Select what to copy:                                                       │
│                                                                             │
│  MCP Servers (5 available, all compatible)                                  │
│    [x] github          stdio                                               │
│    [x] filesystem      stdio                                               │
│    [x] postgres        sse                                                 │
│    [x] redis           sse                                                 │
│    [ ] local-dev       stdio (disabled)                                    │
│                                                                             │
│  Skills (3 available, all compatible)                                       │
│    [x] refactor-helper                                                     │
│    [x] Code Review     → code-review (will be renamed)                     │
│    [x] test-gen                                                            │
│                                                                             │
│  Agents (2 available, all compatible)                                       │
│    [x] code-reviewer   (tools format will be adjusted)                     │
│    [x] pr-helper                                                           │
│                                                                             │
│  Commands (1 available, NOT compatible)                                     │
│    [ ] deploy-script   ⚠ Not supported by Claude Code                      │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  [Space] Toggle  [a] All  [Enter] Continue  [Esc] Back                     │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Step 3: Confirm**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Copy Profile: opencode/work (3/3)                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Profile name in claude-code:                                               │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ work█                                                                  │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  Summary:                                                                   │
│    Source:  opencode/work                                                  │
│    Target:  claude-code/work                                               │
│                                                                             │
│    Will copy:                                                              │
│      • 4 MCP servers                                                       │
│      • 3 skills (1 will be renamed)                                        │
│      • 2 agents (tools format adjusted)                                    │
│                                                                             │
│    Will skip:                                                              │
│      • 1 MCP server (disabled)                                             │
│      • 1 command (not supported)                                           │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  [Enter] Create Profile  [Esc] Back                                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Success:**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Profile Copied Successfully                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ✓ Created claude-code/work                                                │
│                                                                             │
│  Copied:                                                                    │
│    ✓ 4 MCP servers                                                         │
│    ✓ 3 skills                                                              │
│    ✓ 2 agents                                                              │
│                                                                             │
│  Transformations applied:                                                   │
│    • Skill "Code Review" → "code-review" (name sanitized)                  │
│    • Agent tools converted to Claude Code format                           │
│                                                                             │
│  ┌─ Next Steps ─────────────────────────────────────────────────────────┐   │
│  │  • Run 'bridle profile switch claude-code work' to activate         │   │
│  │  • Or press [S] to switch now                                       │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  [S]witch to profile  [Enter] Done                                         │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum CopyError {
    #[error("Source profile not found: {harness}/{profile}")]
    SourceNotFound { harness: String, profile: String },

    #[error("Target profile already exists: {harness}/{profile}. Use --force to overwrite")]
    TargetExists { harness: String, profile: String },

    #[error("Target harness not installed: {0}")]
    TargetHarnessNotInstalled(String),

    #[error("No compatible resources to copy")]
    NothingToCopy,

    #[error("Transformation failed for {resource_type} '{name}': {reason}")]
    TransformFailed {
        resource_type: String,
        name: String,
        reason: String,
    },

    #[error("Failed to read source profile: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to write target profile: {0}")]
    WriteError(String),
}
```

### Error Messages

| Error | User Message | Recovery |
|-------|--------------|----------|
| Source not found | "Profile 'opencode/work' does not exist" | List available profiles |
| Target exists | "Profile 'claude-code/work' already exists. Use --force to overwrite" | Suggest --force flag |
| Target harness not installed | "Claude Code is not installed. Install it first or choose another target" | Show install instructions |
| Nothing to copy | "No resources could be copied (all incompatible with target)" | Show compatibility matrix |
| Transform failed | "Could not transform skill 'X': invalid frontmatter" | Skip and continue, warn user |

### Warnings (Non-Fatal)

```
⚠ Warning: 2 resources could not be copied:
  • Command 'deploy-script': Not supported by claude-code
  • Skill 'legacy-helper': Invalid frontmatter format

The remaining 8 resources were copied successfully.
```

## Implementation Phases

### Phase 1: Core Infrastructure
- [ ] Define canonical format types in `src/copy/canonical.rs`
- [ ] Implement MCP server extraction (reuse `read_mcp_config`)
- [ ] Implement MCP server writing (reuse `write_mcp_config`)
- [ ] Add `profile copy` CLI subcommand (MCP-only mode)
- [ ] Basic error handling

### Phase 2: Resource Transformation
- [ ] Skill extraction from all harnesses
- [ ] Skill transformation (name sanitization, frontmatter)
- [ ] Skill writing to target formats
- [ ] Agent extraction and transformation
- [ ] Command extraction and transformation
- [ ] Compatibility checking

### Phase 3: CLI Polish
- [ ] `--include` and `--exclude` flags
- [ ] `--dry-run` mode
- [ ] `--interactive` mode with dialoguer
- [ ] Rich output with colors and status indicators
- [ ] JSON output format support

### Phase 4: TUI Integration
- [ ] "Copy to..." in profile context menu
- [ ] Three-step copy wizard
- [ ] Resource selection interface
- [ ] Success/failure result view

### Phase 5: Edge Cases & Testing
- [ ] Handle disabled MCP servers
- [ ] Handle missing optional fields
- [ ] Handle malformed source configs
- [ ] Integration tests with temp directories
- [ ] CLI tests with assert_cmd

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_canonical_roundtrip() {
        let original = json!({
            "command": "uvx",
            "args": ["mcp-server-github"],
            "env": { "GITHUB_TOKEN": "xxx" }
        });

        let canonical = CanonicalMcpServer::from_json("github", &original).unwrap();
        let restored = canonical.to_json_for_harness(HarnessKind::ClaudeCode);

        assert_eq!(restored["command"], "uvx");
        assert_eq!(restored["args"][0], "mcp-server-github");
    }

    #[test]
    fn test_skill_name_sanitization() {
        let skill = CanonicalSkill {
            name: "Code Review Helper".into(),
            description: "Helps with reviews".into(),
            content: "# Skill content".into(),
        };

        let transformed = transform_skill(&skill, HarnessKind::OpenCode).unwrap();
        assert_eq!(transformed.name, "code-review-helper");
    }

    #[test]
    fn test_compatibility_check() {
        assert!(supports_agents(HarnessKind::OpenCode));
        assert!(supports_agents(HarnessKind::ClaudeCode));
        assert!(!supports_agents(HarnessKind::Goose));
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration {
    use tempfile::TempDir;

    #[test]
    fn test_copy_mcp_opencode_to_claude() {
        let temp = TempDir::new().unwrap();

        // Create source profile
        let source_dir = temp.path().join("profiles/opencode/work");
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(
            source_dir.join("opencode.jsonc"),
            r#"{ "mcp": { "github": { "command": "uvx", "args": ["mcp-server-github"] } } }"#,
        ).unwrap();

        // Copy
        let result = copy_profile(
            HarnessKind::OpenCode, "work",
            HarnessKind::ClaudeCode, "work",
            &CopyOptions::default(),
        );

        assert!(result.is_ok());

        // Verify target
        let target_path = temp.path().join("profiles/claude-code/work/.mcp.json");
        assert!(target_path.exists());

        let content = fs::read_to_string(target_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed["mcpServers"]["github"].is_object());
    }
}
```

### CLI Tests

```rust
#[test]
fn test_copy_command_dry_run() {
    Command::cargo_bin("bridle")
        .unwrap()
        .args(&["profile", "copy", "opencode", "work", "claude-code", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("[DRY RUN]"))
        .stdout(contains("No changes made"));
}

#[test]
fn test_copy_command_source_not_found() {
    Command::cargo_bin("bridle")
        .unwrap()
        .args(&["profile", "copy", "opencode", "nonexistent", "claude-code"])
        .assert()
        .failure()
        .stderr(contains("Profile 'opencode/nonexistent' does not exist"));
}
```

## Dependencies

No new dependencies required. Uses existing:
- `serde_json` - JSON parsing/writing
- `serde_yaml` - YAML parsing/writing (Goose)
- `dialoguer` - Interactive selection (already in use)
- `harness_locate` - Harness detection and paths

## References

- MCP config reading: `src/install/mcp_config.rs:36-83`
- MCP config writing: `src/install/mcp_config.rs:85-134`
- MCP key mapping: `src/install/mcp_config.rs:26-34`
- Name sanitization: `src/install/installer.rs:59-68`
- Skill transformation: `src/config/manager/files.rs:191-237`
- Profile structure: `src/config/manager/mod.rs`
- CLI commands: `src/cli/commands.rs`
- Harness trait: `src/harness/mod.rs:22-42`
