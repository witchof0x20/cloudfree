// Copyright (C) 2026 Jade
// SPDX-License-Identifier: GPL-3.0-only

use crate::ai::ModelRegistry;
use crate::mcp::protocol::*;
use serde_json::json;

pub fn list_resources() -> ResourcesList {
    let mut resources = vec![];

    // Add model info resources
    let models = ModelRegistry::get_all_models();
    for model in models {
        resources.push(Resource {
            uri: format!("model://{}", model.id),
            name: model.name.clone(),
            description: Some(model.description.clone()),
            mime_type: Some("application/json".to_string()),
        });
    }

    ResourcesList { resources }
}

pub fn get_resource_content(uri: &str) -> Option<ResourceContents> {
    if let Some(model_id) = uri.strip_prefix("model://") {
        if let Some(model) = ModelRegistry::get_model(model_id) {
            let info = json!({
                "id": model.id,
                "name": model.name,
                "description": model.description,
                "category": model.category,
                "base_neurons": model.base_neurons,
                "input_schema": model.input_schema,
            });

            return Some(ResourceContents {
                contents: vec![ResourceContent {
                    uri: uri.to_string(),
                    mime_type: "application/json".to_string(),
                    text: serde_json::to_string_pretty(&info).unwrap_or_else(|_| info.to_string()),
                }],
            });
        }
    }

    None
}
