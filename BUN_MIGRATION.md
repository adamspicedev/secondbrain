# Bun Migration Guide

This project has been migrated to **Bun** as the JavaScript runtime and package manager.

## Why Bun?

- ⚡ **~4x faster** npm install/run compared to Node.js
- 🚀 **Native TypeScript support** - no tsc compilation needed
- 📦 **Drop-in Node.js replacement** - runs existing npm packages
- 💪 **Built-in testing & bundling** - fewer dependencies needed
- 🔒 **Better security** - lockfile hashing, built-in checkers

## Installation

### macOS / Linux

```bash
curl -fsSL https://bun.sh/install | bash
```

### Windows

```powershell
powershell -c "$(IRM bun.sh/install.ps1)"
```

### Verify Installation

```bash
bun --version  # Should show v1.0.0 or higher
```

## Key Differences from npm

### Installation

```bash
# OLD
npm install

# NEW
bun install
```

### Running Scripts

```bash
# OLD
npm run dev

# NEW
bun run dev
```

### Adding Packages

```bash
# OLD
npm install package-name

# NEW
bun add package-name
```

### Dev Dependencies

```bash
# OLD
npm install --save-dev package-name

# NEW
bun add -d package-name
```

### Global Installation

```bash
# OLD
npm install -g package-name

# NEW
bun add -g package-name
```

## Common Commands Used in This Project

```bash
# Install all dependencies
bun install

# Run development server (Tauri + Vite)
bun run dev

# Build production app
bun run build

# Type check TypeScript
bun run type-check

# Preview build output
bun run preview
```

## Lock File

- **Old**: `package-lock.json`
- **New**: `bun.lockb` (binary format, faster)

Don't commit the `.lockb` file; Bun handles this automatically.

## Configuration

The project includes a `bunfig.toml` file with Bun-specific settings:

```toml
[install]
peer = true

[test]
timeout = 30000

[build]
minify = { syntax = true, whitespace = true }
target = "browser"

[jsx]
runtime = "react"
```

## Troubleshooting

### "bun: command not found"

Ensure Bun is in your PATH:

```bash
# Add to ~/.bash_profile, ~/.zshrc, or ~/.bashrc
export PATH=$HOME/.bun/bin:$PATH
```

### Dependency Conflicts

If you encounter issues, clear and reinstall:

```bash
rm -rf bun.lockb node_modules
bun install
```

### TypeScript Issues

Bun runs TypeScript natively—no separate tsc step needed. If you need IDE support:

```bash
bun run type-check
```

## What Hasn't Changed

- **Tauri commands**: Still `tauri dev`, `tauri build`
- **Rust backend**: Unchanged—Bun only affects JavaScript
- **PostgreSQL**: Unchanged—database code works the same
- **Dependencies**: All npm packages work with Bun

## Learn More

- [Bun Documentation](https://bun.sh/docs)
- [Bun API Reference](https://bun.sh/docs/api/file-io)
- [Performance Tips](https://bun.sh/docs/cli/install#performance)
