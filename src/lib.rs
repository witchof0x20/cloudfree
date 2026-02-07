// Copyright (C) 2026 Jade
// SPDX-License-Identifier: GPL-3.0-only

use worker::*;

mod ai;
mod mcp;

use mcp::{JsonRpcRequest, McpServer};

fn cors_headers() -> Headers {
    let headers = Headers::new();
    let _ = headers.set("Access-Control-Allow-Origin", "*");
    let _ = headers.set("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS");
    let _ = headers.set(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization, Mcp-Session-Id, Mcp-Protocol-Version",
    );
    headers
}

/// Build a JSON response with CORS headers, preserving Content-Type.
fn json_response<B: serde::Serialize>(value: &B) -> Result<Response> {
    let headers = cors_headers();
    headers.set("Content-Type", "application/json")?;
    Response::from_json(value).map(|r| r.with_headers(headers))
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    if req.method() == Method::Options {
        return Ok(Response::builder()
            .with_headers(cors_headers())
            .with_status(204)
            .empty());
    }

    let url = req.url()?;
    let path = url.path();

    match (req.method(), path.as_ref()) {
        (Method::Get, "/health") => {
            let headers = cors_headers();
            headers.set("Content-Type", "text/plain")?;
            Response::ok("OK").map(|r| r.with_headers(headers))
        }
        (Method::Post, "/mcp") => handle_mcp(req, env).await,
        // GET and DELETE on /mcp: 405 per MCP spec
        (Method::Get | Method::Delete, "/mcp") => Ok(Response::builder()
            .with_headers(cors_headers())
            .with_status(405)
            .empty()),
        _ => {
            let headers = cors_headers();
            Response::error("Not found", 404).map(|r| r.with_headers(headers))
        }
    }
}

async fn handle_mcp(mut req: Request, env: Env) -> Result<Response> {
    // Optional authentication
    if let Ok(secret) = env.secret("MCP_AUTH_TOKEN") {
        let auth_token = secret.to_string();
        let provided_token = req
            .headers()
            .get("Authorization")?
            .and_then(|h| h.strip_prefix("Bearer ").map(|s| s.to_string()));

        if provided_token.as_deref() != Some(auth_token.as_str()) {
            return Response::error("Unauthorized", 401).map(|r| r.with_headers(cors_headers()));
        }
    }

    let json_req: JsonRpcRequest = match req.json().await {
        Ok(req) => req,
        Err(e) => {
            console_log!("Failed to parse request: {}", e);
            return Response::error("Invalid JSON-RPC request", 400)
                .map(|r| r.with_headers(cors_headers()));
        }
    };

    match McpServer::handle_request(&env, json_req).await {
        Some(response) => json_response(&response),
        None => {
            // Notifications get HTTP 202 with no body
            Ok(Response::builder()
                .with_status(202)
                .with_headers(cors_headers())
                .empty())
        }
    }
}
