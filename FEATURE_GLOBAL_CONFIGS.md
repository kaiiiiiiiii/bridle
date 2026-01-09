# Global Configurations for Profiles

## Summary

Add support for global configurations (agents, skills, MCPs) that can be shared across multiple profiles and harnesses. Changes propagate automatically, with local profile configs always taking precedence over globals.

## Core Principle

```
Activation = Global configs (base layer) + Local profile configs (overlay wins)
```

If a profile has a local config for "Skill A", that local version wins over any global "Skill A".

## Architecture

### Global Directory Structure

```
~/.config/bridle/globals/
├── mcp/
│   ├── github.yaml
│   ├── filesystem.yaml
│   └── postgres.yaml
├── agents/
│   ├── code-reviewer.yaml
│   └── pr-reviewer.yaml
└── skills/
    ├── refactor-helper.yaml
    └── test-generator.yaml
```

### Profile Structure (unchanged)

```
~/.config/bridle/profiles/
├── claude-code/
│   ├── default/
│   │   ├── .mcp.json          # Merged: local wins over global
│   │   ├── agents/            # Local agent overrides
│   │   ├── skills/            # Local skill overrides
│   │   └── profile.yaml
│   └── work/
│       └── ...
├── opencode/
│   └── ...
└── goose/
    └── ...
```

### Profile YAML Extension

```yaml
# profile.yaml
name: default
globals:
  mcp:
    - github
    - filesystem
  agents:
    - code-reviewer
  skills:
    - refactor-helper
```

### Global Config Format (Canonical)

Globals are stored in canonical format; bridle transforms to harness-specific format on activation.

```yaml
# ~/.config/bridle/globals/mcp/github.yaml
name: github
type: stdio  # stdio | sse | http | streamable_http
command: uvx
args:
  - mcp-server-github
env:
  GITHUB_TOKEN: ${GITHUB_TOKEN}
  # Other env vars...
```

```yaml
# ~/.config/bridle/globals/agents/code-reviewer.yaml
identifier: code-quality-reviewer
whenToUse: |
  Use this agent when the user has written code and needs quality review.
systemPrompt: |
  You are an expert code quality reviewer specializing in identifying issues.
tools:
  - Read
  - Grep
  - Glob
```

```yaml
# ~/.config/bridle/globals/skills/refactor-helper.yaml
name: refactor-helper
description: Helps refactor code while maintaining behavior
tools:
  - Read
  - Write
  - Grep
  - Bash
```

## Config Merge Behavior

### MCP Servers

| Scenario | Result |
|----------|--------|
| Global defines "github", local not defined | Global "github" applied |
| Global defines "github", local also defines "github" | Local "github" wins |
| Global defines "github", local defines "filesystem" | Both applied (merged) |
| Local defines "github", global not defined | Local "github" only |

### Agents

Same merge behavior - local overrides win when names match.

### Skills

Same merge behavior - local overrides win when names match.

## Commands

### Enhanced Existing Commands

```bash
# Install now supports --global flag
bridle install gh/mcp-server-github --global
bridle install gh/refactor-skill --global

# Creates global config in ~/.config/bridle/globals/
# Does NOT attach to any profile
```

```bash
# Uninstall now supports --global flag
bridle uninstall --global github  # Remove from globals, warn if profiles reference it
```

### New Commands

```bash
# Attach globals to a profile
bridle profile attach-global claude-code default --mcp github --mcp filesystem
bridle profile attach-global claude-code default --agents code-reviewer
bridle profile attach-global claude-code default --all

# Detach globals from a profile
bridle profile detach-global claude-code default --mcp github
bridle profile detach-global claude-code default --all

# List globals attached to a profile
bridle profile globals claude-code default

# Refresh profiles (manual trigger)
bridle refresh                    # Refresh all profiles with attached globals
bridle refresh claude-code        # Refresh only claude-code profiles
bridle refresh --global github    # Refresh all profiles using github global
```

## Propagation Settings

### Default: Immediate

Changes to globals propagate immediately to active profiles.

### Configuration

```toml
# ~/.config/bridle/config.toml
[globals]
propagation = "immediate"  # immediate | on-refresh | manual

[globals.immediate]
auto_reload = false  # Whether to reload harness after config change
confirm_each = true  # Ask before refreshing each profile
```

### Behavior

| Mode | On Global Change |
|------|------------------|
| `immediate` | Auto-copy to all profiles, optionally reload harness |
| `on-refresh` | Copy when `bridle refresh` runs |
| `manual` | Copy only when `bridle refresh --global <name>` runs |

## Error Handling

### Global Deletion

When a global is deleted:
1. Bridle removes it from all profile `globals.yaml` attachments
2. Warns user: "Removed global 'github' from 3 profiles"
3. Profiles retain local copies (if any)

### Missing Global on Activation

When activating a profile that references a missing global:
1. Warning: "Global 'github' not found, skipping"
2. Profile activates with other globals
3. Error logged, not blocking

### Malformed Global Config

1. Validation error on install/import
2. Warning on activation: "Global 'xyz' is malformed, skipping"
3. Error details in `bridle status`

## TUI Integration

### Dashboard - New Globals Tab

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  bridle v0.2.5                              Profile: claude-code  [default]│
├─────────────────────────────────────────────────────────────────────────────┤
│  Profiles  │  MCP Servers  │  Config  │  Globals  │  Skills                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─ Global MCP Servers ───────────────────────────────────────────────────┐  │
│  │  github        [●]  stdio  │  uvx mcp-server-github        [Edit] [X]  │  │
│  │  filesystem    [●]  stdio  │  npx -y @modelcontextplugin/  [Edit] [X]  │  │
│  │  postgres      [●]  sse    │  https://postgres-mcp.io      [Edit] [X]  │  │
│  │  redis         [ ]  sse    │  https://redis-mcp.example.co [Edit] [X]  │  │
│  │                                                                      │   │
│  │  [+ Add MCP]                                                         │   │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─ Global Agents ────────────────────────────────────────────────────────┐  │
│  │  code-quality   [●]  identifier: code-quality-reviewer   [Edit] [X]  │  │
│  │  pr-reviewer    [●]  identifier: pr-quality-reviewer     [Edit] [X]  │  │
│  │                                                                      │   │
│  │  [+ Add Agent]                                                         │   │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─ Global Skills ────────────────────────────────────────────────────────┐  │
│  │  refactor-help  [●]  tools: Read, Write, Grep, Bash     [Edit] [X]  │  │
│  │  test-gen       [ ]  tools: Read, Write, Glob           [Edit] [X]  │  │
│  │                                                                      │   │
│  │  [+ Add Skill]                                                         │   │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  [E]dit  [D]elete  [+Add]  [I]mport  [?] Help                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Profile List - Global/Local Split

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Profiles: claude-code                                    [+ New Profile]  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  🔷 GLOBAL PROFILES                                                         │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  📦 github-mcp      [●]  MCP: github, filesystem                      │  │
│  │  📦 full-stack      [ ]  MCP: github, postgres, redis                 │  │
│  │                      Agents: code-quality, pr-reviewer                │  │
│  │                                                                      │   │
│  │  [Activate]  [E]dit Name  [D]etach from Globals                       │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  🏠 LOCAL PROFILES                                                          │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  ● default           [●]  MCP: github (local), filesystem (global)    │  │
│  │  ○ work              [ ]  MCP: github (global)                        │  │
│  │  ○ debug             [ ]  MCP: github (global), filesystem (global)   │  │
│  │                                                                      │   │
│  │  [Activate]  [E]dit  [D]elete  [S]witch                              │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  [G]lobal Profiles  [L]ocal Profiles  [A]ll  [Tab] Switch View             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Profile Details - Globals Indicator

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Profile: claude-code/default  ● active                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─ Summary ─────────────────────────────────────────────────────────────┐  │
│  │ Harness: Claude Code  │ Globals: 3 attached                           │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─ Globals ──────────────────────────────────────────────────────────────┐  │
│  │                                                                      │   │
│  │  MCP Servers (2 attached)                                            │   │
│  │  ● github        [L] Local override present        [View] [Edit]    │   │
│  │  ● filesystem    [G] Global only                    [View] [Edit]    │   │
│  │                                                                      │   │
│  │  Agents (1 attached)                                                 │   │
│  │  ● code-review   [L] Local override present        [View] [Edit]    │   │
│  │                                                                      │   │
│  │  Skills (0 attached)                                                 │   │
│  │  [+ Add Global Skill]                                               │   │
│  │                                                                      │   │
│  ├──────────────────────────────────────────────────────────────────────┤   │
│  │  [L] = Local override wins  [G] = Global only                        │   │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  [Tab] Switch View  [E]dit Profile  [+Add Global]  [R]efresh  [?] Help     │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Legend Icons

| Icon | Meaning |
|------|---------|
| `●` | Active profile |
| `○` | Inactive profile |
| `📦` | Global profile |
| `🏠` | Local profile |
| `[L]` | Local override exists (local wins) |
| `[G]` | Global only (no local override) |
| `[!]` | Error / needs attention |
| `[X]` | Detach / delete |

### Change Notification

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  ⚠️  Global Config Changed                                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Global "github.yaml" was modified at 2:34 PM                              │
│                                                                             │
│  Affects 3 profiles:                                                        │
│    • claude-code/default (active)                                          │
│    • claude-code/work                                                     │
│    • opencode/dev                                                         │
│                                                                             │
│  ┌─ Actions ─────────────────────────────────────────────────────────────┐   │
│  │  [R]efresh all affected profiles                                     │   │
│  │  [S]kip (apply on next manual refresh)                               │   │
│  │  [A]lways refresh automatically for this global                      │   │
│  │                                                                      │   │
│  └────────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  [R]efresh  [S]kip  [A]lways  [Esc] Dismiss                                │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Implementation Phases

### Phase 1: Foundation
- [ ] Global config directory structure
- [ ] GlobalConfigManager module
- [ ] Canonical format for MCP/agents/skills
- [ ] `bridle install --global` command
- [ ] `bridle profile attach-global` command
- [ ] `bridle profile detach-global` command

### Phase 2: Merge Logic
- [ ] Config merge on activation
- [ ] Local-wins overlay behavior
- [ ] Profile validation for missing globals
- [ ] `bridle refresh` command

### Phase 3: Propagation
- [ ] File watcher for globals directory
- [ ] Immediate propagation mode
- [ ] Configurable propagation settings
- [ ] Change notification UI

### Phase 4: TUI
- [ ] New Globals tab in dashboard
- [ ] Global/local split in profile list
- [ ] Globals indicator in profile details
- [ ] Add/remove global workflows

### Phase 5: Polish
- [ ] Error handling and warnings
- [ ] Validation on import/install
- [ ] Documentation and examples
- [ ] First-run initialization wizard

## Out of Scope (For This Feature)

- Real-time sync between machines (use git sync feature)
- Global-to-global dependencies
- Conflict resolution UI (simple overwrite for now)
- Import/export of global configs (future)
- Template library for common globals (future)

## References

- Existing profile structure: `src/config/`
- MCP configuration: `src/install/mcp_config.rs`
- TUI implementation: `src/tui/`
