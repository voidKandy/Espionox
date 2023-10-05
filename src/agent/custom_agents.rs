use serde::Deserialize;
use std::fs;

use super::Agent;

use crate::configuration::GLOBAL_CONFIG_PATH;
// Add this to your existing code
#[derive(Debug, Deserialize)]
struct AgentThreadConfig {
    name: String,
    // Add other fields for settings...
}

fn toml_path() -> std::path::PathBuf {
    let base_path = GLOBAL_CONFIG_PATH;
    base_path.join("/agents")
}
