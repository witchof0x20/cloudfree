// Copyright (C) 2026 Jade
// SPDX-License-Identifier: GPL-3.0-only

use crate::ai::ModelRegistry;
use crate::mcp::protocol::*;

pub fn list_tools() -> ToolsList {
    let models = ModelRegistry::get_all_models();
    let tools = models
        .into_iter()
        .map(|model| Tool {
            name: model.id.clone(),
            description: format!("{} - {}", model.name, model.description),
            input_schema: model.input_schema,
        })
        .collect();

    ToolsList { tools }
}

pub fn create_tool_result(result: serde_json::Value, is_error: bool) -> ToolResult {
    let text = if is_error {
        result.as_str().unwrap_or("Unknown error").to_string()
    } else {
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
    };

    ToolResult {
        content: vec![ContentBlock::Text { text }],
        is_error: if is_error { Some(true) } else { None },
    }
}
