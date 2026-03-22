//! MCP Client Module for aiTerm
//!
//! Provides MCP (Model Context Protocol) client functionality for connecting
//! to MCP servers and exposing their tools to the LLM.

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Server '{0}' not found")]
    ServerNotFound(String),

    #[error("Process spawn failed: {0}")]
    SpawnError(String),

    #[error("Communication error: {0}")]
    CommunicationError(String),

    #[error("Tool '{tool}' not found on server '{server}'")]
    ToolNotFound { tool: String, server: String },

    #[error("Tool execution failed: {0}")]
    ToolExecutionError(String),

    #[error("JSON-RPC error: {0}")]
    JsonRpcError(String),

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Server process exited: {0}")]
    ProcessExited(i32),
}

impl From<std::io::Error> for McpError {
    fn from(e: std::io::Error) -> Self {
        McpError::CommunicationError(e.to_string())
    }
}

impl From<serde_json::Error> for McpError {
    fn from(e: serde_json::Error) -> Self {
        McpError::JsonRpcError(e.to_string())
    }
}

// ============================================================================
// Configuration Types
// ============================================================================

/// Configuration for a single MCP server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
}

impl McpServerConfig {
    pub fn new(name: &str, command: &str, args: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            env: None,
        }
    }
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            command: String::new(),
            args: Vec::new(),
            env: None,
        }
    }
}

// ============================================================================
// MCP Protocol Types
// ============================================================================

/// JSON-RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: &str, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(id),
            method: method.to_string(),
            params,
        }
    }
}

/// JSON-RPC success response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcSuccessResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub result: serde_json::Value,
}

/// JSON-RPC error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub error: JsonRpcErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcErrorDetail {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC response (can be success or error)
#[derive(Debug, Clone, Deserialize)]
    #[serde(untagged)]
pub enum JsonRpcResponse {
    Success(JsonRpcSuccessResponse),
    Error(JsonRpcErrorResponse),
}

// ============================================================================
// Tool Types
// ============================================================================

/// Tool definition exposed by MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

impl Tool {
    pub fn new(name: &str, description: &str, input_schema: serde_json::Value) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            input_schema,
        }
    }
}

/// Tool call from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Result from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub id: String,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

// ============================================================================
// MCP Client (Single Server Connection)
// ============================================================================

/// Client for a single MCP server via stdio
pub struct McpClient {
    request_id: Arc<Mutex<u64>>,
    #[allow(dead_code)]
    child: Child,
    writer: Arc<Mutex<BufWriter<tokio::process::ChildStdin>>>,
    reader: Arc<Mutex<BufReader<tokio::process::ChildStdout>>>,
}

impl McpClient {
    /// Connect to an MCP server
    pub async fn connect(config: &McpServerConfig) -> Result<Self, McpError> {
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args);

        // Set up environment
        if let Some(env) = &config.env {
            for (key, value) in env {
                cmd.env(key, value);
            }
        }

        // Set up stdio for JSON-RPC communication
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        let mut child = cmd.spawn().map_err(|e| McpError::SpawnError(e.to_string()))?;

        let stdin = child.stdin.take().ok_or_else(|| {
            McpError::CommunicationError("Failed to take stdin".to_string())
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            McpError::CommunicationError("Failed to take stdout".to_string())
        })?;

        let reader = BufReader::new(stdout);
        let writer = BufWriter::new(stdin);

        Ok(Self {
            request_id: Arc::new(Mutex::new(1)),
            child,
            writer: Arc::new(Mutex::new(writer)),
            reader: Arc::new(Mutex::new(reader)),
        })
    }

    /// Send a JSON-RPC request and wait for response
    pub async fn send_request(&self, method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value, McpError> {
        let id = {
            let mut id = self.request_id.lock().await;
            let current_id = *id;
            *id += 1;
            current_id
        };

        let request = JsonRpcRequest::new(id, method, params);
        let request_json = serde_json::to_string(&request)?;
        let request_line = format!("{}\n", request_json);

        // Send request
        {
            let mut writer = self.writer.lock().await;
            writer.write_all(request_line.as_bytes()).await?;
            writer.flush().await?;
        }

        // Wait for response
        let mut reader = self.reader.lock().await;
        let mut line = String::new();

        // Set a timeout for reading (30 seconds)
        let timeout_duration = Duration::from_secs(30);
        let deadline = std::time::Instant::now() + timeout_duration;

        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return Err(McpError::Timeout);
            }

            match tokio::time::timeout(remaining, reader.read_line(&mut line)).await {
                Ok(Ok(0)) => {
                    return Err(McpError::CommunicationError("EOF received".to_string()));
                }
                Ok(Ok(_)) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    // Try to parse the response
                    let response: JsonRpcResponse = match serde_json::from_str(&line) {
                        Ok(r) => r,
                        Err(_) => {
                            line.clear();
                            continue;
                        }
                    };

                    match response {
                        JsonRpcResponse::Success(success) => {
                            return Ok(success.result);
                        }
                        JsonRpcResponse::Error(error) => {
                            return Err(McpError::JsonRpcError(error.error.message));
                        }
                    }
                }
                Ok(Err(e)) => {
                    return Err(McpError::CommunicationError(e.to_string()));
                }
                Err(_) => {
                    return Err(McpError::Timeout);
                }
            }
        }
    }

    /// Initialize the MCP connection
    pub async fn initialize(&self, client_info: serde_json::Value) -> Result<serde_json::Value, McpError> {
        self.send_request("initialize", Some(client_info)).await
    }

    /// List available tools
    pub async fn list_tools(&self) -> Result<Vec<Tool>, McpError> {
        let result = self.send_request("tools/list", None).await?;

        #[derive(Deserialize)]
        struct ToolsListResult {
            tools: Vec<Tool>,
        }

        let tools_result: ToolsListResult = serde_json::from_value(result)?;
        Ok(tools_result.tools)
    }

    /// Call a tool
    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Serialize)]
        struct CallToolParams {
            name: String,
            #[serde(rename = "input")]
            arguments: serde_json::Value,
        }

        let params = serde_json::to_value(CallToolParams {
            name: name.to_string(),
            arguments,
        })?;

        self.send_request("tools/call", Some(params)).await
    }
}

// ============================================================================
// MCP Manager (Multiple Servers)
// ============================================================================

/// Manages multiple MCP server connections and aggregates their tools
pub struct McpManager {
    clients: HashMap<String, McpClient>,
    tools: Vec<Tool>,
}

impl McpManager {
    /// Create a new manager and connect to all configured servers
    pub async fn new(servers: &[McpServerConfig]) -> Result<Self, McpError> {
        let mut clients = HashMap::new();
        let mut all_tools = Vec::new();

        for config in servers {
            let client = McpClient::connect(config).await?;

            // Initialize the connection
            client.initialize(get_client_info()).await?;

            let tools = client.list_tools().await?;

            // Prepend server name to tool names to avoid conflicts
            for mut tool in tools {
                tool.name = format!("{}/{}", config.name, tool.name);
                all_tools.push(tool);
            }

            clients.insert(config.name.clone(), client);
        }

        Ok(Self {
            clients,
            tools: all_tools,
        })
    }

    /// Get all available tools from all servers
    pub fn get_tools(&self) -> Vec<Tool> {
        self.tools.clone()
    }

    /// Call a tool by its full name (format: "server_name/tool_name")
    pub async fn call_tool(&self, full_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let parts: Vec<&str> = full_name.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(McpError::ToolExecutionError(format!(
                "Invalid tool name format: {}. Expected 'server/tool'",
                full_name
            )));
        }

        let (server_name, tool_name) = (parts[0], parts[1]);

        let client = self.clients.get(server_name).ok_or_else(|| {
            McpError::ServerNotFound(server_name.to_string())
        })?;

        client.call_tool(tool_name, arguments).await
    }

    /// Check if any servers are connected
    pub fn is_connected(&self) -> bool {
        !self.clients.is_empty()
    }

    /// Get server names
    pub fn server_names(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }
}

// ============================================================================
// MCP Client Info
// ============================================================================

/// Client information for MCP handshake
pub fn get_client_info() -> serde_json::Value {
    serde_json::json!({
        "name": "aiTerm",
        "version": "0.1.0",
        "protocolVersion": "2024-11-05"
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_config_default() {
        let config = McpServerConfig::default();
        assert!(config.name.is_empty());
        assert!(config.command.is_empty());
        assert!(config.args.is_empty());
        assert!(config.env.is_none());
    }

    #[test]
    fn test_mcp_server_config_new() {
        let config = McpServerConfig::new("test", "npx", vec!["-y", "server"]);
        assert_eq!(config.name, "test");
        assert_eq!(config.command, "npx");
        assert_eq!(config.args, vec!["-y", "server"]);
    }

    #[test]
    fn test_tool_new() {
        let tool = Tool::new(
            "test_tool",
            "A test tool",
            serde_json::json!({"type": "object", "properties": {}})
        );
        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");
    }

    #[test]
    fn test_json_rpc_request() {
        let request = JsonRpcRequest::new(
            1,
            "tools/list",
            None
        );
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tools/list");
        assert!(request.params.is_none());
    }

    #[test]
    fn test_json_rpc_request_with_params() {
        let request = JsonRpcRequest::new(
            2,
            "tools/call",
            Some(serde_json::json!({"name": "test", "input": {}}))
        );
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tools/call");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_get_client_info() {
        let info = get_client_info();
        assert_eq!(info["name"], "aiTerm");
        assert_eq!(info["version"], "0.1.0");
    }
}
