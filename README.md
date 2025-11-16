# 🎙️ Whispo

> AI-powered dictation tool built with Tauri and Rust

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](./LICENSE)
[![Platform: macOS | Windows | Linux](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey.svg)]()
[![Built with Tauri](https://img.shields.io/badge/Built%20with-Tauri-FFC131.svg)](https://tauri.app)

Whispo is a powerful, privacy-focused dictation application that transcribes your voice in real-time and seamlessly integrates with any application on your system.

---

## ✨ Features

### Core Functionality
- **⌨️ Push-to-Talk Recording**: Hold a global shortcut to record, release to transcribe
- **🤖 AI-Powered Transcription**: Powered by OpenAI Whisper (via OpenAI or Groq)
- **📝 Automatic Text Insertion**: Transcripts are automatically inserted into your active application
- **🔒 Privacy First**: All data stored locally on your machine
- **🌍 Universal Compatibility**: Works with any text input field on macOS, Windows, and Linux

### Advanced Features
- **🎯 Voice Activation**: Hands-free recording with voice detection
- **📡 Streaming Dictation**: Real-time transcription as you speak
- **📋 App-Specific Rules**: Custom settings per application
- **🔄 Profile Management**: Multiple configuration profiles for different workflows
- **🎨 Context-Aware Formatting**: Automatic formatting based on application context
- **🔗 Fusion Transcription**: Multi-provider transcription with confidence scoring
- **🧠 LLM Post-Processing**: Enhance transcripts with GPT-4, Gemini, or other LLMs
- **🔌 MCP Integration**: Full Model Context Protocol support for AI context sharing

---

## 📥 Download

**Latest Release:** [Download for your platform](https://github.com/egoist/whispo/releases/latest)

Supported Platforms:
- macOS (Apple Silicon & Intel)
- Windows (x64)
- Linux (x64)

---

## 🚀 Quick Start

### 1. Install and Launch
Download the installer for your platform and run it. Whispo will guide you through the initial setup.

### 2. Grant Permissions
- **Microphone Access**: Required for recording your voice
- **Accessibility Access**: Required for automatic text insertion (macOS/Linux)

### 3. Configure API Keys
Whispo supports multiple transcription providers:
- **OpenAI**: Get your API key from [platform.openai.com](https://platform.openai.com)
- **Groq**: Get your API key from [console.groq.com](https://console.groq.com)

### 4. Start Dictating
- Press and hold your configured shortcut (default: `Ctrl`)
- Speak your message
- Release to transcribe and insert

For detailed setup instructions, see the [Quick Start Guide](./docs/guides/QUICK_START.md).

---

## 📖 Documentation

### User Guides
- [Quick Start Guide](./docs/guides/QUICK_START.md) - Get started with Whispo
- [Development Guide](./docs/guides/DEVELOPMENT.md) - Complete development guide
- [MCP Integration](./docs/features/MCP_INTEGRATION.md) - Model Context Protocol support

### Development & Contributing
- [Contributing Guidelines](./CONTRIBUTING.md) - How to contribute to Whispo
- [Tauri Migration](./docs/migration/TAURI_MIGRATION.md) - Electron to Tauri conversion details
- [Implementation Complete](./docs/migration/IMPLEMENTATION_COMPLETE.md) - Full feature list
- [Missing Features Fixed](./docs/migration/MISSING_FEATURES_FIXED.md) - Critical fixes applied

---

## 🛠️ Development

### Prerequisites
- **Node.js**: v22.15.0 or higher
- **pnpm**: v9.12.1 or higher
- **Rust**: Latest stable (for Tauri development)
- **Python**: 3.12+ (Windows only, for native module compilation)

### Setup

```bash
# Clone the repository
git clone https://github.com/egoist/whispo.git
cd whispo

# Install dependencies
pnpm install
```

### Development

```bash
# Run UI only (frontend development)
pnpm run dev

# Run full app with Tauri + UI (recommended)
pnpm run dev:tauri
```

### Build

```bash
# Build for your current platform
pnpm run build:tauri

# Platform-specific builds
pnpm run build:mac    # macOS
pnpm run build:win    # Windows
pnpm run build:linux  # Linux
```

### Project Structure

```
whispo/
├── src/                    # Frontend (TypeScript + React)
│   ├── renderer/          # React UI components
│   └── shared/            # Shared types and utilities
├── src-tauri/             # Backend (Rust + Tauri)
│   └── src/
│       ├── main.rs        # Main application logic
│       ├── config.rs      # Configuration management
│       ├── keyboard.rs    # Keyboard event handling
│       ├── shortcuts.rs   # Global shortcuts
│       ├── mcp/           # Model Context Protocol
│       └── platform/      # Platform-specific code
├── docs/                  # Documentation
│   ├── guides/           # User guides
│   ├── features/         # Feature documentation
│   └── migration/        # Migration documentation
└── scripts/              # Build and release scripts
```

---

## 🎯 Key Features Explained

### Voice Activation
Automatically start recording when you speak, no need to hold a key. Perfect for hands-free operation.

### Streaming Dictation
See your words appear in real-time as you speak, with live transcription updates.

### App-Specific Rules
Create custom settings for different applications:
- Different shortcuts per app
- Custom formatting rules
- Application-specific glossaries

### Fusion Transcription
Use multiple transcription providers simultaneously and combine results with confidence scoring for maximum accuracy.

### MCP Integration
Connect Whispo to the Model Context Protocol ecosystem:
- **Client Mode**: Connect to MCP servers (filesystem, GitHub, databases)
- **Server Mode**: Expose Whispo functionality to other apps
- **Context-Aware**: Use project context for better transcription

[Learn more about MCP Integration →](./docs/features/MCP_INTEGRATION.md)

---

## 🔧 Configuration

Configuration is stored in:
- **macOS**: `~/Library/Application Support/whispo/config.json`
- **Windows**: `%APPDATA%\whispo\config.json`
- **Linux**: `~/.config/whispo/config.json`

You can manage all settings through the in-app Settings interface.

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines
- Follow existing code style
- Write tests for new features
- Update documentation
- Keep commits atomic and descriptive

---

## 📜 License

[AGPL-3.0](./LICENSE) - This project is licensed under the GNU Affero General Public License v3.0.

---

## 🙏 Acknowledgments

Built with:
- [Tauri](https://tauri.app) - Build smaller, faster, and more secure desktop applications
- [OpenAI Whisper](https://openai.com/research/whisper) - Robust speech recognition
- [React](https://react.dev) - UI framework
- [Rust](https://www.rust-lang.org) - Systems programming language

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/egoist/whispo/issues)
- **Discussions**: [GitHub Discussions](https://github.com/egoist/whispo/discussions)
- **Website**: [whispo.app](https://whispo.app)

---

**Made with ❤️ by the Whispo team**
