use axum::{extract::State, routing::post, Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::mcp::tools::McpState;

type SharedState = Arc<McpState>;

async fn handle_mcp(State(state): State<SharedState>, Json(body): Json<Value>) -> Json<Value> {
    let id = body.get("id").cloned().unwrap_or(Value::Null);
    let method = body
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("");

    let result = match method {
        "initialize" => {
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "claude-prompt-editor",
                    "version": "0.1.0"
                }
            })
        }
        "tools/list" => {
            json!({
                "tools": [
                    {
                        "name": "load_prompt",
                        "description": "Load a prompt by name, stripping frontmatter and interpolating variables",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string", "description": "Prompt file name (without .md extension)" }
                            },
                            "required": ["name"]
                        }
                    },
                    {
                        "name": "list_prompts",
                        "description": "List all available prompts",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "set_variable",
                        "description": "Set a variable for a specific prompt",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "prompt_name": { "type": "string", "description": "Prompt name" },
                                "key": { "type": "string", "description": "Variable key" },
                                "value": { "type": "string", "description": "Variable value" }
                            },
                            "required": ["prompt_name", "key", "value"]
                        }
                    },
                    {
                        "name": "get_prompt_health",
                        "description": "Run the linter on a prompt and return lint findings as JSON",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string", "description": "Prompt file name (without .md extension)" }
                            },
                            "required": ["name"]
                        }
                    }
                ]
            })
        }
        "tools/call" => {
            let params = body.get("params").cloned().unwrap_or(json!({}));
            let tool_name = params
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            match tool_name {
                "load_prompt" => {
                    let name = arguments
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    match crate::mcp::tools::load_prompt(&state, name).await {
                        Ok(content) => json!({
                            "content": [{ "type": "text", "text": content }]
                        }),
                        Err(e) => json!({
                            "isError": true,
                            "content": [{ "type": "text", "text": e }]
                        }),
                    }
                }
                "list_prompts" => {
                    match crate::mcp::tools::list_prompts(&state).await {
                        Ok(entries) => {
                            let names: Vec<String> =
                                entries.iter().map(|e| e.name.clone()).collect();
                            json!({
                                "content": [{ "type": "text", "text": serde_json::to_string(&names).unwrap_or_default() }]
                            })
                        }
                        Err(e) => json!({
                            "isError": true,
                            "content": [{ "type": "text", "text": e }]
                        }),
                    }
                }
                "set_variable" => {
                    let prompt_name = arguments
                        .get("prompt_name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    let key = arguments
                        .get("key")
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    let value = arguments
                        .get("value")
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    crate::mcp::tools::set_variable(&state, prompt_name, key, value).await;
                    json!({
                        "content": [{ "type": "text", "text": "Variable set successfully" }]
                    })
                }
                "get_prompt_health" => {
                    let name = arguments
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    match crate::mcp::tools::get_prompt_health(&state, name) {
                        Ok(json_str) => json!({
                            "content": [{ "type": "text", "text": json_str }]
                        }),
                        Err(e) => json!({
                            "isError": true,
                            "content": [{ "type": "text", "text": e }]
                        }),
                    }
                }
                _ => json!({
                    "isError": true,
                    "content": [{ "type": "text", "text": format!("Unknown tool: {}", tool_name) }]
                }),
            }
        }
        _ => {
            return Json(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            }));
        }
    };

    Json(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    }))
}

/// Start the MCP HTTP server on the given port. Returns the actual port bound.
pub async fn start_server(mcp_state: McpState, port: u16) -> Result<u16, String> {
    let shared_state: SharedState = Arc::new(mcp_state);

    let app = Router::new()
        .route("/mcp", post(handle_mcp))
        .with_state(shared_state);

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

    let actual_port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local addr: {}", e))?
        .port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    Ok(actual_port)
}
