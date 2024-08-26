use crate::WireframeSettings;
use anyhow::{Result, Context};
use serde::Deserialize;
use tracing::info;
use std::{env, path::PathBuf};

#[derive(Deserialize, Debug)]
pub struct JsonLineList {
    pub line_list: Vec<[u32; 2]>,
}

pub fn json_parse(settings: &WireframeSettings) -> Result<JsonLineList> {
    let gltf_path = settings.gltf_path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("No gltf path provided"))?;

        let path = build_asset_path(&gltf_path);

    if !path.exists() {
        return Err(anyhow::anyhow!("JSON file not found: {}", path.display()));
    }
    else{
      info!("JSON file found: {}", path.display());
    }

    let json_data = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    serde_json::from_str(&json_data)
        .with_context(|| "Failed to parse JSON")
}

fn build_asset_path(gltf_path: &str) -> PathBuf {
  // Try to get the assets directory from an environment variable
  let assets_dir = env::var("ASSETS_DIR").unwrap_or_else(|_| "assets".to_string());

  // Create a PathBuf for the assets directory
  let mut path = PathBuf::from(assets_dir);

  // Append the gltf_path and change the extension to .json
  path.push(gltf_path);
  path.set_extension("json");

  path
}