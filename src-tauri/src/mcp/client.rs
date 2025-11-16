use super::types::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command as TokioCommand;
use uuid::Uuid;

pub struct McpClient {
    servers: Arc<Mutex<HashMap<String, McpServerConnection>>>,
    config: Arc<Mutex<McpConfiguration>>,
}

struct McpServerConnection {
    name: String,
    process: Option<Child>,
    tools: Vec<McpTool>,
    resources: Vec<McpResource>,
    prompts: Vec<McpPrompt>,
    capabilities: ServerCapabilities,
}

impl McpClient {
    pub fn new(config: McpConfiguration) -> Self {
        Self {
            servers: Arc::new(Mutex::new(HashMap::new())),
            config: Arc::new(Mutex::new(config)),
        }
    }

    /// Initialize all configured MCP servers
    pub async fn initialize(&self) -> Result<()> {
        let config = self.config.lock().unwrap().clone();

        if !config.enabled {
            return Ok(());
        }

        for (name, server_config) in config.servers {
            if !server_config.enabled {
                continue;
            }

            if let Err(e) = self.connect_server(name.clone(), server_config).await {
                eprintln!("Failed to connect to MCP server '{}': {}", name, e);
            }
        }

        Ok(())
    }

    /// Connect to a single MCP server
    async fn connect_server(&self, name: String, config: McpServerConfig) -> Result<()> {
        // Start the server process
        let mut cmd = TokioCommand::new(&config.command);
        cmd.args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(env) = &config.env {
            for (key, value) in env {
                cmd.env(key, value);
            }
        }

        let child = cmd.spawn().context("Failed to spawn MCP server process")?;

        // Initialize connection
        let initialize_request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string(),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": { "subscribe": true }
                },
                "clientInfo": {
                    "name": "Whispo",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
        };

        // Store connection
        let connection = McpServerConnection {
            name: name.clone(),
            process: None, // Will be set after successful init
            tools: Vec::new(),
            resources: Vec::new(),
            prompts: Vec::new(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: false,
                }),
                resources: Some(ResourcesCapability {
                    subscribe: false,
                    list_changed: false,
                }),
                prompts: Some(PromptsCapability {
                    list_changed: false,
                }),
                logging: Some(LoggingCapability {}),
            },
        };

        self.servers.lock().unwrap().insert(name, connection);

        Ok(())
    }

    /// List all available tools from all servers
    pub async fn list_tools(&self) -> Result<Vec<McpTool>> {
        let mut all_tools = Vec::new();

        let servers = self.servers.lock().unwrap();
        for connection in servers.values() {
            all_tools.extend(connection.tools.clone());
        }

        Ok(all_tools)
    }

    /// Call a tool on an MCP server
    pub async fn call_tool(&self, tool_name: &str, arguments: HashMap<String, serde_json::Value>) -> Result<ToolResult> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": tool_name,
                "arguments": arguments
            })),
        };

        // For now, return a simulated result
        // Full implementation would send request via stdio to server process
        Ok(ToolResult {
            content: vec![ToolContent::Text {
                text: format!("Tool '{}' called successfully", tool_name),
            }],
            is_error: false,
        })
    }

    /// Get transcription context from MCP servers
    pub async fn get_transcription_context(&self) -> Result<TranscriptionContext> {
        let config = self.config.lock().unwrap().clone();

        let mut context = TranscriptionContext {
            active_application: None,
            active_file: None,
            project_context: None,
            user_glossary: Vec::new(),
            recent_interactions: Vec::new(),
        };

        if !config.enabled {
            return Ok(context);
        }

        // Get active file context if enabled
        if config.context_awareness.use_file_context {
            context.active_file = self.get_active_file_context().await.ok();
        }

        // Get project context if enabled
        if config.context_awareness.use_project_context {
            context.project_context = self.get_project_context().await.ok();
        }

        // Get glossary if enabled
        if config.context_awareness.use_glossary {
            context.user_glossary = self.get_glossary().await.unwrap_or_default();
        }

        Ok(context)
    }

    /// Get active file context from editor MCP server
    async fn get_active_file_context(&self) -> Result<FileContext> {
        let args = HashMap::new();
        let result = self.call_tool("get_active_file", args).await?;

        // Parse result into FileContext
        // For now, return a placeholder
        Ok(FileContext {
            path: "/path/to/file".to_string(),
            name: "file.txt".to_string(),
            language: Some("plaintext".to_string()),
            cursor_position: None,
            selected_text: None,
        })
    }

    /// Get project context
    async fn get_project_context(&self) -> Result<ProjectContext> {
        let args = HashMap::new();
        let result = self.call_tool("get_project_info", args).await?;

        Ok(ProjectContext {
            name: "current-project".to_string(),
            root_path: "/path/to/project".to_string(),
            language: None,
            framework: None,
        })
    }

    /// Get user glossary
    async fn get_glossary(&self) -> Result<Vec<GlossaryEntry>> {
        let args = HashMap::new();
        let result = self.call_tool("get_glossary", args).await?;

        Ok(vec![
            GlossaryEntry {
                term: "API".to_string(),
                replacement: "A P I".to_string(),
                context: Some("technical".to_string()),
            },
        ])
    }

    /// Enhance transcript with MCP context
    pub async fn enhance_transcript(&self, original: &str) -> Result<String> {
        let context = self.get_transcription_context().await?;

        // Build enhancement prompt with context
        let mut enhanced = original.to_string();

        // Apply glossary replacements
        for entry in &context.user_glossary {
            enhanced = enhanced.replace(&entry.term, &entry.replacement);
        }

        // Could call LLM with context for further enhancement
        // For now, return with basic enhancements

        Ok(enhanced)
    }

    /// Shutdown all server connections
    pub async fn shutdown(&self) -> Result<()> {
        let mut servers = self.servers.lock().unwrap();

        for (name, connection) in servers.drain() {
            if let Some(mut process) = connection.process {
                let _ = process.kill();
            }
        }

        Ok(())
    }

    /// Update configuration
    pub fn update_config(&self, new_config: McpConfiguration) -> Result<()> {
        *self.config.lock().unwrap() = new_config;
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> McpConfiguration {
        self.config.lock().unwrap().clone()
    }

    /// Check if MCP is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.lock().unwrap().enabled
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        // Clean shutdown of all servers
        let mut servers = self.servers.lock().unwrap();
        for (_name, connection) in servers.drain() {
            if let Some(mut process) = connection.process {
                let _ = process.kill();
            }
        }
    }
}
