// Copyright (C) 2026 Jade
// SPDX-License-Identifier: GPL-3.0-only

use worker::*;
use crate::ai::{ModelRegistry, AiResponse};
use wasm_bindgen::prelude::*;
use js_sys::Promise;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    pub type CloudflareAI;

    #[wasm_bindgen(structural, method)]
    fn run(this: &CloudflareAI, model: &str, input: &JsValue) -> Promise;
}

pub struct AiBridge;

impl AiBridge {
    pub async fn run_inference(
        env: &Env,
        model_id: &str,
        input: serde_json::Value,
    ) -> Result<AiResponse> {
        let model = ModelRegistry::get_model(model_id)
            .ok_or_else(|| Error::RustError(format!("Unknown model: {}", model_id)))?;

        let estimated_neurons = model.estimate_neurons(&input);

        // Transform input to match Cloudflare AI API format
        let ai_input = Self::format_input_for_model(model_id, input)?;

        console_log!("Calling AI with model: {}, input: {}", model_id, serde_json::to_string(&ai_input).unwrap_or_default());

        // Get AI binding from environment
        // Access the env as a JsValue to get the AI binding
        unsafe {
            let env_ptr = env as *const Env as *const JsValue;
            let env_js = &*env_ptr;

            let ai_binding = js_sys::Reflect::get(env_js, &JsValue::from_str("AI"))
                .map_err(|_| Error::RustError("Failed to get AI binding from env".to_string()))?;

            // Serialize input using JSON.parse for guaranteed correct format
            let input_json = serde_json::to_string(&ai_input)
                .map_err(|e| Error::RustError(format!("Failed to serialize to JSON: {}", e)))?;

            console_log!("JSON input: {}", input_json);

            let input_js = js_sys::JSON::parse(&input_json)
                .map_err(|e| Error::RustError(format!("Failed to parse JSON: {:?}", e)))?;

            // Get the run method
            let run_fn = js_sys::Reflect::get(&ai_binding, &JsValue::from_str("run"))
                .map_err(|_| Error::RustError("Failed to get run method".to_string()))?
                .dyn_into::<js_sys::Function>()
                .map_err(|_| Error::RustError("run is not a function".to_string()))?;

            // Call AI.run(model, input) with the AI binding as `this`
            let model_js = JsValue::from_str(model_id);
            let promise = run_fn.call2(&ai_binding, &model_js, &input_js)
                .map_err(|e| Error::RustError(format!("Failed to call AI.run: {:?}", e)))?
                .dyn_into::<Promise>()
                .map_err(|_| Error::RustError("AI.run did not return a promise".to_string()))?;

            let result = wasm_bindgen_futures::JsFuture::from(promise).await
                .map_err(|e| Error::RustError(format!("AI inference failed: {:?}", e)))?;

            // Parse the result
            let ai_result: serde_json::Value = serde_wasm_bindgen::from_value(result)
                .map_err(|e| Error::RustError(format!("Failed to parse AI result: {}", e)))?;

            console_log!("AI result: {}", serde_json::to_string(&ai_result).unwrap_or_default());

            // Extract neurons_used from response, fallback to estimate
            let neurons_used = ai_result.get("neurons_used")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(estimated_neurons);

            Ok(AiResponse {
                result: ai_result,
                neurons_used,
            })
        }
    }

    fn format_input_for_model(model_id: &str, input: serde_json::Value) -> Result<serde_json::Value> {
        // Format input according to model type
        if model_id.contains("llama") || model_id.contains("mistral") {
            // Text generation models - use simple prompt format
            let prompt = input.get("prompt")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::RustError("Missing 'prompt' field".to_string()))?;

            Ok(serde_json::json!({ "prompt": prompt }))
        } else if model_id.contains("bge") {
            // Embedding models expect { text: "..." } or { text: [...] }
            let text = input.get("text")
                .ok_or_else(|| Error::RustError("Missing 'text' field".to_string()))?;

            Ok(serde_json::json!({ "text": text }))
        } else if model_id.contains("stable-diffusion") {
            // Image generation models expect { prompt: "..." }
            let prompt = input.get("prompt")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::RustError("Missing 'prompt' field".to_string()))?;

            Ok(serde_json::json!({ "prompt": prompt }))
        } else if model_id.contains("whisper") {
            // Whisper expects { audio: [...] }
            Ok(input)
        } else {
            // Default: pass through
            Ok(input)
        }
    }
}
