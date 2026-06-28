use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;

use crate::appearances::config::AppearanceEdit;
use crate::edit;

#[derive(Debug, Clone, Serialize)]
pub struct LoadedAppConfig {
    pub urls: HashMap<String, String>,
    pub edits: Vec<AppearanceEdit>,
}

pub fn load_app_config(config_path: &Path) -> Result<LoadedAppConfig, String> {
    let urls = edit::config::load_config_values(config_path).map_err(|e| e.to_string())?;
    let edits = crate::appearances::config::load_edits(config_path).map_err(|e| e.to_string())?;
    Ok(LoadedAppConfig { urls, edits })
}
