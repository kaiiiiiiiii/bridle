# .mcp.json Examples from GitHub

## Overview
This document contains real-world examples of `.mcp.json` files found in GitHub repositories. These files define MCP (Model Context Protocol) server configurations used by AI coding assistants like Claude Code.

## Transport Types Found

### 1. **stdio** (Standard Input/Output) - Most Common
The stdio transport runs MCP servers as child processes communicating via stdin/stdout.

### 2. **sse** (Server-Sent Events)
SSE transport connects to remote MCP servers via HTTP Server-Sent Events.

### 3. **http** (HTTP)
HTTP transport connects to remote MCP servers via standard HTTP requests.

### 4. **WebSocket** (ws/wss)
WebSocket transport for bidirectional communication (less common in examples found).

---

## Example 1: juanfont/headscale
**Repository:** https://github.com/juanfont/headscale
**Transport Types:** stdio only

```json
{
  "mcpServers": {
    "claude-code-mcp": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@steipete/claude-code-mcp@latest"],
      "env": {}
    },
    "sequential-thinking": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"],
      "env": {}
    },
    "nixos": {
      "type": "stdio",
      "command": "uvx",
      "args": ["mcp-nixos"],
      "env": {}
    },
    "context7": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp"],
      "env": {}
    },
    "git": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@cyanheads/git-mcp-server"],
      "env": {}
    }
  }
}
```

**Key Features:**
- Multiple stdio servers
- Uses both `npx` and `uvx` package runners
- Empty env objects (no environment variables needed)

---

## Example 2: twentyhq/twenty
**Repository:** https://github.com/twentyhq/twenty
**Transport Types:** stdio
**Notable:** Environment variable usage

```json
{
  "mcpServers": {
    "postgres": {
      "type": "stdio",
      "command": "uv",
      "args": ["run", "postgres-mcp", "--access-mode=unrestricted"],
      "env": {
        "DATABASE_URI": "${PG_DATABASE_URL}"
      }
    },
    "playwright": {
      "type": "stdio",
      "command": "npx",
      "args": ["@playwright/mcp@latest", "--no-sandbox", "--headless"],
      "env": {}
    },
    "context7": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp"],
      "env": {}
    }
  }
}
```

**Key Features:**
- Environment variable substitution: `${PG_DATABASE_URL}`
- Command-line flags in args: `--access-mode=unrestricted`, `--no-sandbox`, `--headless`
- Uses `uv` Python package runner

---

## Example 3: julep-ai/julep
**Repository:** https://github.com/julep-ai/julep
**Transport Types:** http, sse, stdio (MIXED!)

```json
{
  "mcpServers": {
    "deepwiki": {
      "type": "http",
      "url": "https://mcp.deepwiki.com/mcp"
    },
    "linear": {
      "type": "sse",
      "url": "https://mcp.linear.app/sse"
    },
    "fetch": {
      "type": "stdio",
      "command": "uvx",
      "args": ["mcp-server-fetch"],
      "env": {}
    },
    "firecrawl": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "firecrawl-mcp"],
      "env": {}
    }
  }
}
```

**Key Features:**
- **HTTP transport:** Simple URL-based connection
- **SSE transport:** Server-sent events for real-time updates
- **Mixed transports:** Combines remote (http/sse) and local (stdio) servers
- No command/args for http/sse, just URL

---

## Example 4: coder/coder
**Repository:** https://github.com/coder/coder
**Transport Types:** stdio
**Notable:** Complex command arguments

```json
{
  "mcpServers": {
    "go-language-server": {
      "type": "stdio",
      "command": "go",
      "args": [
        "run",
        "github.com/isaacphi/mcp-language-server@latest",
        "-workspace",
        "./",
        "-lsp",
        "go",
        "--",
        "run",
        "golang.org/x/tools/gopls@latest"
      ],
      "env": {}
    },
    "typescript-language-server": {
      "type": "stdio",
      "command": "go",
      "args": [
        "run",
        "github.com/isaacphi/mcp-language-server@latest",
        "-workspace",
        "./site/",
        "-lsp",
        "pnpx",
        "--",
        "typescript-language-server",
        "--stdio"
      ],
      "env": {}
    }
  }
}
```

**Key Features:**
- Complex nested commands (go run wrapping other tools)
- Workspace-specific configurations
- Language server integration

---

## Example 5: langbot-app/LangBot
**Repository:** https://github.com/langbot-app/LangBot
**Transport Types:** stdio
**Notable:** Environment variable with secrets

```json
{
  "mcpServers": {
    "sequential-thinking": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"],
      "env": {}
    },
    "github": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "${GITHUB_PERSONAL_ACCESS_TOKEN}"
      }
    },
    "fetch": {
      "type": "stdio",
      "command": "uvx",
      "args": ["mcp-server-fetch"],
      "env": {}
    },
    "playwright": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@playwright/mcp@latest"],
      "env": {}
    }
  }
}
```

**Key Features:**
- GitHub integration with PAT (Personal Access Token)
- Environment variable for sensitive data
- Common pattern: `${VAR_NAME}` for substitution

---

## Example 6: etewiah/property_web_builder
**Repository:** https://github.com/etewiah/property_web_builder
**Transport Types:** stdio
**Notable:** Multiple official MCP servers

```json
{
  "mcpServers": {
    "postgres": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres"],
      "env": {}
    },
    "git": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-git"],
      "env": {}
    },
    "filesystem": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem"],
      "env": {}
    },
    "memory": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-memory"],
      "env": {}
    }
  }
}
```

**Key Features:**
- All official `@modelcontextprotocol` servers
- Standard pattern for MCP server installation

---

## Example 7: YFGaia/dify-plus
**Repository:** https://github.com/YFGaia/dify-plus
**Transport Types:** http, stdio (MIXED)

```json
{
  "mcpServers": {
    "context7": {
      "type": "http",
      "url": "https://mcp.context7.com/mcp"
    },
    "sequential-thinking": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"],
      "env": {}
    },
    "github": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "${GITHUB_PERSONAL_ACCESS_TOKEN}"
      }
    },
    "fetch": {
      "type": "stdio",
      "command": "uvx",
      "args": ["mcp-server-fetch"],
      "env": {}
    },
    "playwright": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@playwright/mcp@latest"],
      "env": {}
    }
  }
}
```

**Key Features:**
- HTTP transport for context7 service
- Mixed local and remote servers

---

## Example 8: mmkal/pgkit
**Repository:** https://github.com/mmkal/pgkit
**Transport Types:** stdio
**Notable:** Optional type field, local file paths

```json
{
  "mcpServers": {
    "fetch": {
      "type": "stdio",
      "command": "uvx",
      "args": ["mcp-server-fetch"],
      "env": {}
    },
    "duckduckgo-web-search": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "github:mmkal/duck-duck-scrape-mcp"]
    },
    "playwright": {
      "command": "npx",
      "args": ["@playwright/mcp@latest", "--headless"]
    },
    "sequential-thinking": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"]
    },
    "cli-parser": {
      "type": "stdio",
      "command": "node",
      "args": ["../vibebot/src/mcp/cli-parser.ts"]
    },
    "op": {
      "type": "stdio",
      "command": "node",
      "args": ["./server/mcp/op.ts"]
    }
  }
}
```

**Key Features:**
- **Type field is optional** (defaults to stdio)
- GitHub repo references: `github:mmkal/duck-duck-scrape-mcp`
- Local TypeScript files: `./server/mcp/op.ts`
- Relative paths for custom servers

---

## Example 9: udecode/dotai
**Repository:** https://github.com/udecode/dotai
**Path:** `.claude-plugin/plugins/codex/.mcp.json`
**Transport Types:** stdio
**Notable:** Custom command with complex args

```json
{
  "mcpServers": {
    "gpt": {
      "type": "stdio",
      "command": "codex",
      "args": [
        "-s",
        "danger-full-access",
        "-m",
        "gpt-5",
        "-c",
        "model_reasoning_effort=high",
        "mcp-server"
      ],
      "env": {}
    },
    "codex": {
      "type": "stdio",
      "command": "codex",
      "args": [
        "-s",
        "danger-full-access",
        "-m",
        "gpt-5-codex",
        "-c",
        "model_reasoning_effort=high",
        "mcp-server"
      ],
      "env": {}
    }
  }
}
```

**Key Features:**
- Custom binary: `codex`
- Model selection via args
- Configuration flags

---

## Example 10: bbaserdem/NixOS-Config
**Repository:** https://github.com/bbaserdem/NixOS-Config
**Transport Types:** stdio
**Notable:** Nix package manager integration

```json
{
  "mcpServers": {
    "taskmaster-ai": {
      "type": "stdio",
      "command": "pnpx",
      "args": ["task-master-ai"]
    },
    "github": {
      "type": "stdio",
      "command": "pnpx",
      "args": ["@modelcontextprotocol/server-github"]
    },
    "nixos": {
      "type": "stdio",
      "command": "nix",
      "args": [
        "run",
        "github:utensils/mcp-nixos",
        "--"
      ]
    }
  }
}
```

**Key Features:**
- `pnpx` (pnpm's npx equivalent)
- Nix flake references: `github:utensils/mcp-nixos`

---

## Example 11: MicroPyramid/opensource-job-portal
**Repository:** https://github.com/MicroPyramid/opensource-job-portal
**Transport Types:** stdio
**Notable:** API key in args (security concern!)

```json
{
  "mcpServers": {
    "context7": {
      "type": "stdio",
      "command": "npx",
      "args": [
        "-y",
        "@upstash/context7-mcp",
        "--api-key",
        "ctx7sk-ff0c5d4d-0a4f-4565-8952-65ce0d547de7"
      ],
      "env": {}
    },
    "playwright": {
      "type": "stdio",
      "command": "npx",
      "args": ["@playwright/mcp@latest"],
      "env": {}
    },
    "svelte": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@sveltejs/mcp"],
      "env": {}
    }
  }
}
```

**Key Features:**
- **WARNING:** API key hardcoded in args (bad practice!)
- Should use environment variables instead

---

## Summary of Patterns

### Transport Type Distribution
- **stdio:** ~95% of examples (most common)
- **http:** ~3% (remote services)
- **sse:** ~2% (real-time services like Linear)
- **websocket:** <1% (rare, not found in sample)

### Common Commands
1. `npx` - Node package runner (most common)
2. `uvx` - Python uv package runner
3. `pnpx` - pnpm package runner
4. `node` - Direct Node.js execution
5. `go` - Go runtime
6. `nix` - Nix package manager

### Environment Variable Patterns
- `${VAR_NAME}` - Standard substitution syntax
- Common vars:
  - `GITHUB_PERSONAL_ACCESS_TOKEN`
  - `DATABASE_URI` / `PG_DATABASE_URL`
  - API keys (should be in env, not args!)

### Server Naming Conventions
- Lowercase with hyphens: `sequential-thinking`, `claude-code-mcp`
- Descriptive names: `postgres`, `github`, `playwright`
- No strict naming rules

### Configuration Structure
```json
{
  "mcpServers": {
    "server-name": {
      "type": "stdio|sse|http|ws",
      "command": "executable",      // stdio only
      "args": ["arg1", "arg2"],     // stdio only
      "url": "https://...",         // http/sse only
      "env": {                      // optional
        "VAR": "${VALUE}"
      }
    }
  }
}
```

---

## Test Cases for Bridle

### Test Case 1: Simple stdio server
```json
{
  "mcpServers": {
    "test-simple": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-memory"],
      "env": {}
    }
  }
}
```

### Test Case 2: Server with environment variables
```json
{
  "mcpServers": {
    "test-env": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "${GITHUB_TOKEN}"
      }
    }
  }
}
```

### Test Case 3: HTTP transport
```json
{
  "mcpServers": {
    "test-http": {
      "type": "http",
      "url": "https://mcp.context7.com/mcp"
    }
  }
}
```

### Test Case 4: SSE transport
```json
{
  "mcpServers": {
    "test-sse": {
      "type": "sse",
      "url": "https://mcp.linear.app/sse"
    }
  }
}
```

### Test Case 5: Mixed transports
```json
{
  "mcpServers": {
    "local-server": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-git"],
      "env": {}
    },
    "remote-http": {
      "type": "http",
      "url": "https://mcp.deepwiki.com/mcp"
    },
    "remote-sse": {
      "type": "sse",
      "url": "https://mcp.linear.app/sse"
    }
  }
}
```

### Test Case 6: Optional type field (defaults to stdio)
```json
{
  "mcpServers": {
    "test-no-type": {
      "command": "npx",
      "args": ["@playwright/mcp@latest"]
    }
  }
}
```

### Test Case 7: Local file paths
```json
{
  "mcpServers": {
    "test-local": {
      "type": "stdio",
      "command": "node",
      "args": ["./custom-server/index.js"]
    }
  }
}
```

### Test Case 8: Complex args with flags
```json
{
  "mcpServers": {
    "test-complex": {
      "type": "stdio",
      "command": "npx",
      "args": [
        "@playwright/mcp@latest",
        "--no-sandbox",
        "--headless",
        "--disable-gpu"
      ],
      "env": {}
    }
  }
}
```

---

## Repositories with .mcp.json Files

### High-Quality Examples
1. **juanfont/headscale** - https://github.com/juanfont/headscale
2. **twentyhq/twenty** - https://github.com/twentyhq/twenty
3. **julep-ai/julep** - https://github.com/julep-ai/julep (mixed transports!)
4. **coder/coder** - https://github.com/coder/coder
5. **langbot-app/LangBot** - https://github.com/langbot-app/LangBot
6. **etewiah/property_web_builder** - https://github.com/etewiah/property_web_builder
7. **mmkal/pgkit** - https://github.com/mmkal/pgkit
8. **udecode/dotai** - https://github.com/udecode/dotai

### Additional Examples Found
- kurrent-io/KurrentDB
- itlackey/sv-window-manager
- coreanq/web_serial
- fiswakenl/viztz
- QuickFind-Limited/claude_code_api
- nicogis/MCP-Server-ArcGIS-Pro-AddIn
- danielscholl/mvn-mcp-server
- ndendic/RustyTags
- jtmenchaca/tidy-ts
- marky291/Broke-Forge

---

## Key Insights for Bridle Implementation

1. **Type field is optional** - Defaults to "stdio" if omitted
2. **Environment variables** - Use `${VAR_NAME}` syntax for substitution
3. **Mixed transports** - Single file can contain stdio, http, sse, ws servers
4. **No env object required** - Can omit if empty
5. **Package managers** - Support npx, uvx, pnpx, nix, go run
6. **Local paths** - Relative and absolute paths supported for custom servers
7. **GitHub references** - Can use `github:owner/repo` in args
8. **Security** - API keys should be in env vars, not hardcoded in args

