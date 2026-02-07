// Copyright (C) 2026 Jade
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AiRequest {
    pub model: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiResponse {
    pub result: serde_json::Value,
    pub neurons_used: u32,
}
