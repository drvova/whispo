use super::tools::{get_whispo_tools, handle_tool_call};
use super::types::*;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Whispo as an MCP Server
/// Other applications can connect to Whispo to access dictation functionality
pub struct McpServer {
    info: ServerInfo,
    tools: Vec<McpTool>,
    resources: Vec<McpResource>,
    prompts: Vec<McpPrompt>,
    state: Arc<Mutex<ServerState>>,
}

struct ServerState {
    initialized: bool,
    client_capabilities: Option<serde_json::Value>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            info: ServerInfo {
                name: "Whispo".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                protocol_version: "2024-11-05".to_string(),
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
            },
            tools: get_whispo_tools(),
            resources: get_whispo_resources(),
            prompts: get_whispo_prompts(),
            state: Arc::new(Mutex::new(ServerState {
                initialized: false,
                client_capabilities: None,
            })),
        }
    }

    /// Handle incoming MCP requests
    pub async fn handle_request(&self, request: McpRequest) -> Result<McpResponse> {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "tools/list" => self.handle_list_tools(request).await,
            "tools/call" => self.handle_call_tool(request).await,
            "resources/list" => self.handle_list_resources(request).await,
            "resources/read" => self.handle_read_resource(request).await,
            "prompts/list" => self.handle_list_prompts(request).await,
            "prompts/get" => self.handle_get_prompt(request).await,
            _ => Ok(McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
            }),
        }
    }

    async fn handle_initialize(&self, request: McpRequest) -> Result<McpResponse> {
        let mut state = self.state.lock().unwrap();
        state.initialized = true;

        if let Some(params) = &request.params {
            state.client_capabilities = params.get("capabilities").cloned();
        }

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({
                "protocolVersion": self.info.protocol_version,
                "capabilities": self.info.capabilities,
                "serverInfo": {
                    "name": self.info.name,
                    "version": self.info.version
                }
            })),
            error: None,
        })
    }

    async fn handle_list_tools(&self, request: McpRequest) -> Result<McpResponse> {
        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({
                "tools": self.tools
            })),
            error: None,
        })
    }

    async fn handle_call_tool(&self, request: McpRequest) -> Result<McpResponse> {
        let params = request.params.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Missing params for tools/call")
        })?;

        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;

        let arguments = params
            .get("arguments")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let result = handle_tool_call(tool_name, arguments).await?;

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::to_value(result)?),
            error: None,
        })
    }

    async fn handle_list_resources(&self, request: McpRequest) -> Result<McpResponse> {
        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({
                "resources": self.resources
            })),
            error: None,
        })
    }

    async fn handle_read_resource(&self, request: McpRequest) -> Result<McpResponse> {
        let params = request.params.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Missing params for resources/read")
        })?;

        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing resource URI"))?;

        // Handle resource reading based on URI
        let content = self.read_resource(uri).await?;

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::to_value(content)?),
            error: None,
        })
    }

    async fn handle_list_prompts(&self, request: McpRequest) -> Result<McpResponse> {
        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({
                "prompts": self.prompts
            })),
            error: None,
        })
    }

    async fn handle_get_prompt(&self, request: McpRequest) -> Result<McpResponse> {
        let params = request.params.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Missing params for prompts/get")
        })?;

        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing prompt name"))?;

        let prompt = self.get_prompt(name, params.get("arguments")).await?;

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::to_value(prompt)?),
            error: None,
        })
    }

    async fn read_resource(&self, uri: &str) -> Result<ResourceContent> {
        // Parse URI and return appropriate resource
        match uri {
            "whispo://config" => Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: "application/json".to_string(),
                text: Some(serde_json::json!({"provider": "openai"}).to_string()),
                blob: None,
            }),
            "whispo://history" => Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: "application/json".to_string(),
                text: Some(serde_json::json!({"items": []}).to_string()),
                blob: None,
            }),
            _ => anyhow::bail!("Unknown resource URI: {}", uri),
        }
    }

    async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value> {
        match name {
            "transcription_help" => Ok(serde_json::json!({
                "messages": [{
                    "role": "user",
                    "content": "Help me improve my voice dictation accuracy"
                }]
            })),
            _ => anyhow::bail!("Unknown prompt: {}", name),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.state.lock().unwrap().initialized
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

fn get_whispo_resources() -> Vec<McpResource> {
    vec![
        McpResource {
            uri: "whispo://config".to_string(),
            name: "Whispo Configuration".to_string(),
            description: Some("Current Whispo configuration".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        McpResource {
            uri: "whispo://history".to_string(),
            name: "Transcription History".to_string(),
            description: Some("Recent transcription history".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        McpResource {
            uri: "whispo://glossary".to_string(),
            name: "User Glossary".to_string(),
            description: Some("Custom terms and replacements".to_string()),
            mime_type: Some("application/json".to_string()),
        },
    ]
}

fn get_whispo_prompts() -> Vec<McpPrompt> {
    vec![
        McpPrompt {
            name: "transcription_help".to_string(),
            description: Some("Get help improving transcription accuracy".to_string()),
            arguments: None,
        },
        McpPrompt {
            name: "format_transcript".to_string(),
            description: Some("Format a transcript for specific context".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "transcript".to_string(),
                    description: Some("The transcript to format".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "context".to_string(),
                    description: Some("Target context (code, email, etc.)".to_string()),
                    required: true,
                },
            ]),
        },
    ]
}
