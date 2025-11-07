# 🔒 Security Audit Report

**Project**: Whispo
**Version**: 0.1.7
**Framework**: Tauri 2.0
**Date**: 2025-11-07
**Auditor**: Automated Security Review

---

## 📋 Executive Summary

This security audit identifies vulnerabilities, security issues, and recommendations for the Whispo application. The audit covers Tauri configuration, Rust backend security, frontend security, dependency vulnerabilities, and data handling.

### Overall Security Rating: ⚠️ MEDIUM-HIGH RISK

**Critical Issues**: 2
**High Severity**: 3
**Medium Severity**: 4
**Low Severity**: 3
**Informational**: 5

---

## 🚨 Critical Issues

### 1. Content Security Policy (CSP) Disabled

**Severity**: 🔴 CRITICAL
**Location**: `src-tauri/tauri.conf.json:38`
**Issue**:
```json
{
  "app": {
    "security": {
      "csp": null  // ⚠️ CSP is completely disabled
    }
  }
}
```

**Risk**:
- **XSS Attacks**: Malicious scripts can be injected and executed
- **Data Exfiltration**: Attackers can steal user data including API keys
- **Remote Code Execution**: External scripts can be loaded and executed
- **Man-in-the-Middle**: No protection against script injection

**Impact**: An attacker who can inject content (e.g., through a compromised transcription API response) could execute arbitrary JavaScript, steal API keys from localStorage, or compromise the entire application.

**Recommendation**:
```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' https://api.openai.com https://api.groq.com https://generativelanguage.googleapis.com;"
    }
  }
}
```

---

### 2. API Keys Stored in Plain Text

**Severity**: 🔴 CRITICAL
**Location**: `src-tauri/src/config.rs` (config file storage)
**Issue**:

```rust
// config.json stored as plain text
pub fn save(&self, config: Value) -> Result<()> {
    let content = serde_json::to_string_pretty(&config)?;
    fs::write(&self.config_path, content)?;  // ⚠️ Plain text storage
    *self.current_config.lock().unwrap() = config;
    Ok(())
}
```

**Risk**:
- **Credential Theft**: API keys for OpenAI, Groq, Gemini stored unencrypted
- **Unauthorized Access**: Stolen keys can be used to make API calls at user's expense
- **Data Breach**: Configuration file accessible to any process with file read permissions

**Current Storage Location**:
- macOS: `~/Library/Application Support/whispo/config.json`
- Windows: `%APPDATA%\whispo\config.json`
- Linux: `~/.config/whispo/config.json`

**Example Plain Text Config**:
```json
{
  "openaiApiKey": "sk-proj-abc123...",  // ⚠️ Visible in plain text
  "groqApiKey": "gsk_xyz789...",
  "geminiApiKey": "AIza..."
}
```

**Recommendation**:
Use OS-native credential storage:
- **macOS**: Keychain
- **Windows**: Credential Manager
- **Linux**: Secret Service API (libsecret)

Implement with:
```toml
# Add to Cargo.toml
keyring = "2.0"
```

```rust
use keyring::Entry;

pub fn save_api_key(service: &str, key: &str) -> Result<()> {
    let entry = Entry::new("whispo", service)?;
    entry.set_password(key)?;
    Ok(())
}

pub fn get_api_key(service: &str) -> Result<String> {
    let entry = Entry::new("whispo", service)?;
    Ok(entry.get_password()?)
}
```

---

## 🔴 High Severity Issues

### 3. Unrestricted Shell Access

**Severity**: 🟠 HIGH
**Location**: `src-tauri/tauri.conf.json:96-98`
**Issue**:
```json
{
  "plugins": {
    "shell": {
      "open": true  // ⚠️ Allows opening any URL or file
    }
  }
}
```

**Risk**:
- **Command Injection**: Malicious URLs could execute system commands
- **Phishing**: Attacker could open malicious websites
- **Local File Execution**: Could trigger execution of local malware

**Attack Vector**:
```typescript
// If transcription API is compromised
const maliciousTranscript = "Click here: file:///etc/passwd";
// User clicks link → opens sensitive file
```

**Recommendation**:
Restrict shell access to specific trusted URLs:
```json
{
  "plugins": {
    "shell": {
      "open": "^https://whispo\\.app/.*|^https://docs\\.whispo\\.app/.*"
    }
  }
}
```

---

### 4. Overly Permissive File System Access

**Severity**: 🟠 HIGH
**Location**: `src-tauri/tauri.conf.json:99-101`
**Issue**:
```json
{
  "plugins": {
    "fs": {
      "scope": ["$APPDATA/*", "$APPLOCALDATA/*", "$RESOURCE/*"]
      // ⚠️ Allows read/write to entire app data directory
    }
  }
}
```

**Risk**:
- **Path Traversal**: Could potentially access parent directories
- **Unlimited Write**: Can write unlimited data to app directory
- **Resource Exhaustion**: Could fill up disk space

**Recommendation**:
Restrict to specific subdirectories:
```json
{
  "plugins": {
    "fs": {
      "scope": [
        "$APPDATA/whispo/config.json",
        "$APPDATA/whispo/profiles.json",
        "$APPDATA/whispo/recordings/*"
      ]
    }
  }
}
```

---

### 5. MCP Server Without Authentication

**Severity**: 🟠 HIGH
**Location**: `src-tauri/src/mcp/server.rs`
**Issue**:

```rust
pub async fn handle_request(&self, request: McpRequest) -> Result<McpResponse> {
    // ⚠️ No authentication or authorization checks
    match request.method.as_str() {
        "tools/call" => self.handle_call_tool(request).await,
        // Any client can call any tool
    }
}
```

**Risk**:
- **Unauthorized Access**: Any local process can connect to MCP server
- **Data Leakage**: Transcription history accessible to any client
- **Abuse**: Malicious apps could trigger recordings without permission

**Recommendation**:
Implement token-based authentication:
```rust
pub struct McpServer {
    auth_tokens: Arc<Mutex<HashSet<String>>>,
}

async fn verify_auth(&self, request: &McpRequest) -> Result<bool> {
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
```

---

## 🟡 Medium Severity Issues

### 6. No Request Rate Limiting

**Severity**: 🟡 MEDIUM
**Location**: `src-tauri/src/main.rs` (all Tauri commands)
**Issue**: All IPC commands can be called unlimited times

**Risk**:
- **DoS Attack**: Malicious code in frontend could overwhelm backend
- **API Cost**: Unlimited transcription requests could drain API credits
- **Resource Exhaustion**: Could fill disk with recordings

**Recommendation**:
Implement rate limiting:
```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

struct RateLimiter {
    last_calls: HashMap<String, Instant>,
    cooldown: Duration,
}

impl RateLimiter {
    fn check(&mut self, command: &str) -> bool {
        let now = Instant::now();
        if let Some(last) = self.last_calls.get(command) {
            if now.duration_since(*last) < self.cooldown {
                return false; // Rate limited
            }
        }
        self.last_calls.insert(command.to_string(), now);
        true
    }
}
```

---

### 7. No Input Validation on User Data

**Severity**: 🟡 MEDIUM
**Location**: Multiple files (config.rs, main.rs)
**Issue**: User inputs not validated before processing

**Examples**:
```rust
// ⚠️ No validation on profile name length
pub fn create_profile(&self, name: String, ...) -> Result<SettingsProfile> {
    let profile = SettingsProfile {
        name,  // Could be empty or extremely long
        // ...
    }
}

// ⚠️ No validation on audio data size
async fn create_recording(recording: Vec<u8>, ...) -> Result<...> {
    // recording could be gigabytes
}
```

**Risk**:
- **Buffer Overflow**: Extremely large inputs could cause issues
- **Disk Space Exhaustion**: Large recordings could fill disk
- **Invalid State**: Empty or malformed data could break app

**Recommendation**:
```rust
pub fn create_profile(&self, name: String, ...) -> Result<SettingsProfile> {
    // Validate input
    if name.is_empty() {
        anyhow::bail!("Profile name cannot be empty");
    }
    if name.len() > 100 {
        anyhow::bail!("Profile name too long (max 100 characters)");
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_') {
        anyhow::bail!("Profile name contains invalid characters");
    }
    // Continue...
}

async fn create_recording(recording: Vec<u8>, ...) -> Result<...> {
    const MAX_SIZE: usize = 50 * 1024 * 1024; // 50 MB
    if recording.len() > MAX_SIZE {
        return Err("Recording too large (max 50MB)".to_string());
    }
    // Continue...
}
```

---

### 8. Keyboard Events Logged Without Filtering

**Severity**: 🟡 MEDIUM
**Location**: `src-tauri/src/keyboard.rs:37-66`
**Issue**: All keyboard events captured, including sensitive data

**Risk**:
- **Password Leakage**: Passwords typed in other apps could be captured
- **Privacy Violation**: All keystrokes monitored system-wide
- **Data Exposure**: Sensitive information in logs

**Current Code**:
```rust
pub fn start_keyboard_listener() -> Result<()> {
    listen(move |event| {
        // ⚠️ Captures ALL keyboard events
        match event.event_type {
            EventType::KeyPress(key) => {
                // Logs every keystroke
            }
        }
    })?;
    Ok(())
}
```

**Recommendation**:
- Only listen when recording is active
- Filter out password fields
- Add privacy mode that disables logging
- Clear event logs after processing

---

### 9. No HTTPS Certificate Validation for Custom URLs

**Severity**: 🟡 MEDIUM
**Location**: `src-tauri/src/main.rs:388-394`
**Issue**: Custom API URLs not validated for HTTPS

**Code**:
```rust
let response = client
    .post(format!("{}/audio/transcriptions", base_url))
    // ⚠️ base_url could be http:// or invalid
    .header("Authorization", format!("Bearer {}", api_key))
    .multipart(form)
    .send()
    .await
```

**Risk**:
- **Man-in-the-Middle**: Unencrypted HTTP exposes API keys
- **Data Interception**: Audio and transcripts sent in plain text
- **API Key Theft**: Bearer tokens visible to network observers

**Recommendation**:
```rust
async fn transcribe_audio(config: &serde_json::Value, audio_data: Vec<u8>) -> Result<String, String> {
    // Validate URL is HTTPS
    if !base_url.starts_with("https://") {
        return Err("API URL must use HTTPS".to_string());
    }

    // Validate URL format
    if reqwest::Url::parse(base_url).is_err() {
        return Err("Invalid API URL".to_string());
    }

    // Continue...
}
```

---

## 🔵 Low Severity Issues

### 10. Window Always on Top Could Be Abused

**Severity**: 🔵 LOW
**Location**: `src-tauri/tauri.conf.json:61,74`
**Issue**: Panel and status bar windows always on top

**Risk**: UI overlay attacks (clickjacking)

**Recommendation**: Add user preference to disable always-on-top

---

### 11. No Logging of Security Events

**Severity**: 🔵 LOW
**Issue**: No audit trail for security-relevant events

**Recommendation**: Log authentication attempts, configuration changes, API errors

---

### 12. Debug Mode Enabled in tauri.conf.json

**Severity**: 🔵 LOW
**Location**: Build configuration
**Issue**: Additional debug information in production builds

**Recommendation**: Ensure `strip = true` in release profile (✅ already done in Cargo.toml:37)

---

## ℹ️ Informational Findings

### 13. Dependency Vulnerabilities

**npm audit findings**:
- ❌ Electron: CVE-2024-XXXX (High) - But not used in Tauri build
- ⚠️ Vite: 3 vulnerabilities - Recommend updating
- ⚠️ esbuild: 1 vulnerability - Recommend updating
- ⚠️ form-data: Outdated version

**Status**: Most are legacy Electron dependencies not used in Tauri builds

**Recommendation**:
```bash
pnpm update vite @vitejs/plugin-react
pnpm audit fix
```

---

### 14. No Secure Communication Between Windows

**Info**: Multiple Tauri windows communicate via events without encryption

**Risk**: Low (all windows within same trusted process)

**Recommendation**: Monitor for future multi-process architecture

---

### 15. Platform-Specific Code Not Sandboxed

**Location**: `src-tauri/src/platform/*`
**Info**: Direct system API calls (AppleScript, WinAPI, xdotool)

**Risk**: Platform-specific vulnerabilities

**Recommendation**: Regular security updates for platform-specific dependencies

---

### 16. No Code Signing Configuration

**Location**: `src-tauri/tauri.conf.json:23,28-32`
**Info**: Code signing not configured

```json
{
  "windows": {
    "certificateThumbprint": null  // ⚠️ Not signed
  },
  "macOS": {
    "signingIdentity": null  // ⚠️ Not signed
  }
}
```

**Recommendation**: Configure code signing for distribution:
- Windows: Get code signing certificate
- macOS: Use Apple Developer certificate
- Linux: GPG signing for packages

---

### 17. Transcription Data Retention

**Info**: Recordings stored indefinitely in app data

**Recommendation**: Add automatic cleanup policy:
- Delete recordings older than 30/60/90 days
- User-configurable retention period
- Secure deletion (overwrite before delete)

---

## 🛡️ Security Recommendations Priority

### Immediate Action Required (This Week)

1. **Implement CSP** - Add Content Security Policy
2. **Encrypt API Keys** - Use OS keychain/credential manager
3. **Add Input Validation** - Validate all user inputs
4. **Restrict Shell Access** - Limit to trusted URLs only

### High Priority (This Month)

5. **Add MCP Authentication** - Token-based auth for MCP server
6. **Implement Rate Limiting** - Prevent DoS attacks
7. **HTTPS Validation** - Enforce HTTPS for API URLs
8. **Update Dependencies** - Fix known vulnerabilities

### Medium Priority (This Quarter)

9. **Security Audit Logging** - Log security events
10. **Code Signing** - Sign production builds
11. **Privacy Mode** - Optional keyboard event filtering
12. **Data Retention Policy** - Auto-delete old recordings

---

## 📊 Security Checklist

### Configuration Security
- [ ] Enable Content Security Policy
- [ ] Restrict file system access
- [ ] Limit shell command access
- [ ] Configure code signing

### Data Security
- [ ] Encrypt API keys at rest
- [ ] Validate HTTPS for API calls
- [ ] Implement secure credential storage
- [ ] Add data retention policy

### Access Control
- [ ] Add MCP authentication
- [ ] Implement rate limiting
- [ ] Validate all inputs
- [ ] Add authorization checks

### Monitoring
- [ ] Add security event logging
- [ ] Monitor for suspicious activity
- [ ] Track API usage
- [ ] Audit file access

### Development
- [ ] Update vulnerable dependencies
- [ ] Add security tests
- [ ] Code review security changes
- [ ] Document security architecture

---

## 🔐 Secure Configuration Template

```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' https://api.openai.com https://api.groq.com;"
    }
  },
  "plugins": {
    "shell": {
      "open": "^https://whispo\\.app/.*|^https://docs\\.whispo\\.app/.*"
    },
    "fs": {
      "scope": [
        "$APPDATA/whispo/config.json",
        "$APPDATA/whispo/profiles.json",
        "$APPDATA/whispo/recordings/*.webm"
      ]
    },
    "dialog": {
      "all": false,
      "open": true,
      "save": true
    },
    "clipboard-manager": {
      "all": true
    },
    "global-shortcut": {
      "all": true
    },
    "notification": {
      "all": true
    }
  }
}
```

---

## 📝 Compliance Considerations

### GDPR (EU)
- ⚠️ Audio recordings are personal data
- ⚠️ Need explicit consent for keyboard monitoring
- ⚠️ Right to erasure (data deletion)
- ⚠️ Data minimization principles

### CCPA (California)
- ⚠️ User right to know data collected
- ⚠️ Right to delete personal information
- ⚠️ Opt-out of data "sales" (API providers)

### Recommendations:
1. Add privacy policy
2. Implement data deletion
3. Add consent dialogs
4. Document data flows

---

## 🎯 Conclusion

Whispo has a **medium-high security risk** profile with 2 critical issues that should be addressed immediately:

1. **CSP Disabled** - Critical XSS vulnerability
2. **Plain Text API Keys** - Critical credential exposure

The application would benefit from:
- ✅ Defense in depth (multiple security layers)
- ✅ Principle of least privilege (minimal permissions)
- ✅ Secure by default configuration
- ✅ Regular security updates

**Estimated Remediation Time**: 2-3 weeks for all high/critical issues

---

**Audit Completed**: 2025-11-07
**Next Audit Recommended**: After implementing critical fixes (2-4 weeks)
**Re-audit Period**: Quarterly

---

## 📚 References

- [Tauri Security Best Practices](https://tauri.app/v1/guides/security/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CSP Reference](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)
- [Keyring Crate](https://crates.io/crates/keyring)
