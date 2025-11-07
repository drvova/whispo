# Tauri Migration Guide

This document describes the migration of Whispo from Electron to Tauri.

## Overview

Whispo has been converted from an Electron application to a Tauri application. Tauri provides:
- Smaller bundle sizes (Rust backend instead of Node.js)
- Better performance and security
- Native system integration
- Cross-platform compatibility

## What Has Been Done

### ✅ Core Structure

1. **Tauri Configuration**
   - Created `src-tauri/` directory with Rust backend
   - Created `src-tauri/Cargo.toml` with dependencies
   - Created `src-tauri/tauri.conf.json` with app configuration
   - Set up build configuration in `src-tauri/build.rs`

2. **Rust Backend** (`src-tauri/src/`)
   - `main.rs`: Core Tauri application with window management, system tray, and IPC commands
   - `keyboard.rs`: Keyboard event listening and text writing (migrated from whispo-rs)
   - `config.rs`: Configuration storage and management
   - `commands.rs`: Additional Tauri commands (placeholder)

3. **IPC Migration**
   - Converted Electron IPC (tipc) to Tauri commands
   - Created `src/renderer/src/lib/tauri-client.ts` to provide compatible API
   - Updated `src/renderer/src/lib/tipc-client.ts` to use Tauri client

4. **Frontend Integration**
   - Created `vite.config.ts` for frontend build configuration
   - Updated `package.json` scripts for Tauri development and build

5. **Window Management**
   - Main window (settings/history)
   - Panel window (recording UI)
   - Statusbar window (persistent status bar)
   - Setup window (permissions/onboarding)

6. **System Integration**
   - System tray with menu
   - Window positioning and sizing
   - Clipboard integration

## What Still Needs to Be Done

### 🔨 Critical Tasks

1. **Tauri Plugins Installation**
   ```bash
   # These need to be installed once network/cargo access is available
   cargo install tauri-cli
   ```

2. **Icons**
   - Copy icon files from `build/` to `src-tauri/icons/`
   - Ensure all required sizes are present:
     - 32x32.png
     - 128x128.png
     - 128x128@2x.png
     - icon.icns (macOS)
     - icon.ico (Windows)

3. **Global Keyboard Shortcuts**
   - Implement global shortcut registration in Rust
   - Currently uses `tauri-plugin-global-shortcut`
   - Need to register the recording shortcut (default: Ctrl)

4. **Complete Tauri Commands**

   The following commands from the original tipc router need full implementation:

   - `getUpdateInfo` / `quitAndInstall` / `checkForUpdatesAndDownload` - Auto-updates
   - `getMicrophoneStatus` / `requestMicrophoneAccess` - Microphone permissions
   - `isAccessibilityGranted` / `requestAccesssbilityAccess` - Accessibility permissions
   - `openMicrophoneInSystemPreferences` - System preferences navigation
   - `showContextMenu` - Context menu display
   - `toggleRecordingTranscript` - Toggle between original/processed transcripts
   - Voice activation commands (all implemented as stubs)
   - Streaming dictation commands (all implemented as stubs)
   - App-specific rules commands (stubs)
   - Profile management commands (stubs)
   - Fusion transcription commands (stubs)
   - Context formatting commands (stubs)

5. **STT Integration**

   The `create_recording` command needs full implementation:
   - HTTP calls to OpenAI/Groq for transcription
   - Fusion transcription logic
   - Post-processing with LLMs
   - Error handling

6. **Platform-Specific Code**

   Need conditional compilation for:
   - macOS: Microphone permissions, accessibility API
   - Windows: Different permission model
   - Linux: Permission handling

### 🎯 Medium Priority

1. **App Detection**
   - Implement active application detection
   - App-specific rules matching
   - Context detection for formatting

2. **Voice Activation**
   - Implement voice-based recording trigger
   - Audio level monitoring
   - Threshold detection

3. **Streaming Dictation**
   - Real-time transcription
   - Streaming audio processing
   - Live text insertion

4. **Auto-Updater**
   - Tauri has built-in updater support
   - Need to configure GitHub releases
   - Implement update checking and installation

### 📝 Low Priority

1. **Migration from Electron Dependencies**
   - Remove unused Electron packages:
     - `electron`
     - `electron-vite`
     - `electron-builder`
     - `electron-updater`
     - `@egoist/electron-panel-window`
     - `@electron-toolkit/*`
     - `@egoist/tipc`

2. **Testing**
   - Test all IPC commands
   - Test window management
   - Test system tray functionality
   - Test on all platforms (macOS, Windows, Linux)

3. **Documentation**
   - Update README.md with Tauri instructions
   - Document new build process
   - Update development setup

## Architecture Changes

### Electron vs Tauri

**Electron Architecture:**
```
┌─────────────────────────────────┐
│   Frontend (Renderer Process)   │
│   React + TypeScript            │
└────────────┬────────────────────┘
             │ IPC (tipc)
┌────────────▼────────────────────┐
│   Backend (Main Process)        │
│   Node.js + TypeScript          │
│   ├── Spawns whispo-rs binary  │
│   └── Communicates via stdio    │
└─────────────────────────────────┘
```

**Tauri Architecture:**
```
┌─────────────────────────────────┐
│   Frontend (WebView)            │
│   React + TypeScript            │
└────────────┬────────────────────┘
             │ IPC (invoke)
┌────────────▼────────────────────┐
│   Backend (Core Process)        │
│   Rust                          │
│   ├── Integrated keyboard code  │
│   └── Direct native access      │
└─────────────────────────────────┘
```

### Key Differences

1. **Backend Language**: Node.js/TypeScript → Rust
2. **IPC Mechanism**: tipc (custom) → Tauri invoke (built-in)
3. **Keyboard Handling**: External binary → Integrated module
4. **Bundle Size**: ~100-200MB → ~10-20MB (estimated)
5. **Startup Time**: Slower → Faster
6. **Memory Usage**: Higher → Lower

## Development Workflow

### Development

```bash
# Start development server
pnpm dev

# This will:
# 1. Start Vite dev server for frontend (port 5173)
# 2. Build Rust backend
# 3. Launch Tauri app with hot reload
```

### Building

```bash
# Build for current platform
pnpm tauri:build

# Build for specific platforms
pnpm tauri:build:win   # Windows
pnpm tauri:build:mac   # macOS (Apple Silicon)
pnpm tauri:build:linux # Linux
```

### Prerequisites

- **Rust**: Install from https://rustup.rs/
- **System Dependencies**:
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio C++ Build Tools
  - **Linux**: webkit2gtk, libappindicator

## Configuration Files

### `src-tauri/tauri.conf.json`

Main Tauri configuration:
- App metadata (name, version, identifier)
- Window configurations
- Build settings
- Plugin configurations
- Security policies

### `src-tauri/Cargo.toml`

Rust dependencies:
- `tauri`: Core framework
- `rdev`: Keyboard event listening
- `enigo`: Keyboard simulation
- `reqwest`: HTTP client for API calls
- `serde`: JSON serialization
- Tauri plugins for various features

### `vite.config.ts`

Frontend build configuration:
- Entry point: `src/renderer`
- Output: `out/renderer`
- Dev server port: 5173

## Known Issues and Limitations

1. **Network Restrictions**
   - Cannot install Tauri CLI via cargo in current environment
   - Need to install on a machine with network access

2. **Missing Implementations**
   - Many commands are stubs returning placeholder data
   - STT integration needs HTTP client implementation
   - Platform-specific APIs need conditional compilation

3. **Testing Required**
   - Full application flow not tested
   - Cross-platform compatibility not verified
   - Performance benchmarking needed

## Migration Checklist

- [x] Create Tauri project structure
- [x] Migrate keyboard handling code
- [x] Create Tauri commands for IPC
- [x] Update frontend to use Tauri client
- [x] Configure build system
- [x] Set up window management
- [x] Implement system tray
- [ ] Install Tauri CLI
- [ ] Add icon files
- [ ] Implement global keyboard shortcuts
- [ ] Complete all Tauri commands
- [ ] Implement STT integration
- [ ] Add platform-specific code
- [ ] Test on all platforms
- [ ] Remove Electron dependencies
- [ ] Update documentation

## Next Steps

1. **Get network/cargo access** to install Tauri CLI
2. **Copy icons** to `src-tauri/icons/`
3. **Implement critical commands** (permissions, shortcuts, STT)
4. **Test basic functionality** (recording, transcription, text insertion)
5. **Implement remaining features** (voice activation, streaming, etc.)
6. **Clean up dependencies** (remove Electron packages)
7. **Test on all platforms**
8. **Create release builds**

## Resources

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Tauri API Reference](https://tauri.app/v1/api/js/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [rdev Documentation](https://docs.rs/rdev/)
- [enigo Documentation](https://docs.rs/enigo/)

## Contact

For questions or issues with the migration, refer to the main README.md or open an issue on GitHub.
