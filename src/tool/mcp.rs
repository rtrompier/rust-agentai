//! # Model Context Protocol Tools
//!
//! This module external tools that can connect with MCP Servers.
//!
//! Supported connection types:
//! - `stdio`
//! - `http`
//!
//!

use crate::tool::{Tool, ToolBox, ToolError};
use anyhow::Result as AnyhowResult;
use async_trait::async_trait;
use log::{debug, info};
use rmcp::{
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    service::RunningService,
    transport::{ConfigureCommandExt, StreamableHttpClientTransport, TokioChildProcess},
    RoleClient, ServiceExt,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;

// Type aliases for the different client types we'll store
type ChildProcessClient = RunningService<RoleClient, ()>;
type HttpClient = RunningService<RoleClient, rmcp::model::InitializeRequestParam>;

pub struct McpToolBox {
    child_clients: HashMap<String, Arc<ChildProcessClient>>,
    http_clients: HashMap<String, Arc<HttpClient>>,
    tools: Vec<Tool>,
}

pub enum McpServer {
    ChildProcess(ChildProcess),
    StreamableHttp(StreamableHttp),
}

pub struct ChildProcess {
    pub command: String,
    pub args: Vec<String>,
}

pub struct StreamableHttp {
    pub url: String,
}

impl McpToolBox {
    pub async fn new(servers: Vec<McpServer>) -> AnyhowResult<Self> {
        let mut child_clients = HashMap::new();
        let mut http_clients = HashMap::new();
        let mut all_tools = Vec::new();

        for (idx, server) in servers.into_iter().enumerate() {
            let server_name = format!("server{}", idx);

            match server {
                McpServer::ChildProcess(child_process) => {
                    let client = ()
                        .serve(TokioChildProcess::new(
                            Command::new(child_process.command).configure(|cmd| {
                                cmd.args(child_process.args);
                            }),
                        )?)
                        .await?;

                    // Get server info and list tools
                    let server_info = client.peer_info();
                    info!("Connected to child process server: {server_info:#?}");

                    // List tools for this server
                    let tools_response = client.list_tools(Default::default()).await?;
                    for tool in tools_response.tools {
                        let name = format!("{}_{}", server_name, tool.name);
                        debug!("added stdio tool {name}");
                        all_tools.push(Tool {
                            name,
                            description: tool.description.map(|d| d.to_string()),
                            schema: Some(serde_json::to_value(tool.input_schema)?),
                        });
                    }

                    child_clients.insert(server_name.clone(), Arc::new(client));
                }
                McpServer::StreamableHttp(streamable_http) => {
                    let transport = StreamableHttpClientTransport::from_uri(streamable_http.url);
                    let client_info = ClientInfo {
                        protocol_version: Default::default(),
                        capabilities: ClientCapabilities::default(),
                        client_info: Implementation {
                            name: "sse-client".to_string(),
                            version: "0.0.1".to_string(),
                        },
                    };
                    let client = client_info.serve(transport).await?;

                    // Get server info and list tools
                    let server_info = client.peer_info();
                    info!("Connected to HTTP server: {server_info:#?}");

                    // List tools for this server
                    let tools_response = client.list_tools(Default::default()).await?;
                    for tool in tools_response.tools {
                        let name = format!("{}_{}", server_name, tool.name);
                        debug!("added http tool {name}");
                        all_tools.push(Tool {
                            name,
                            description: tool.description.map(|d| d.to_string()),
                            schema: Some(serde_json::to_value(tool.input_schema)?),
                        });
                    }

                    http_clients.insert(server_name.clone(), Arc::new(client));
                }
            };
        }

        Ok(Self {
            child_clients,
            http_clients,
            tools: all_tools,
        })
    }
    
    pub fn add_tool(&mut self, tool: Tool) {
        self.tools.push(tool);
    }
}

#[async_trait]
impl ToolBox for McpToolBox {
    fn tools_definitions(&self) -> Result<Vec<Tool>, ToolError> {
        Ok(self.tools.clone())
    }

    async fn call_tool(&self, tool_name: String, arguments: Value) -> Result<String, ToolError> {
        // Extract server name and actual tool name from the prefixed tool name
        let parts: Vec<String> = tool_name.splitn(2, '_').map(|s| s.to_string()).collect();
        if parts.len() != 2 {
            return Err(ToolError::NoToolFound(tool_name));
        }

        let server_name = &parts[0];
        let actual_tool_name = &parts[1];
        println!("server_name: {server_name}, actual_tool_name: {actual_tool_name}");

        // Try child process clients first
        if let Some(client) = self.child_clients.get(server_name) {
            let call_result = client
                .call_tool(CallToolRequestParam {
                    name: actual_tool_name.clone().into(),
                    arguments: Some(arguments.as_object().unwrap().clone()),
                })
                .await
                .map_err(anyhow::Error::new)?;

            // Convert the response content to string
            // For now, we'll serialize the entire response as JSON
            let response_json = serde_json::to_string(&call_result.content)
                .unwrap_or_else(|_| "Unable to serialize response".to_string());

            return Ok(response_json);
        }

        // Try HTTP clients
        if let Some(client) = self.http_clients.get(server_name) {
            let call_result = client
                .call_tool(CallToolRequestParam {
                    name: actual_tool_name.clone().into(),
                    arguments: Some(arguments.as_object().unwrap().clone()),
                })
                .await
                .map_err(anyhow::Error::new)?;

            // Convert the response content to string
            // For now, we'll serialize the entire response as JSON
            let response_json = serde_json::to_string(&call_result.content)
                .unwrap_or_else(|_| "Unable to serialize response".to_string());

            return Ok(response_json);
        }

        Err(ToolError::NoToolFound(actual_tool_name.to_string()))
    }

    fn add_tool(&mut self, tool: Tool) {
        self.tools.push(tool);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result as AnyhowResult;
    use serde_json::json;

    // Helper function to create a McpToolBox for testing
    async fn create_test_toolbox() -> AnyhowResult<McpToolBox> {
        let child_process = ChildProcess {
            command: "uvx".to_string(),
            args: vec![
                "mcp-server-time".to_string(),
                "--local-timezone".to_string(),
                "UTC".to_string(),
            ],
        };
        McpToolBox::new(vec![McpServer::ChildProcess(child_process)]).await
    }

    #[tokio::test]
    async fn test_new_and_tools_definitions() -> AnyhowResult<()> {
        let mcp_tools = create_test_toolbox().await?;

        let tool_defs = mcp_tools.tools_definitions()?;

        // Assert that we get at least one tool definition
        assert!(tool_defs.len() >= 1);

        // Assert that tools have the server prefix (now using server0_ instead of server_0_)
        let tools_with_prefix: Vec<_> = tool_defs
            .iter()
            .filter(|t| t.name.starts_with("server0_"))
            .collect();
        assert!(!tools_with_prefix.is_empty());

        // Print available tools for debugging
        println!("Available tools:");
        for tool in &tool_defs {
            println!("  - {}", tool.name);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_call_tool_convert_time() -> AnyhowResult<()> {
        let mcp_tools = create_test_toolbox().await?;

        // First, get the available tools to see what's actually available
        let tool_defs = mcp_tools.tools_definitions()?;
        let convert_time_tool = tool_defs
            .iter()
            .find(|t| t.name.contains("convert_time"))
            .expect("convert_time tool should be available");

        // Call the 'convert_time' tool with required arguments (using the actual tool name)
        let arguments = json!({
            "source_timezone": "Europe/Warsaw",
            "target_timezone": "America/New_York",
            "time": "12:00"
        });
        let result = mcp_tools
            .call_tool(convert_time_tool.name.clone(), arguments)
            .await?;

        // Assert that the result is a non-empty string (the converted time)
        assert!(!result.is_empty());
        println!("Convert time result: {}", result);

        Ok(())
    }

    #[tokio::test]
    async fn test_call_tool_invalid_tool() -> AnyhowResult<()> {
        let mcp_tools = create_test_toolbox().await?;

        // Call a non-existent tool
        let arguments = json!({});
        let result = mcp_tools
            .call_tool("non_existent_tool".to_string(), arguments)
            .await;

        // Assert that calling a non-existent tool returns an error
        assert!(result.is_err());

        Ok(())
    }
}
