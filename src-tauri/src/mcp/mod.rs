// Model Context Protocol implementation for Whispo
// Provides context-aware transcription and tool integration

pub mod types;
pub mod client;
pub mod server;
pub mod tools;

pub use client::McpClient;
pub use server::McpServer;
pub use types::*;
