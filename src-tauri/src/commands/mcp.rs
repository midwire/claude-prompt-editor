use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::mcp::tools::McpState;

#[derive(Clone)]
pub struct McpServerState {
    pub port: Arc<Mutex<Option<u16>>>,
    pub mcp_state: Arc<Mutex<Option<McpState>>>,
}

impl McpServerState {
    pub fn new() -> Self {
        Self {
            port: Arc::new(Mutex::new(None)),
            mcp_state: Arc::new(Mutex::new(None)),
        }
    }
}

pub async fn start_mcp(prompts_dir: PathBuf, port: u16) -> Result<u16, String> {
    let state = McpState::new(prompts_dir);
    crate::mcp::server::start_server(state, port).await
}
