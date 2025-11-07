# 📚 Whispo Documentation

Complete documentation for Whispo, the AI-powered dictation tool.

---

## 📖 Table of Contents

### 🚀 [User Guides](./guides/)
Start here if you're new to Whispo.

- **[Quick Start Guide](./guides/QUICK_START.md)**
  Get up and running in 5 minutes

### ✨ [Features](./features/)
Learn about Whispo's powerful features.

- **[Model Context Protocol Integration](./features/MCP_INTEGRATION.md)**
  Complete guide to MCP client/server mode, tools, and context-aware transcription

### 🔧 [Migration Documentation](./migration/)
Technical documentation for the Electron to Tauri migration.

- **[Tauri Migration Overview](./migration/TAURI_MIGRATION.md)**
  High-level overview of the migration from Electron to Tauri

- **[Implementation Complete](./migration/IMPLEMENTATION_COMPLETE.md)**
  Comprehensive list of all implemented features

- **[Implementation Plan](./migration/IMPLEMENTATION-PLAN.md)**
  Original migration plan and architecture decisions

- **[Missing Features Fixed](./migration/MISSING_FEATURES_FIXED.md)**
  Critical features that were added post-migration

---

## 🎯 Quick Links

### For Users
- [Installation & Setup](./guides/QUICK_START.md#installation)
- [Basic Usage](./guides/QUICK_START.md#usage)
- [MCP Setup](./features/MCP_INTEGRATION.md#configuration)
- [Troubleshooting](./guides/QUICK_START.md#troubleshooting)

### For Developers
- [Project Structure](../README.md#project-structure)
- [Development Setup](../README.md#development)
- [Build Instructions](../README.md#build)
- [Architecture](./migration/TAURI_MIGRATION.md#architecture)

### For Contributors
- [Contributing Guidelines](../README.md#contributing)
- [Code Style](./migration/IMPLEMENTATION_COMPLETE.md#code-quality)
- [Testing](./migration/IMPLEMENTATION_COMPLETE.md#testing)

---

## 📂 Documentation Structure

```
docs/
├── README.md                  # This file - Documentation index
│
├── guides/                    # User-facing guides
│   └── QUICK_START.md        # Getting started guide
│
├── features/                  # Feature documentation
│   └── MCP_INTEGRATION.md    # Model Context Protocol
│
└── migration/                 # Technical migration docs
    ├── TAURI_MIGRATION.md              # Migration overview
    ├── IMPLEMENTATION_COMPLETE.md      # Complete feature list
    ├── IMPLEMENTATION-PLAN.md          # Original plan
    └── MISSING_FEATURES_FIXED.md       # Post-migration fixes
```

---

## 🔍 Find What You Need

### "How do I...?"

| Question | Answer |
|----------|--------|
| ...install Whispo? | [Quick Start Guide](./guides/QUICK_START.md) |
| ...configure MCP servers? | [MCP Integration](./features/MCP_INTEGRATION.md#configuration) |
| ...set up voice activation? | [Quick Start Guide](./guides/QUICK_START.md#voice-activation) |
| ...create app-specific rules? | [Quick Start Guide](./guides/QUICK_START.md#app-specific-rules) |
| ...build from source? | [Development Setup](../README.md#development) |

### "What is...?"

| Term | Definition | Learn More |
|------|------------|------------|
| **MCP** | Model Context Protocol - AI context sharing standard | [MCP Integration](./features/MCP_INTEGRATION.md) |
| **Voice Activation** | Hands-free recording triggered by speech | [Implementation Complete](./migration/IMPLEMENTATION_COMPLETE.md#voice-activation) |
| **Streaming Dictation** | Real-time transcription as you speak | [Implementation Complete](./migration/IMPLEMENTATION_COMPLETE.md#streaming-dictation) |
| **Fusion Transcription** | Multi-provider transcription with confidence scoring | [Implementation Complete](./migration/IMPLEMENTATION_COMPLETE.md#fusion-transcription) |
| **App Rules** | Custom settings per application | [Implementation Complete](./migration/IMPLEMENTATION_COMPLETE.md#app-specific-rules) |

---

## 🆘 Getting Help

- **Issues**: Found a bug? [Report it on GitHub](https://github.com/egoist/whispo/issues)
- **Questions**: Have a question? [Start a discussion](https://github.com/egoist/whispo/discussions)
- **Website**: Visit [whispo.app](https://whispo.app) for more information

---

## 📝 Contributing to Documentation

Documentation improvements are always welcome! If you find something unclear or missing:

1. [Open an issue](https://github.com/egoist/whispo/issues) describing what's unclear
2. Or submit a PR with improvements
3. Follow the existing structure and style

### Documentation Standards

- **Clear and Concise**: Get to the point quickly
- **Examples First**: Show, don't just tell
- **Complete**: No placeholders or "TODO" sections
- **Accurate**: Test all code examples
- **Organized**: Use proper headings and structure

---

**Last Updated**: 2025-11-07
**Version**: Tauri Migration Complete
