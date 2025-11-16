# 🔒 Security Implementation Guide

Guide for implementing security fixes identified in the security audit.

---

## 📋 Quick Start - Critical Fixes

These are the **MUST-FIX** issues that should be addressed immediately:

1. [Enable Content Security Policy](#1-enable-content-security-policy)
2. [Encrypt API Keys](#2-encrypt-api-keys-with-os-keychain)
3. [Add Input Validation](#3-add-input-validation)
4. [Restrict Shell Access](#4-restrict-shell-access)

---

## 1. Enable Content Security Policy

**Priority**: 🔴 CRITICAL
**Time**: 10 minutes
**File**: `src-tauri/tauri.conf.json`

### Current (Insecure)
```json
{
  "app": {
    "security": {
      "csp": null  // ⚠️ DANGEROUS
    }
  }
}
```

### Fixed (Secure)
```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' https://api.openai.com https://api.groq.com https://generativelanguage.googleapis.com;"
    }
  }
}
```

### Explanation
- **`default-src 'self'`**: Only load resources from same origin
- **`script-src 'self'`**: Only execute scripts from app (no inline scripts, no eval)
- **`style-src 'self' 'unsafe-inline'`**: Styles from app + inline CSS (needed for React)
- **`img-src 'self' data: https:`**: Images from app, data URLs, and HTTPS
- **`connect-src 'self' https://api.openai.com ...`**: Only connect to these APIs

### Testing
```bash
# 1. Update tauri.conf.json
# 2. Run app
pnpm run dev:tauri

# 3. Check DevTools console for CSP violations
# If you see errors, adjust CSP accordingly
```

---

## 2. Encrypt API Keys with OS Keychain

**Priority**: 🔴 CRITICAL
**Time**: 2-3 hours
**Files**: `src-tauri/Cargo.toml`, `src-tauri/src/config.rs`

### Step 1: Add Dependency

Add to `src-tauri/Cargo.toml`:
```toml
[dependencies]
keyring = "2.0"
```

### Step 2: Create Secure Config Module

Create `src-tauri/src/secure_config.rs`:
```rust
use keyring::Entry;
use anyhow::Result;

pub struct SecureConfig;

impl SecureConfig {
    /// Save API key to OS keychain
    pub fn save_api_key(service: &str, key: &str) -> Result<()> {
        let entry = Entry::new("whispo", service)?;
        entry.set_password(key)?;
        Ok(())
    }

    /// Get API key from OS keychain
    pub fn get_api_key(service: &str) -> Result<String> {
        let entry = Entry::new("whispo", service)?;
        Ok(entry.get_password()?)
    }

    /// Delete API key from OS keychain
    pub fn delete_api_key(service: &str) -> Result<()> {
        let entry = Entry::new("whispo", service)?;
        entry.delete_password()?;
        Ok(())
    }
}

// Helper functions for specific services
pub fn save_openai_key(key: &str) -> Result<()> {
    SecureConfig::save_api_key("openai", key)
}

pub fn get_openai_key() -> Result<String> {
    SecureConfig::get_api_key("openai")
}

pub fn save_groq_key(key: &str) -> Result<()> {
    SecureConfig::save_api_key("groq", key)
}

pub fn get_groq_key() -> Result<String> {
    SecureConfig::get_api_key("groq")
}

pub fn save_gemini_key(key: &str) -> Result<()> {
    SecureConfig::save_api_key("gemini", key)
}

pub fn get_gemini_key() -> Result<String> {
    SecureConfig::get_api_key("gemini")
}
```

### Step 3: Update Config Storage

Modify `src-tauri/src/config.rs`:
```rust
use crate::secure_config;

impl ConfigStore {
    pub fn save(&self, mut config: Value) -> Result<()> {
        // Extract API keys
        let openai_key = config.get("openaiApiKey")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let groq_key = config.get("groqApiKey")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let gemini_key = config.get("geminiApiKey")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Save to OS keychain
        if let Some(key) = openai_key {
            if !key.is_empty() {
                secure_config::save_openai_key(&key)?;
            }
        }
        if let Some(key) = groq_key {
            if !key.is_empty() {
                secure_config::save_groq_key(&key)?;
            }
        }
        if let Some(key) = gemini_key {
            if !key.is_empty() {
                secure_config::save_gemini_key(&key)?;
            }
        }

        // Remove API keys from config JSON
        if let Some(obj) = config.as_object_mut() {
            obj.remove("openaiApiKey");
            obj.remove("groqApiKey");
            obj.remove("geminiApiKey");
        }

        // Save config WITHOUT API keys
        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&self.config_path, content)?;
        *self.current_config.lock().unwrap() = config;
        Ok(())
    }

    pub fn get(&self) -> Value {
        let mut config = self.current_config.lock().unwrap().clone();

        // Load API keys from keychain
        if let Ok(key) = secure_config::get_openai_key() {
            config["openaiApiKey"] = serde_json::json!(key);
        }
        if let Ok(key) = secure_config::get_groq_key() {
            config["groqApiKey"] = serde_json::json!(key);
        }
        if let Ok(key) = secure_config::get_gemini_key() {
            config["geminiApiKey"] = serde_json::json!(key);
        }

        config
    }
}
```

### Step 4: Add Migration

Create migration for existing users:
```rust
#[tauri::command]
async fn migrate_api_keys(config_store: State<'_, Arc<ConfigStore>>) -> Result<(), String> {
    // Get current config (with keys in JSON)
    let config = config_store.get();

    // This will extract keys and save to keychain
    config_store.save(config).map_err(|e| e.to_string())?;

    Ok(())
}
```

### Testing
```bash
# After implementation:
# 1. Check keychain (macOS)
security find-generic-password -s whispo -a openai

# 2. Check config.json (should NOT contain keys)
cat ~/Library/Application\ Support/whispo/config.json
# Should NOT see: "openaiApiKey": "sk-..."

# 3. Test app still works with transcription
```

---

## 3. Add Input Validation

**Priority**: 🟠 HIGH
**Time**: 1-2 hours
**File**: `src-tauri/src/main.rs`, `src-tauri/src/config.rs`

### Create Validation Module

Create `src-tauri/src/validation.rs`:
```rust
use anyhow::{Result, bail};

pub struct Validator;

impl Validator {
    /// Validate profile name
    pub fn profile_name(name: &str) -> Result<()> {
        if name.is_empty() {
            bail!("Profile name cannot be empty");
        }
        if name.len() > 100 {
            bail!("Profile name too long (max 100 characters)");
        }
        if !name.chars().all(|c| {
            c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_'
        }) {
            bail!("Profile name contains invalid characters");
        }
        Ok(())
    }

    /// Validate audio data size
    pub fn audio_data(data: &[u8]) -> Result<()> {
        const MAX_SIZE: usize = 50 * 1024 * 1024; // 50 MB
        if data.is_empty() {
            bail!("Audio data is empty");
        }
        if data.len() > MAX_SIZE {
            bail!("Audio data too large (max 50MB)");
        }
        Ok(())
    }

    /// Validate API URL is HTTPS
    pub fn api_url(url: &str) -> Result<()> {
        if !url.starts_with("https://") {
            bail!("API URL must use HTTPS");
        }
        if reqwest::Url::parse(url).is_err() {
            bail!("Invalid API URL format");
        }
        Ok(())
    }

    /// Validate shortcut string
    pub fn shortcut(shortcut: &str) -> Result<()> {
        if shortcut.is_empty() {
            bail!("Shortcut cannot be empty");
        }
        if shortcut.len() > 50 {
            bail!("Shortcut string too long");
        }
        Ok(())
    }
}
```

### Apply Validation

Update functions to use validation:
```rust
use crate::validation::Validator;

async fn create_recording(recording: Vec<u8>, duration: i64) -> Result<...> {
    // Validate input
    Validator::audio_data(&recording).map_err(|e| e.to_string())?;

    if duration < 0 || duration > 3600000 {
        return Err("Invalid duration".to_string());
    }

    // Continue...
}

pub fn create_profile(&self, name: String, ...) -> Result<SettingsProfile> {
    // Validate input
    Validator::profile_name(&name)?;

    // Continue...
}

async fn transcribe_audio(config: &serde_json::Value, audio_data: Vec<u8>) -> Result<String, String> {
    // Validate base URL
    Validator::api_url(base_url).map_err(|e| e.to_string())?;

    // Continue...
}
```

---

## 4. Restrict Shell Access

**Priority**: 🟠 HIGH
**Time**: 5 minutes
**File**: `src-tauri/tauri.conf.json`

### Current (Insecure)
```json
{
  "plugins": {
    "shell": {
      "open": true  // ⚠️ Opens ANY URL
    }
  }
}
```

### Fixed (Secure)
```json
{
  "plugins": {
    "shell": {
      "open": "^https://whispo\\.app/.*|^https://docs\\.whispo\\.app/.*|^https://github\\.com/egoist/whispo/.*"
    }
  }
}
```

This allows only:
- `https://whispo.app/*`
- `https://docs.whispo.app/*`
- `https://github.com/egoist/whispo/*`

---

## 5. Add MCP Authentication

**Priority**: 🟠 HIGH
**Time**: 3-4 hours
**Files**: `src-tauri/src/mcp/server.rs`, `src-tauri/src/mcp/types.rs`

### Step 1: Add Auth Token Generation

```rust
use uuid::Uuid;
use std::collections::HashSet;

pub struct McpServer {
    // Existing fields...
    auth_tokens: Arc<Mutex<HashSet<String>>>,
}

impl McpServer {
    pub fn new() -> Self {
        let mut tokens = HashSet::new();

        // Generate initial token
        let initial_token = Uuid::new_v4().to_string();
        tokens.insert(initial_token.clone());

        // Save token to config or display to user
        println!("MCP Auth Token: {}", initial_token);

        Self {
            // Existing fields...
            auth_tokens: Arc::new(Mutex::new(tokens)),
        }
    }

    fn verify_auth(&self, request: &McpRequest) -> Result<bool> {
        let token = request.params
            .as_ref()
            .and_then(|p| p.get("auth_token"))
            .and_then(|t| t.as_str());

        if let Some(token) = token {
            let tokens = self.auth_tokens.lock().unwrap();
            Ok(tokens.contains(token))
        } else {
            Ok(false)
        }
    }

    pub async fn handle_request(&self, request: McpRequest) -> Result<McpResponse> {
        // Skip auth for initialize
        if request.method != "initialize" {
            if !self.verify_auth(&request)? {
                return Ok(McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(McpError {
                        code: -32001,
                        message: "Authentication required".to_string(),
                        data: None,
                    }),
                });
            }
        }

        // Continue with existing logic...
    }
}
```

### Step 2: Add Token Management Commands

```rust
#[tauri::command]
async fn mcp_generate_token(
    mcp_client: State<'_, Arc<McpClient>>
) -> Result<String, String> {
    let token = Uuid::new_v4().to_string();
    // Save to server's auth_tokens
    Ok(token)
}

#[tauri::command]
async fn mcp_revoke_token(
    mcp_client: State<'_, Arc<McpClient>>,
    token: String
) -> Result<(), String> {
    // Remove from server's auth_tokens
    Ok(())
}
```

---

## 6. Add Rate Limiting

**Priority**: 🟡 MEDIUM
**Time**: 2-3 hours
**File**: `src-tauri/src/rate_limiter.rs`

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct RateLimiter {
    limits: Mutex<HashMap<String, Vec<Instant>>>,
    max_calls: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_calls: usize, window_seconds: u64) -> Self {
        Self {
            limits: Mutex::new(HashMap::new()),
            max_calls,
            window: Duration::from_secs(window_seconds),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let mut limits = self.limits.lock().unwrap();
        let now = Instant::now();

        let calls = limits.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove old calls outside window
        calls.retain(|&instant| now.duration_since(instant) < self.window);

        // Check if under limit
        if calls.len() < self.max_calls {
            calls.push(now);
            true
        } else {
            false
        }
    }
}

// Usage in main.rs
lazy_static! {
    static ref TRANSCRIPTION_LIMITER: RateLimiter = RateLimiter::new(10, 60); // 10 calls per minute
}

#[tauri::command]
async fn create_recording(...) -> Result<...> {
    if !TRANSCRIPTION_LIMITER.check("create_recording") {
        return Err("Rate limit exceeded. Please wait before trying again.".to_string());
    }

    // Continue...
}
```

---

## 7. Restrict File System Access

**Priority**: 🟡 MEDIUM
**Time**: 10 minutes
**File**: `src-tauri/tauri.conf.json`

```json
{
  "plugins": {
    "fs": {
      "scope": [
        "$APPDATA/whispo/config.json",
        "$APPDATA/whispo/profiles.json",
        "$APPDATA/whispo/recordings/*.webm",
        "$APPDATA/whispo/history.json"
      ]
    }
  }
}
```

---

## 🧪 Testing Security Fixes

### 1. Test CSP
```bash
# Open DevTools
# Try to execute in console:
eval('alert("XSS")')
# Should fail with CSP error
```

### 2. Test API Key Encryption
```bash
# Check config file
cat ~/Library/Application\ Support/whispo/config.json
# Should NOT contain API keys

# Check keychain (macOS)
security find-generic-password -s whispo
# Should show entries
```

### 3. Test Input Validation
```typescript
// Try creating profile with invalid name
await invoke('create_profile', {
  name: '../../../etc/passwd',  // Should fail
  description: 'Test'
})
```

### 4. Test Rate Limiting
```typescript
// Rapidly call transcription
for (let i = 0; i < 20; i++) {
  await invoke('create_recording', { ... })
}
// Should fail after max calls
```

---

## 📚 Additional Resources

- [Tauri Security Documentation](https://tauri.app/v1/guides/security/)
- [OWASP Secure Coding Practices](https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [CSP Reference](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)

---

## 🎯 Implementation Checklist

### Critical (Do First)
- [ ] Enable Content Security Policy
- [ ] Implement encrypted API key storage
- [ ] Add input validation
- [ ] Restrict shell access

### High Priority
- [ ] Add MCP authentication
- [ ] Implement rate limiting
- [ ] Enforce HTTPS for APIs
- [ ] Restrict file system access

### Medium Priority
- [ ] Add security event logging
- [ ] Implement data retention policy
- [ ] Add privacy mode for keyboard events
- [ ] Configure code signing

---

**Next Steps**: Start with the Critical fixes, then work down the priority list. Test each fix thoroughly before moving to the next.
