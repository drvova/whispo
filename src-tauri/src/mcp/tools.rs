use super::types::*;
use anyhow::Result;
use std::collections::HashMap;

/// Define all MCP tools that Whispo provides
pub fn get_whispo_tools() -> Vec<McpTool> {
    vec![
        // Transcription history tool
        McpTool {
            name: "get_transcription_history".to_string(),
            description: "Get recent transcription history from Whispo".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of items to return",
                        "default": 10
                    },
                    "since": {
                        "type": "string",
                        "description": "ISO timestamp to get items after",
                        "format": "date-time"
                    }
                }
            }),
        },
        // Start dictation tool
        McpTool {
            name: "start_dictation".to_string(),
            description: "Start voice dictation in Whispo".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "context": {
                        "type": "string",
                        "description": "Context hint for dictation (code, email, etc.)"
                    }
                }
            }),
        },
        // Get configuration tool
        McpTool {
            name: "get_dictation_config".to_string(),
            description: "Get current Whispo configuration".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        // Update glossary tool
        McpTool {
            name: "update_glossary".to_string(),
            description: "Update user glossary for better transcription".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "entries": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "term": { "type": "string" },
                                "replacement": { "type": "string" },
                                "context": { "type": "string" }
                            },
                            "required": ["term", "replacement"]
                        }
                    }
                },
                "required": ["entries"]
            }),
        },
        // Get active profile tool
        McpTool {
            name: "get_active_profile".to_string(),
            description: "Get the currently active Whispo profile".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        // Switch profile tool
        McpTool {
            name: "switch_profile".to_string(),
            description: "Switch to a different Whispo profile".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "profile_id": {
                        "type": "string",
                        "description": "ID of the profile to switch to"
                    }
                },
                "required": ["profile_id"]
            }),
        },
        // Transcribe audio tool
        McpTool {
            name: "transcribe_audio".to_string(),
            description: "Transcribe audio using Whispo's configured providers".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "audio_path": {
                        "type": "string",
                        "description": "Path to audio file to transcribe"
                    },
                    "provider": {
                        "type": "string",
                        "enum": ["openai", "groq", "auto"],
                        "description": "Provider to use for transcription"
                    },
                    "context": {
                        "type": "string",
                        "description": "Context to improve transcription accuracy"
                    }
                },
                "required": ["audio_path"]
            }),
        },
    ]
}

/// Handle tool calls for Whispo MCP server
pub async fn handle_tool_call(
    tool_name: &str,
    arguments: HashMap<String, serde_json::Value>,
) -> Result<ToolResult> {
    match tool_name {
        "get_transcription_history" => handle_get_history(arguments).await,
        "start_dictation" => handle_start_dictation(arguments).await,
        "get_dictation_config" => handle_get_config(arguments).await,
        "update_glossary" => handle_update_glossary(arguments).await,
        "get_active_profile" => handle_get_active_profile(arguments).await,
        "switch_profile" => handle_switch_profile(arguments).await,
        "transcribe_audio" => handle_transcribe_audio(arguments).await,
        _ => Ok(ToolResult {
            content: vec![ToolContent::Text {
                text: format!("Unknown tool: {}", tool_name),
            }],
            is_error: true,
        }),
    }
}

async fn handle_get_history(args: HashMap<String, serde_json::Value>) -> Result<ToolResult> {
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;

    // In real implementation, would query actual history
    let history_json = serde_json::json!({
        "items": [],
        "total": 0,
        "limit": limit
    });

    Ok(ToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&history_json)?,
        }],
        is_error: false,
    })
}

async fn handle_start_dictation(args: HashMap<String, serde_json::Value>) -> Result<ToolResult> {
    let context = args
        .get("context")
        .and_then(|v| v.as_str())
        .unwrap_or("generic");

    Ok(ToolResult {
        content: vec![ToolContent::Text {
            text: format!("Started dictation with context: {}", context),
        }],
        is_error: false,
    })
}

async fn handle_get_config(_args: HashMap<String, serde_json::Value>) -> Result<ToolResult> {
    // In real implementation, would get actual config
    let config = serde_json::json!({
        "provider": "openai",
        "model": "whisper-1",
        "enabled": true
    });

    Ok(ToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&config)?,
        }],
        is_error: false,
    })
}

async fn handle_update_glossary(args: HashMap<String, serde_json::Value>) -> Result<ToolResult> {
    let entries = args.get("entries").and_then(|v| v.as_array());

    let count = entries.map(|e| e.len()).unwrap_or(0);

    Ok(ToolResult {
        content: vec![ToolContent::Text {
            text: format!("Updated glossary with {} entries", count),
        }],
        is_error: false,
    })
}

async fn handle_get_active_profile(
    _args: HashMap<String, serde_json::Value>,
) -> Result<ToolResult> {
    let profile = serde_json::json!({
        "id": "default",
        "name": "Default Profile",
        "active": true
    });

    Ok(ToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&profile)?,
        }],
        is_error: false,
    })
}

async fn handle_switch_profile(args: HashMap<String, serde_json::Value>) -> Result<ToolResult> {
    let profile_id = args
        .get("profile_id")
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    Ok(ToolResult {
        content: vec![ToolContent::Text {
            text: format!("Switched to profile: {}", profile_id),
        }],
        is_error: false,
    })
}

async fn handle_transcribe_audio(args: HashMap<String, serde_json::Value>) -> Result<ToolResult> {
    let audio_path = args
        .get("audio_path")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if audio_path.is_empty() {
        return Ok(ToolResult {
            content: vec![ToolContent::Text {
                text: "Error: audio_path is required".to_string(),
            }],
            is_error: true,
        });
    }

    // In real implementation, would actually transcribe the audio
    Ok(ToolResult {
        content: vec![ToolContent::Text {
            text: format!("Transcribed audio from: {}", audio_path),
        }],
        is_error: false,
    })
}
