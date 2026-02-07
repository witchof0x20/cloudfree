// Copyright (C) 2026 Jade
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: ModelCategory,
    pub base_neurons: u32,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelCategory {
    #[serde(rename = "llm")]
    Llm,
    #[serde(rename = "embedding")]
    Embedding,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "audio")]
    Audio,
}

impl ModelInfo {
    pub fn estimate_neurons(&self, input: &serde_json::Value) -> u32 {
        match self.category {
            ModelCategory::Llm => {
                let prompt = input.get("prompt")
                    .and_then(|p| p.as_str())
                    .unwrap_or("");
                let tokens = (prompt.len() / 4).max(1) as u32;
                tokens + 100
            }
            ModelCategory::Embedding => {
                let text = input.get("text")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                let tokens = (text.len() / 4).max(1) as u32;
                tokens / 10
            }
            ModelCategory::Image => 5000,
            ModelCategory::Audio => {
                input.get("audio")
                    .and_then(|a| a.as_str())
                    .map(|s| (s.len() / 1000).max(1) as u32 * 10)
                    .unwrap_or(100)
            }
        }
    }
}

pub struct ModelRegistry;

impl ModelRegistry {
    pub fn get_all_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "@cf/meta/llama-3.1-8b-instruct".to_string(),
                name: "Llama 3.1 8B Instruct".to_string(),
                description: "Meta's Llama 3.1 8B instruction-tuned model for text generation".to_string(),
                category: ModelCategory::Llm,
                base_neurons: 100,
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "string",
                            "description": "The text prompt to generate from"
                        },
                        "max_tokens": {
                            "type": "integer",
                            "description": "Maximum tokens to generate",
                            "default": 256
                        }
                    },
                    "required": ["prompt"]
                }),
            },
            ModelInfo {
                id: "@cf/mistral/mistral-7b-instruct-v0.1".to_string(),
                name: "Mistral 7B Instruct".to_string(),
                description: "Mistral's 7B instruction-tuned model for text generation".to_string(),
                category: ModelCategory::Llm,
                base_neurons: 90,
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "string",
                            "description": "The text prompt to generate from"
                        },
                        "max_tokens": {
                            "type": "integer",
                            "description": "Maximum tokens to generate",
                            "default": 256
                        }
                    },
                    "required": ["prompt"]
                }),
            },
            ModelInfo {
                id: "@cf/baai/bge-base-en-v1.5".to_string(),
                name: "BGE Base English v1.5".to_string(),
                description: "BAAI's text embedding model for semantic search and similarity".to_string(),
                category: ModelCategory::Embedding,
                base_neurons: 10,
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "The text to generate embeddings for"
                        }
                    },
                    "required": ["text"]
                }),
            },
            ModelInfo {
                id: "@cf/stabilityai/stable-diffusion-xl-base-1.0".to_string(),
                name: "Stable Diffusion XL".to_string(),
                description: "Stability AI's SDXL model for high-quality image generation".to_string(),
                category: ModelCategory::Image,
                base_neurons: 5000,
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "string",
                            "description": "The text prompt describing the image to generate"
                        },
                        "num_steps": {
                            "type": "integer",
                            "description": "Number of denoising steps",
                            "default": 20
                        }
                    },
                    "required": ["prompt"]
                }),
            },
            ModelInfo {
                id: "@cf/openai/whisper".to_string(),
                name: "Whisper".to_string(),
                description: "OpenAI's Whisper model for speech recognition and transcription".to_string(),
                category: ModelCategory::Audio,
                base_neurons: 100,
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "audio": {
                            "type": "string",
                            "description": "Base64-encoded audio data"
                        },
                        "language": {
                            "type": "string",
                            "description": "Language code (e.g., 'en' for English)"
                        }
                    },
                    "required": ["audio"]
                }),
            },
            // Additional LLM models
            ModelInfo {
                id: "@cf/meta/llama-3.1-70b-instruct".to_string(),
                name: "Llama 3.1 70B Instruct".to_string(),
                description: "Meta's Llama 3.1 70B large-scale multilingual instruction model".to_string(),
                category: ModelCategory::Llm,
                base_neurons: 300,
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "prompt": { "type": "string", "description": "The text prompt" },
                        "max_tokens": { "type": "integer", "default": 256 }
                    },
                    "required": ["prompt"]
                }),
            },
            ModelInfo {
                id: "@cf/meta/llama-3.2-1b-instruct".to_string(),
                name: "Llama 3.2 1B Instruct".to_string(),
                description: "Meta's Llama 3.2 1B small multilingual dialogue model".to_string(),
                category: ModelCategory::Llm,
                base_neurons: 50,
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "prompt": { "type": "string", "description": "The text prompt" },
                        "max_tokens": { "type": "integer", "default": 256 }
                    },
                    "required": ["prompt"]
                }),
            },
            ModelInfo {
                id: "@cf/qwen/qwen2.5-coder-32b-instruct".to_string(),
                name: "Qwen 2.5 Coder 32B".to_string(),
                description: "Qwen's code-specific model for programming tasks".to_string(),
                category: ModelCategory::Llm,
                base_neurons: 200,
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "prompt": { "type": "string", "description": "The code prompt" },
                        "max_tokens": { "type": "integer", "default": 512 }
                    },
                    "required": ["prompt"]
                }),
            },
            // Additional embedding models
            ModelInfo {
                id: "@cf/baai/bge-large-en-v1.5".to_string(),
                name: "BGE Large English v1.5".to_string(),
                description: "BAAI's large 1024-dimensional English embeddings".to_string(),
                category: ModelCategory::Embedding,
                base_neurons: 15,
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "Text to embed" }
                    },
                    "required": ["text"]
                }),
            },
            ModelInfo {
                id: "@cf/baai/bge-m3".to_string(),
                name: "BGE M3".to_string(),
                description: "BAAI's multi-functional, multilingual, multi-granular embeddings".to_string(),
                category: ModelCategory::Embedding,
                base_neurons: 20,
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "Text to embed" }
                    },
                    "required": ["text"]
                }),
            },
            // Additional image generation models
            ModelInfo {
                id: "@cf/black-forest-labs/flux-1-schnell".to_string(),
                name: "Flux 1 Schnell".to_string(),
                description: "Black Forest Labs' fast 12B parameter image generation model".to_string(),
                category: ModelCategory::Image,
                base_neurons: 4000,
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "prompt": { "type": "string", "description": "Image description" },
                        "num_steps": { "type": "integer", "default": 4 }
                    },
                    "required": ["prompt"]
                }),
            },
            ModelInfo {
                id: "@cf/bytedance/stable-diffusion-xl-lightning".to_string(),
                name: "Stable Diffusion XL Lightning".to_string(),
                description: "ByteDance's high-quality 1024px image generation in few steps".to_string(),
                category: ModelCategory::Image,
                base_neurons: 3500,
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "prompt": { "type": "string", "description": "Image description" },
                        "num_steps": { "type": "integer", "default": 8 }
                    },
                    "required": ["prompt"]
                }),
            },
        ]
    }

    pub fn get_model(id: &str) -> Option<ModelInfo> {
        // First check if it's in our curated list
        if let Some(model) = Self::get_all_models().into_iter().find(|m| m.id == id) {
            return Some(model);
        }

        // Fallback: dynamically create model info based on ID pattern
        Self::create_dynamic_model(id)
    }

    fn create_dynamic_model(id: &str) -> Option<ModelInfo> {
        // For models not in our curated list, infer category from ID
        let (category, base_neurons, input_schema) = if id.contains("llama")
            || id.contains("mistral")
            || id.contains("qwen")
            || id.contains("gemma")
            || id.contains("deepseek")
            || id.contains("gpt")
            || id.contains("phi")
            || id.contains("falcon")
            || id.contains("hermes")
            || id.contains("openchat")
            || id.contains("sqlcoder")
            || id.contains("neural-chat")
            || id.contains("openhermes")
            || id.contains("zephyr")
            || id.contains("starling")
            || id.contains("cybertron")
            || id.contains("chat")
            || id.contains("instruct")
            || id.contains("granite") {
            (ModelCategory::Llm, 100, json!({
                "type": "object",
                "properties": {
                    "prompt": { "type": "string", "description": "Text prompt" }
                },
                "required": ["prompt"]
            }))
        } else if id.contains("bge")
            || id.contains("embedding")
            || id.contains("embed") {
            (ModelCategory::Embedding, 10, json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Text to embed" }
                },
                "required": ["text"]
            }))
        } else if id.contains("stable-diffusion")
            || id.contains("flux")
            || id.contains("dreamshaper")
            || id.contains("lucid")
            || id.contains("phoenix") {
            (ModelCategory::Image, 5000, json!({
                "type": "object",
                "properties": {
                    "prompt": { "type": "string", "description": "Image description" }
                },
                "required": ["prompt"]
            }))
        } else if id.contains("whisper")
            || id.contains("nova")
            || id.contains("asr") {
            (ModelCategory::Audio, 100, json!({
                "type": "object",
                "properties": {
                    "audio": { "type": "string", "description": "Base64 audio" }
                },
                "required": ["audio"]
            }))
        } else {
            // Unknown model - default to LLM
            (ModelCategory::Llm, 100, json!({
                "type": "object",
                "properties": {
                    "prompt": { "type": "string" }
                },
                "required": ["prompt"]
            }))
        };

        Some(ModelInfo {
            id: id.to_string(),
            name: id.split('/').last().unwrap_or(id).replace('-', " ").to_string(),
            description: format!("Auto-detected model: {}", id),
            category,
            base_neurons,
            input_schema,
        })
    }
}
