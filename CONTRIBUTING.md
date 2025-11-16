# Contributing to Whispo

Thank you for your interest in contributing to Whispo! This guide will help you get started.

---

## 🚀 Quick Start

### Prerequisites

- **Node.js**: v22.15.0 or higher
- **pnpm**: v9.12.1 or higher
- **Rust**: Latest stable version
- **Python**: 3.12+ (Windows only)

### Setup

```bash
# Clone the repository
git clone https://github.com/egoist/whispo.git
cd whispo

# Install dependencies
pnpm install
```

---

## 💻 Development Commands

### Running the App

```bash
# Frontend only (UI development)
pnpm run dev

# Full app with Tauri backend (recommended for full testing)
pnpm run dev:tauri
```

**When to use which:**
- **`pnpm run dev`**: Use when working on UI/React components only
- **`pnpm run dev:tauri`**: Use when testing full app functionality, IPC calls, or Rust backend

### Building

```bash
# Build frontend only
pnpm run build

# Build full Tauri app
pnpm run build:tauri

# Platform-specific builds
pnpm run build:mac     # macOS (Apple Silicon & Intel)
pnpm run build:win     # Windows x64
pnpm run build:linux   # Linux x64
```

### Code Quality

```bash
# Type checking
pnpm run typecheck       # Check both frontend and backend types
pnpm run typecheck:web   # Check frontend only
pnpm run typecheck:node  # Check backend only

# Formatting
pnpm run format          # Format all files with Prettier

# Linting
pnpm run lint            # Lint and fix issues with ESLint
```

---

## 📂 Project Structure

```
whispo/
├── src/
│   ├── renderer/              # React UI (TypeScript)
│   │   ├── src/
│   │   │   ├── components/   # UI components
│   │   │   ├── pages/        # App pages
│   │   │   └── lib/          # Utilities
│   │   └── index.html
│   │
│   ├── shared/               # Shared types between frontend/backend
│   │   └── types.ts
│   │
│   └── main/                 # Legacy Electron code (reference only)
│
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── main.rs          # Main application entry
│   │   ├── config.rs        # Configuration management
│   │   ├── keyboard.rs      # Keyboard event handling
│   │   ├── shortcuts.rs     # Global shortcuts
│   │   ├── state.rs         # Application state
│   │   ├── types.rs         # Rust types
│   │   ├── mcp/             # Model Context Protocol
│   │   └── platform/        # Platform-specific code
│   │       ├── macos.rs
│   │       ├── windows.rs
│   │       └── linux.rs
│   │
│   ├── Cargo.toml           # Rust dependencies
│   ├── tauri.conf.json      # Tauri configuration
│   └── build.rs             # Build script
│
├── docs/                    # Documentation
├── scripts/                 # Build scripts
└── package.json            # Node.js dependencies
```

---

## 🔧 Making Changes

### Frontend Changes (TypeScript/React)

1. Start the dev server: `pnpm run dev:tauri`
2. Edit files in `src/renderer/src/`
3. Hot reload will automatically update the UI
4. Test your changes in the app window

### Backend Changes (Rust)

1. Edit files in `src-tauri/src/`
2. The app will automatically rebuild when you save
3. Check the terminal for compilation errors
4. Test IPC communication from frontend

### Adding a New Tauri Command

1. **Define the command in Rust** (`src-tauri/src/main.rs`):

```rust
#[tauri::command]
async fn my_new_command(param: String) -> Result<String, String> {
    // Implementation
    Ok(format!("Received: {}", param))
}
```

2. **Register the command** in the `invoke_handler`:

```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        // ... existing commands
        my_new_command,
    ])
```

3. **Call from TypeScript** (`src/renderer/src/lib/tauri-client.ts`):

```typescript
import { invoke } from '@tauri-apps/api/core'

const result = await invoke('my_new_command', { param: 'test' })
```

---

## 🧪 Testing

### Manual Testing Checklist

Before submitting a PR, test:

- [ ] App launches successfully
- [ ] Global keyboard shortcut works
- [ ] Recording and transcription work
- [ ] Text insertion works in external apps
- [ ] Settings save and load correctly
- [ ] All windows open/close properly
- [ ] Tray icon shows correct state

### Platform Testing

If possible, test on:
- [ ] macOS (Apple Silicon or Intel)
- [ ] Windows 10/11
- [ ] Linux (Ubuntu/Debian)

---

## 📝 Code Style

### TypeScript/React

- Use **functional components** with hooks
- Use **TypeScript** for all new code
- Follow **existing naming conventions**
- Use **Prettier** for formatting: `pnpm run format`
- Use **ESLint** rules: `pnpm run lint`

### Rust

- Follow **Rust standard style** (enforced by rustfmt)
- Use **meaningful variable names**
- Add **comments for complex logic**
- Use **Result<T, String>** for error handling
- Keep **functions focused and small**

### Documentation

- Update **README.md** for user-facing changes
- Update **docs/** for new features
- Add **inline comments** for complex code
- Update **CHANGELOG.md** for significant changes

---

## 🎯 Pull Request Process

1. **Fork the repository** and create a new branch
2. **Make your changes** following the code style
3. **Test thoroughly** on your platform
4. **Run type checking**: `pnpm run typecheck`
5. **Format code**: `pnpm run format`
6. **Commit with clear message**: See commit guidelines below
7. **Push to your fork** and create a Pull Request
8. **Describe your changes** in the PR description

### Commit Message Guidelines

Use clear, descriptive commit messages:

```
feat: add voice activation threshold setting
fix: keyboard shortcut not working on Windows
docs: update MCP integration guide
refactor: simplify audio recording logic
perf: optimize transcription processing
```

Prefixes:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `perf:` - Performance improvement
- `test:` - Test changes
- `chore:` - Build/config changes

---

## 🐛 Reporting Bugs

Found a bug? Please [open an issue](https://github.com/egoist/whispo/issues) with:

1. **Description**: What happened?
2. **Expected behavior**: What should happen?
3. **Steps to reproduce**: How to trigger the bug?
4. **Environment**: OS, version, etc.
5. **Logs**: Any error messages or console output

---

## 💡 Feature Requests

Have an idea? [Open a discussion](https://github.com/egoist/whispo/discussions) to:

1. **Describe the feature**: What do you want?
2. **Explain the use case**: Why is it useful?
3. **Discuss implementation**: How might it work?

---

## 📜 License

By contributing, you agree that your contributions will be licensed under the [AGPL-3.0 License](./LICENSE).

---

## 🙏 Thank You!

Every contribution helps make Whispo better. Thank you for being part of the community!

---

**Questions?** Feel free to ask in [GitHub Discussions](https://github.com/egoist/whispo/discussions).
