# 🛠️ Development Guide

Complete guide for developing Whispo locally.

---

## 📋 Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Development Workflow](#development-workflow)
4. [Project Architecture](#project-architecture)
5. [Common Tasks](#common-tasks)
6. [Debugging](#debugging)
7. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required Software

- **Node.js**: v22.15.0 or higher
  - Download from [nodejs.org](https://nodejs.org)
  - Verify: `node --version`

- **pnpm**: v9.12.1 or higher
  - Install: `npm install -g pnpm`
  - Verify: `pnpm --version`

- **Rust**: Latest stable version
  - Install: [rustup.rs](https://rustup.rs)
  - Verify: `rustc --version`

- **Tauri CLI**: Installed automatically with pnpm
  - Verify: `pnpm tauri --version`

### Platform-Specific Requirements

#### macOS
- **Xcode Command Line Tools**
  ```bash
  xcode-select --install
  ```

#### Windows
- **Visual Studio 2022** with C++ workload
- **WebView2**: Usually pre-installed on Windows 10/11
- **Python 3.12+**: For native module compilation

#### Linux
- **Development libraries**
  ```bash
  # Ubuntu/Debian
  sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

  # Fedora
  sudo dnf install webkit2gtk4.1-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel
  ```

---

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/egoist/whispo.git
cd whispo
```

### 2. Install Dependencies

```bash
pnpm install
```

This installs:
- Node.js dependencies (React, Vite, etc.)
- Rust dependencies (via Cargo)
- Tauri CLI tools

### 3. Verify Installation

```bash
# Check Node.js dependencies
pnpm list

# Check Rust dependencies
cd src-tauri
cargo check
cd ..
```

---

## Development Workflow

### Start Development Server

#### Option 1: Full App (Recommended)

Run the complete Tauri app with hot reload:

```bash
pnpm run dev:tauri
```

**What this does:**
- Starts Vite dev server for React UI
- Compiles Rust backend
- Launches the app window
- Enables hot reload for both frontend and backend

**Use this when:**
- Testing full application functionality
- Working on Rust backend
- Testing IPC communication
- Testing global shortcuts or system integration

#### Option 2: UI Only

Run just the React UI in a browser:

```bash
pnpm run dev
```

**What this does:**
- Starts Vite dev server at http://localhost:5173
- Shows UI in your web browser
- Faster reload times

**Use this when:**
- Working on React components
- Styling with Tailwind CSS
- UI layout and design
- Frontend-only changes

### Making Changes

#### Frontend Changes

1. Edit files in `src/renderer/src/`
2. Save the file
3. UI automatically reloads in the app/browser
4. Check for TypeScript errors in terminal

#### Backend Changes

1. Edit files in `src-tauri/src/`
2. Save the file
3. Rust backend automatically recompiles
4. App restarts with new changes
5. Check terminal for compilation errors

#### Configuration Changes

- **Tauri config**: `src-tauri/tauri.conf.json`
- **Rust dependencies**: `src-tauri/Cargo.toml`
- **Node dependencies**: `package.json`

After changing dependencies, restart the dev server.

---

## Project Architecture

### Frontend (TypeScript + React)

```
src/renderer/src/
├── components/          # React components
│   ├── ui/             # Reusable UI components
│   └── app-layout.tsx  # Main layout
│
├── pages/              # App pages/screens
│   ├── index.tsx       # Home/History
│   ├── settings-*.tsx  # Settings pages
│   └── panel.tsx       # Recording panel
│
├── lib/                # Utilities
│   └── tauri-client.ts # Tauri IPC wrapper
│
└── App.tsx             # Root component
```

### Backend (Rust + Tauri)

```
src-tauri/src/
├── main.rs             # Entry point, Tauri commands
├── config.rs           # Configuration management
├── keyboard.rs         # Keyboard event handling
├── shortcuts.rs        # Global shortcuts
├── state.rs            # Application state
├── types.rs            # Rust type definitions
│
├── mcp/                # Model Context Protocol
│   ├── mod.rs
│   ├── client.rs
│   ├── server.rs
│   ├── tools.rs
│   └── types.rs
│
└── platform/           # Platform-specific code
    ├── mod.rs
    ├── macos.rs
    ├── windows.rs
    └── linux.rs
```

### Communication Flow

```
┌─────────────────────────────────────┐
│   React UI (TypeScript)             │
│   src/renderer/src/                 │
└──────────────┬──────────────────────┘
               │
               │ IPC (invoke)
               │
┌──────────────▼──────────────────────┐
│   Tauri Commands (Rust)             │
│   #[tauri::command]                 │
│   async fn my_command()             │
└──────────────┬──────────────────────┘
               │
               │ Internal calls
               │
┌──────────────▼──────────────────────┐
│   Core Logic (Rust)                 │
│   - Config management               │
│   - Keyboard handling               │
│   - Platform APIs                   │
│   - MCP integration                 │
└─────────────────────────────────────┘
```

---

## Common Tasks

### Add a New Tauri Command

1. **Define in Rust** (`src-tauri/src/main.rs`):

```rust
#[tauri::command]
async fn my_command(param: String) -> Result<String, String> {
    println!("Received: {}", param);
    Ok(format!("Success: {}", param))
}
```

2. **Register the handler**:

```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        // ... existing commands
        my_command,
    ])
```

3. **Call from TypeScript**:

```typescript
import { invoke } from '@tauri-apps/api/core'

const result = await invoke<string>('my_command', {
  param: 'test value'
})
console.log(result)
```

### Add a New React Page

1. **Create page component** (`src/renderer/src/pages/my-page.tsx`):

```tsx
export default function MyPage() {
  return (
    <div>
      <h1>My New Page</h1>
    </div>
  )
}
```

2. **Add route** (`src/renderer/src/App.tsx`):

```tsx
<Route path="/my-page" element={<MyPage />} />
```

3. **Navigate to it**:

```tsx
import { Link } from 'react-router-dom'

<Link to="/my-page">Go to My Page</Link>
```

### Add a New Configuration Field

1. **Update Rust types** (`src-tauri/src/types.rs`):

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    // ... existing fields
    pub my_new_field: String,
}
```

2. **Update TypeScript types** (`src/shared/types.ts`):

```typescript
export type Config = {
  // ... existing fields
  myNewField: string
}
```

3. **Provide default value** in config.rs

---

## Debugging

### Frontend Debugging

#### Chrome DevTools

When running `pnpm run dev:tauri`, press:
- **macOS**: `Cmd + Option + I`
- **Windows/Linux**: `Ctrl + Shift + I`

Or add to your code:
```typescript
console.log('Debug info:', someVariable)
```

#### React DevTools

Not available in Tauri by default. Use browser mode:
```bash
pnpm run dev
```

Then install React DevTools browser extension.

### Backend Debugging

#### Console Logging

Add to your Rust code:
```rust
println!("Debug: {:?}", some_variable);
eprintln!("Error: {:?}", error);
```

Output appears in the terminal where you ran `pnpm run dev:tauri`.

#### Rust Debugger

Use VS Code with rust-analyzer:

1. Install "rust-analyzer" extension
2. Add breakpoints in Rust code
3. Press F5 to start debugging

### Common Debug Scenarios

#### IPC Call Not Working

1. **Check command name matches**:
   - Rust: `#[tauri::command] fn my_command()`
   - TypeScript: `invoke('my_command')`

2. **Check parameter names match**:
   - Rust: `fn cmd(my_param: String)`
   - TypeScript: `invoke('cmd', { myParam: 'value' })`

3. **Check return type**:
   - Rust must return `Result<T, String>`

#### Permission Errors

Check `src-tauri/tauri.conf.json`:
```json
{
  "tauri": {
    "allowlist": {
      "fs": {
        "scope": ["$APPDATA/*"]
      }
    }
  }
}
```

---

## Troubleshooting

### Build Errors

#### "cargo: command not found"
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### "node-gyp" errors on Windows
```bash
# Set Python path
set PYTHON=C:\Users\%USERNAME%\AppData\Local\Programs\Python\Python312\python.exe

# Install with --ignore-scripts
pnpm install --ignore-scripts
```

#### "webkit2gtk" errors on Linux
```bash
# Install WebKit dependencies
sudo apt install libwebkit2gtk-4.1-dev
```

### Runtime Errors

#### App won't launch
1. Check terminal for Rust compilation errors
2. Try `cargo clean` in `src-tauri/`
3. Restart dev server

#### Keyboard shortcuts not working
1. Check if accessibility permissions granted
2. Verify shortcut registration in code
3. Check for conflicting shortcuts

#### Transcription fails
1. Verify API keys in settings
2. Check internet connection
3. Look for error messages in console

---

## Building for Production

### Build Command

```bash
# Build for current platform
pnpm run build:tauri

# Platform-specific
pnpm run build:mac     # macOS
pnpm run build:win     # Windows
pnpm run build:linux   # Linux
```

### Build Output

Installers are created in:
```
src-tauri/target/release/bundle/
├── dmg/          # macOS
├── msi/          # Windows
└── deb/          # Linux
```

### Build Configuration

Edit `src-tauri/tauri.conf.json`:
```json
{
  "bundle": {
    "identifier": "app.whispo",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

---

## Next Steps

- Read [Contributing Guidelines](../../CONTRIBUTING.md)
- Explore [MCP Integration](../features/MCP_INTEGRATION.md)
- Check [Implementation Details](../migration/IMPLEMENTATION_COMPLETE.md)

---

**Need help?** [Open a discussion](https://github.com/egoist/whispo/discussions)
