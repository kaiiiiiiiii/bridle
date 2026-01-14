# Feature: Expanded Install Sources

> **Status:** Planning  
> **Priority:** High  
> **Complexity:** Medium-High

## Overview

Currently, `bridle install` only accepts GitHub repository URLs. This feature expands the installation sources to support:

- **Local files** (Markdown agents, JSON configs)
- **Local archives** (`.zip`, `.tar.gz`)
- **Direct URLs** (remote files and archives)
- **GitHub Gists** (single-file shares)

This enables offline workflows, private sharing, and quick testing without the overhead of creating full GitHub repositories.

---

## Goals

1. **Source-agnostic discovery** — The scanner should not know or care where data came from
2. **Extensibility** — New source types can be added by implementing a single trait
3. **Minimal core changes** — Existing scanner and installer logic remains largely unchanged
4. **Consistent UX** — Same TUI selection flow regardless of source

---

## Architecture

### The Normalization Layer

The key architectural insight is to introduce a **normalization layer** between input parsing and discovery. All sources—regardless of origin—are materialized into a consistent filesystem structure in a staging directory before the existing scanner runs.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         User Input                                  │
│  "neiii/repo"  |  "./skill.zip"  |  "agent.md"  |  "https://..."   │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       SourceResolver                                │
│           Analyzes input → determines SourceType enum               │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          Fetcher Trait                              │
│   GitHubFetcher | ArchiveFetcher | FileFetcher | UrlFetcher | ...  │
│                                                                     │
│   Each fetcher materializes source into staging directory           │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Staging Directory                              │
│   /tmp/bridle_staging_xxxx/                                         │
│   └── skills/my-skill/SKILL.md                                      │
│   └── agents/code-reviewer/agent.md                                 │
│   └── .mcp.json                                                     │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Scanner (unchanged)                              │
│   Discovers skills, agents, commands, MCPs from filesystem          │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Installer (unchanged)                            │
│   Installs discovered components to harness configs                 │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Supported Source Types

### 1. GitHub Repository (existing)

**Pattern:** `owner/repo`, `https://github.com/owner/repo`

**Current behavior:** Uses `skills-locate` crate to download archive, extract, and scan.

**Change:** Refactor to use the new `Fetcher` trait interface.

### 2. Local Archive

**Pattern:** Path ending in `.zip`, `.tar.gz`, `.tgz`, `.tar.xz`

**Examples:**
```bash
bridle install ./downloads/super-skill.zip
bridle install ~/my-skills-v2.tar.gz
```

**Behavior:**
1. Validate archive exists and is readable
2. Extract to staging directory
3. Run standard discovery on extracted contents

**Best for:** Skills (multi-file), bundled collections, offline distribution

### 3. Local File

**Pattern:** Path to `.md` or `.json` file

**Examples:**
```bash
bridle install ./my-agent.md
bridle install ./mcp-config.json
```

**Behavior:**
1. Detect file type from extension and content
2. Create synthetic directory structure in staging:
   - `.md` file → `staging/agents/{filename}/agent.md`
   - `.json` with `mcpServers` → `staging/.mcp.json`
3. Run standard discovery

**Markdown Agent Format:**
```markdown
---
name: "CodeReviewer"
description: "Strict Rust code reviewer"
model: "claude-3-5-sonnet"
---

# System Prompt

You are an expert Rust developer...
```

### 4. Local Directory

**Pattern:** Path to existing directory

**Examples:**
```bash
bridle install ./my-local-skill/
bridle install ~/dev/work-in-progress-agent
```

**Behavior:**
1. Symlink or copy directory to staging (symlink for dev mode)
2. Run standard discovery

**Best for:** Development/testing workflows without copying

### 5. Direct URL

**Pattern:** HTTP(S) URL to file or archive

**Examples:**
```bash
bridle install https://example.com/agents/architect.md
bridle install https://releases.example.com/skill-pack-v2.zip
```

**Behavior:**
1. Detect type from URL path extension or Content-Type header
2. Download to temp file
3. Delegate to FileFetcher or ArchiveFetcher

### 6. GitHub Gist

**Pattern:** `gist:{id}` or `gist:{user}/{id}`

**Examples:**
```bash
bridle install gist:8f3a7b2c1d4e5f6a
bridle install gist:neiii/8f3a7b2c1d4e5f6a
```

**Behavior:**
1. Use GitHub API to fetch gist files
2. Materialize files in staging directory
3. Run standard discovery

**Best for:** Quick sharing of single agents or configs

---

## Implementation Design

### New Types

```rust
// src/install/sources/mod.rs

/// Identifies the type of installation source
#[derive(Debug, Clone)]
pub enum SourceType {
    /// GitHub repository (owner/repo or full URL)
    GitHub { owner: String, repo: String, git_ref: Option<String> },
    /// Local archive file (.zip, .tar.gz, etc.)
    Archive { path: PathBuf },
    /// Local single file (.md, .json)
    File { path: PathBuf },
    /// Local directory
    Directory { path: PathBuf },
    /// Remote URL (file or archive)
    Url { url: String },
    /// GitHub Gist
    Gist { id: String, user: Option<String> },
}

/// Metadata about the source for manifest tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceMeta {
    GitHub { owner: String, repo: String, git_ref: Option<String> },
    Archive { original_path: String },
    File { original_path: String },
    Directory { original_path: String },
    Url { url: String },
    Gist { id: String },
}

/// Result of fetching a source
pub struct FetchResult {
    /// Path to staging directory with normalized content
    pub staging_path: PathBuf,
    /// Source metadata for manifest
    pub source_meta: SourceMeta,
}
```

### The Fetcher Trait

```rust
// src/install/sources/fetcher.rs

/// Trait for fetching and normalizing installation sources
pub trait Fetcher {
    /// Fetch source content and materialize in staging directory
    ///
    /// # Arguments
    /// * `staging_path` - Empty temp directory to populate
    ///
    /// # Returns
    /// * `FetchResult` with staging path and source metadata
    fn fetch(&self, staging_path: &Path) -> Result<FetchResult, FetchError>;
    
    /// Human-readable description of source for UI
    fn description(&self) -> String;
}

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("Source not found: {0}")]
    NotFound(String),
    
    #[error("Invalid source: {0}")]
    Invalid(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Archive extraction failed: {0}")]
    Extraction(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Fetcher Implementations

#### GitHubFetcher

```rust
// src/install/sources/github.rs

pub struct GitHubFetcher {
    owner: String,
    repo: String,
    git_ref: Option<String>,
}

impl Fetcher for GitHubFetcher {
    fn fetch(&self, staging_path: &Path) -> Result<FetchResult, FetchError> {
        // Reuse existing skills-locate logic
        // Download archive, extract to staging_path
        // Return FetchResult with GitHub source meta
    }
}
```

#### ArchiveFetcher

```rust
// src/install/sources/archive.rs

pub struct ArchiveFetcher {
    path: PathBuf,
}

impl Fetcher for ArchiveFetcher {
    fn fetch(&self, staging_path: &Path) -> Result<FetchResult, FetchError> {
        // Detect archive type from extension
        // Use zip/tar/flate2 crates to extract
        // Handle nested root directories (common in GitHub archives)
    }
}
```

#### FileFetcher

```rust
// src/install/sources/file.rs

pub struct FileFetcher {
    path: PathBuf,
}

impl Fetcher for FileFetcher {
    fn fetch(&self, staging_path: &Path) -> Result<FetchResult, FetchError> {
        // Detect file type from extension + content
        // Create synthetic directory structure:
        //   .md → staging/agents/{name}/agent.md
        //   .json (mcp) → staging/.mcp.json
        //   .json (other) → analyze and place appropriately
    }
}
```

#### UrlFetcher

```rust
// src/install/sources/url.rs

pub struct UrlFetcher {
    url: String,
}

impl Fetcher for UrlFetcher {
    fn fetch(&self, staging_path: &Path) -> Result<FetchResult, FetchError> {
        // Download to temp file
        // Detect type from extension or Content-Type
        // Delegate to ArchiveFetcher or FileFetcher
    }
}
```

### SourceResolver

```rust
// src/install/sources/resolver.rs

pub fn resolve(input: &str) -> Result<Box<dyn Fetcher>, ResolveError> {
    // Priority order for ambiguous inputs:
    
    // 1. Explicit schemes
    if input.starts_with("gist:") { return Ok(GistFetcher::new(...)); }
    if input.starts_with("http://") || input.starts_with("https://") {
        return Ok(UrlFetcher::new(input));
    }
    
    // 2. Local paths (check existence)
    let path = Path::new(input);
    if path.exists() {
        if path.is_dir() { return Ok(DirectoryFetcher::new(path)); }
        if is_archive_extension(path) { return Ok(ArchiveFetcher::new(path)); }
        if is_file_extension(path) { return Ok(FileFetcher::new(path)); }
    }
    
    // 3. GitHub shorthand (owner/repo pattern)
    if looks_like_github_ref(input) {
        return Ok(GitHubFetcher::from_ref(input)?);
    }
    
    Err(ResolveError::UnknownSource(input.to_string()))
}
```

---

## File Detection Logic

### Archive Extensions

| Extension | Handler |
|-----------|---------|
| `.zip` | `zip` crate |
| `.tar.gz`, `.tgz` | `tar` + `flate2` |
| `.tar.xz`, `.txz` | `tar` + `xz2` |
| `.tar.bz2`, `.tbz2` | `tar` + `bzip2` |
| `.tar` | `tar` |

### Single File Detection

| Pattern | Detected As | Staging Structure |
|---------|-------------|-------------------|
| `*.md` with YAML frontmatter | Agent | `agents/{name}/agent.md` |
| `*.md` with `SKILL.md` name | Skill | `skills/{parent}/SKILL.md` |
| `*.json` with `mcpServers` key | MCP Config | `.mcp.json` |
| `*.json` with `command` key | MCP Config (single) | `.mcp.json` (wrapped) |

---

## Modified Discovery Flow

The current `discover_skills()` function in [discovery.rs](discovery.rs) is tightly coupled to GitHub. It needs to be refactored:

### Before (current)
```rust
pub fn discover_skills(url: &str) -> Result<DiscoveryResult, DiscoveryError>
```

### After (new)
```rust
/// Discover from any source (high-level entry point)
pub fn discover(input: &str) -> Result<DiscoveryResult, DiscoveryError> {
    let staging = tempfile::tempdir()?;
    let fetcher = resolve(input)?;
    let fetch_result = fetcher.fetch(staging.path())?;
    discover_from_path(staging.path(), fetch_result.source_meta)
}

/// Discover from filesystem (used by all fetchers)
pub fn discover_from_path(
    path: &Path,
    source: SourceMeta,
) -> Result<DiscoveryResult, DiscoveryError> {
    // Walk filesystem looking for:
    // - SKILL.md files
    // - AGENT.md files or */agents/*.md
    // - COMMAND.md files or */commands/*.md  
    // - .mcp.json files
}

/// Legacy GitHub-only function (deprecated, calls discover())
pub fn discover_skills(url: &str) -> Result<DiscoveryResult, DiscoveryError> {
    discover(url)
}
```

---

## Manifest Changes

The manifest needs to track different source types:

### Current SourceInfo
```rust
pub struct SourceInfo {
    pub owner: String,
    pub repo: String,
    pub git_ref: Option<String>,
}
```

### New SourceInfo
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SourceInfo {
    #[serde(rename = "github")]
    GitHub {
        owner: String,
        repo: String,
        git_ref: Option<String>,
    },
    #[serde(rename = "archive")]
    Archive {
        original_path: String,
    },
    #[serde(rename = "file")]
    File {
        original_path: String,
    },
    #[serde(rename = "directory")]
    Directory {
        original_path: String,
    },
    #[serde(rename = "url")]
    Url {
        url: String,
    },
    #[serde(rename = "gist")]
    Gist {
        id: String,
    },
}
```

This is a **breaking change** to the manifest format. Migration strategy:
1. Add `#[serde(untagged)]` initially to support both formats during transition
2. Or bump manifest version and migrate on first write

---

## New Dependencies

```toml
# Cargo.toml additions

# Archive handling
zip = "2.1"
tar = "0.4"
flate2 = "1.0"

# Optional, for .xz and .bz2 support
xz2 = { version = "0.1", optional = true }
bzip2 = { version = "0.4", optional = true }

[features]
default = []
full-archive = ["xz2", "bzip2"]
```

---

## Module Structure

```
src/install/
├── mod.rs              # Re-exports
├── discovery.rs        # Refactored: uses discover_from_path()
├── installer.rs        # Unchanged
├── manifest.rs         # Updated: new SourceInfo enum
├── types.rs            # Updated: new SourceInfo
│
└── sources/            # NEW MODULE
    ├── mod.rs          # Fetcher trait, SourceType enum, resolve()
    ├── github.rs       # GitHubFetcher (extracted from discovery.rs)
    ├── archive.rs      # ArchiveFetcher (.zip, .tar.gz, etc.)
    ├── file.rs         # FileFetcher (single .md, .json)
    ├── directory.rs    # DirectoryFetcher (local dev)
    ├── url.rs          # UrlFetcher (HTTP downloads)
    └── gist.rs         # GistFetcher (GitHub Gist API)
```

---

## CLI Changes

The `install` command arguments remain the same—just `<SOURCE>`:

```bash
# All of these work with the same command
bridle install neiii/skill-pack
bridle install ./downloaded-skills.zip
bridle install ./my-agent.md
bridle install https://example.com/agent.md
bridle install gist:abc123
```

The CLI layer calls `sources::resolve(source_arg)` and the rest flows through unchanged.

---

## Implementation Order

### Phase 1: Refactor (Foundation)
1. Create `src/install/sources/mod.rs` with `Fetcher` trait
2. Extract GitHub logic into `GitHubFetcher`
3. Create `discover_from_path()` that works on filesystem
4. Ensure existing GitHub flow still works

### Phase 2: Local Sources
1. Implement `ArchiveFetcher` for `.zip`
2. Implement `FileFetcher` for `.md` files
3. Implement `DirectoryFetcher` for local dev
4. Add `tar.gz` support to `ArchiveFetcher`

### Phase 3: Remote Sources
1. Implement `UrlFetcher` with download logic
2. Implement `GistFetcher` using GitHub API
3. Add Content-Type detection for URLs

### Phase 4: Polish
1. Update manifest format with migration
2. Add comprehensive tests
3. Update documentation and help text
4. Add progress indicators for downloads

---

## Testing Strategy

### Unit Tests
- Source resolution for each pattern type
- Archive extraction for each format
- File detection logic
- Synthetic directory creation

### Integration Tests
- Install from local `.zip` file
- Install from local `.md` file
- Install from directory
- Install from URL (mock server)
- Verify manifest correctly tracks source type

### Test Fixtures
Create `tests/fixtures/` with:
- `test-skill.zip` — Valid skill archive
- `test-agent.md` — Valid agent with frontmatter
- `test-mcp.json` — Valid MCP config
- `corrupt.zip` — For error handling tests

---

## Error Handling

| Scenario | Error Type | User Message |
|----------|------------|--------------|
| File not found | `FetchError::NotFound` | "File not found: ./missing.zip" |
| Unsupported format | `FetchError::Invalid` | "Unsupported file type: .xyz" |
| Corrupt archive | `FetchError::Extraction` | "Failed to extract archive: invalid zip" |
| Network failure | `FetchError::Network` | "Failed to download: connection refused" |
| Empty source | `DiscoveryError::NoSkillsFound` | "No skills, agents, or MCPs found" |

---

## Future Considerations

### Not in Scope (v1)
- **npm/pypi registries** — Requires complex runner setup
- **Git clone** (vs archive) — Archive download is simpler and sufficient
- **Authentication** — Private repos/gists would need token handling
- **Caching** — Downloaded archives could be cached, but not in v1

### Potential v2 Features
- `bridle install --dev ./path` — Symlink instead of copy for development
- `bridle install --from-manifest manifest.json` — Batch install from list
- `bridle pack ./skill-dir -o skill.zip` — Create distributable archives

---

## Summary

This feature transforms `bridle install` from a GitHub-only tool into a flexible package manager that accepts any reasonable input format. The key insight is the **normalization layer**—by materializing all sources into a filesystem staging area before discovery, we keep the core scanner and installer logic unchanged while supporting an extensible set of input types.
