# Quick Start - Tauri Version

## Prerequisites

1. **Install Rust**: https://rustup.rs/
2. **Install Tauri CLI**:
   ```bash
   cargo install tauri-cli
   ```
3. **Install Node.js dependencies**:
   ```bash
   pnpm install
   ```

## Development

```bash
# Start development server with hot reload
pnpm dev
```

This will:
1. Start Vite dev server for the frontend
2. Compile the Rust backend
3. Launch the Tauri application
4. Enable hot reload for both frontend and backend

## Building

```bash
# Build for your current platform
pnpm tauri:build

# Output will be in src-tauri/target/release/bundle/
```

### Platform-Specific Builds

```bash
# Windows
pnpm tauri:build:win

# macOS (Apple Silicon)
pnpm tauri:build:mac

# Linux
pnpm tauri:build:linux
```

## Project Structure

```
whispo/
├── src/
│   ├── renderer/          # Frontend (React + TypeScript)
│   │   ├── src/
│   │   │   ├── components/
│   │   │   ├── pages/
│   │   │   ├── lib/
│   │   │   │   └── tauri-client.ts  # Tauri IPC client
│   │   │   └── main.tsx
│   │   └── index.html
│   └── shared/            # Shared types
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Main app + Tauri commands
│   │   ├── keyboard.rs    # Keyboard handling
│   │   └── config.rs      # Configuration storage
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── vite.config.ts         # Vite configuration
└── package.json           # Node.js dependencies
```

## Key Changes from Electron

1. **IPC**: `tipcClient.method()` now uses Tauri's `invoke()` under the hood
2. **No preload script**: Tauri doesn't need a preload script
3. **Rust backend**: All backend logic is now in Rust instead of Node.js
4. **Smaller bundles**: ~10-20MB instead of ~100-200MB

## Common Commands

```bash
# Development
pnpm dev                 # Start dev server
pnpm dev:frontend        # Start only frontend (for testing)

# Building
pnpm build               # Build frontend
pnpm tauri:build         # Build complete app

# Code Quality
pnpm format              # Format code
pnpm lint                # Lint code
pnpm typecheck           # Type check TypeScript
```

## Troubleshooting

### "tauri: command not found"

Install Tauri CLI:
```bash
cargo install tauri-cli
```

### Build errors on macOS

Install Xcode Command Line Tools:
```bash
xcode-select --install
```

### Build errors on Linux

Install required dependencies:
```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.0-dev libappindicator3-dev

# Fedora
sudo dnf install webkit2gtk3-devel libappindicator-gtk3-devel
```

### Build errors on Windows

Install Visual Studio C++ Build Tools:
https://visualstudio.microsoft.com/downloads/

## More Information

See `TAURI_MIGRATION.md` for detailed migration documentation.
