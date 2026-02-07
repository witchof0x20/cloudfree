// Copyright (C) 2026 Jade
// SPDX-License-Identifier: GPL-3.0-only

use worker::*;
use crate::mcp::protocol::*;
use crate::mcp::{tools, resources};
use crate::ai::AiBridge;
use serde_json::json;

pub struct McpServer;

impl McpServer {
    /// Returns None for notifications (no response needed), Some for requests.
    pub async fn handle_request(env: &Env, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let method = req.method.as_str();
        let id = req.id.clone();

        // Notifications (no id) don't get a response
        if id.is_none() || id.as_ref() == Some(&serde_json::Value::Null) {
            match method {
                "notifications/initialized" | "notifications/cancelled" => {}
                _ => console_log!("Unhandled notification: {}", method),
            }
            return None;
        }

        let result = match method {
            "initialize" => Self::handle_initialize(),
            "ping" => Ok(json!({})),
            "tools/list" => Self::handle_tools_list(),
            "tools/call" => Self::handle_tools_call(env, req.params).await,
            "resources/list" => Self::handle_resources_list(),
            "resources/read" => Self::handle_resources_read(req.params),
            _ => return Some(JsonRpcResponse::error(id, -32601, format!("Method not found: {}", method))),
        };

        Some(match result {
            Ok(value) => JsonRpcResponse::success(id, value),
            Err(e) => JsonRpcResponse::error(id, -32603, e),
        })
    }

    fn handle_initialize() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {
                "tools": {
                    "listChanged": false
                },
                "resources": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "cloudfree-mcp",
                "version": "0.1.0"
            }
        }))
    }

    fn handle_tools_list() -> Result<serde_json::Value, String> {
        let tools_list = tools::list_tools();
        serde_json::to_value(tools_list).map_err(|e| e.to_string())
    }

    async fn handle_tools_call(env: &Env, params: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
        let params: CallToolParams = serde_json::from_value(params.unwrap_or(json!({})))
            .map_err(|e| format!("Invalid params: {}", e))?;

        let result = AiBridge::run_inference(env, &params.name, params.arguments.unwrap_or(json!({})))
            .await
            .map_err(|e| format!("AI inference failed: {}", e))?;

        // Include neurons used in the response
        let mut tool_result = tools::create_tool_result(result.result, false);

        // Add neurons info to the text response
        if let Some(ContentBlock::Text { text }) = tool_result.content.first_mut() {
            *text = format!("{}\n\n[Neurons used: {}]", text, result.neurons_used);
        }

        serde_json::to_value(tool_result).map_err(|e| e.to_string())
    }

    fn handle_resources_list() -> Result<serde_json::Value, String> {
        let resources_list = resources::list_resources();
        serde_json::to_value(resources_list).map_err(|e| e.to_string())
    }

    fn handle_resources_read(params: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
        let params: ReadResourceParams = serde_json::from_value(params.unwrap_or(json!({})))
            .map_err(|e| format!("Invalid params: {}", e))?;

        let contents = resources::get_resource_content(&params.uri)
            .ok_or_else(|| format!("Resource not found: {}", params.uri))?;

        serde_json::to_value(contents).map_err(|e| e.to_string())
    }
}
