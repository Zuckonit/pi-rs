# Sandbox Feature Design

## Overview

Add sandbox functionality to pi-rs for environment isolation during development.

## Command Line Interface

```bash
# Basic usage - sandbox with project directory
pi-rs --sandbox <project-path>

# With additional mounts (like docker -v)
pi-rs --sandbox /my/project -v /opt/epkg -v /data

# With environment variables
pi-rs --sandbox /my/project -e CUSTOM_VAR=value

# Specify sandbox type (default: epkg)
pi-rs --sandbox /my/project --sandbox-type epkg

# Disable sandbox (override config)
pi-rs --sandbox /my/project --no-sandbox

# Normal usage (no sandbox)
pi-rs
```

## Configuration File

Location: `.pi/sandbox.json` (project directory)

```json
{
  "enabled": true,
  "type": "epkg",
  "mounts": [
    "/opt/epkg",
    "/data"
  ],
  "env": {
    "CUSTOM_VAR": "value"
  }
}
```

## Sandbox Type: epkg

Implementation based on `sandbox-epkg.sh`:

1. Create mount namespace with `sudo unshare --mount`
2. Create tmpfs at `/tmp/sandbox-pi-rs`
3. Bind mount:
   - System directories (read-only): `/usr`, `/etc`, `/lib`, `/bin`, `/sbin`
   - Project directory (read-write)
   - Additional mounts specified by user
4. Pivot root into sandbox
5. Execute pi-rs in sandbox

## Environment Variables

### Auto-propagated (default)

Common API keys are automatically propagated into sandbox:
- `MOONSHOT_API_KEY`
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`
- `GOOGLE_API_KEY`
- `OLLAMA_BASE_URL`
- `AZURE_OPENAI_API_KEY`
- `MISTRAL_API_KEY`
- `GROQ_API_KEY`

### Explicit (CLI)

```bash
pi-rs --sandbox /project -e KEY=value
```

### Config (JSON)

```json
{
  "env": {
    "MY_VAR": "value"
  }
}
```

### Priority

CLI > Config file > Auto-propagated

## Error Handling

| Scenario | Behavior |
|----------|----------|
| `-v` without `--sandbox` | Error: `-v requires --sandbox` |
| No sudo permission | Error: `sandbox requires sudo` |
| Invalid mount path | Error: `mount path does not exist` |
| Sandbox type not supported | Error: `unsupported sandbox type: xxx` |

## Implementation Plan

1. Add CLI arguments in `src/cli/args.rs`
2. Create sandbox config module in `src/sandbox/`
3. Implement epkg sandbox in `src/sandbox/epkg.rs`
4. Integrate sandbox startup in `src/main.rs`
5. Add tests

## Files to Create/Modify

- `src/sandbox/mod.rs` - Sandbox module
- `src/sandbox/epkg.rs` - Epkg sandbox implementation
- `src/sandbox/config.rs` - Configuration parsing
- `src/cli/args.rs` - Add sandbox arguments
- `src/main.rs` - Integrate sandbox startup
