use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use crate::appearances::{self, AppearanceEdit};
use crate::config_loader;
use crate::edit;
use crate::repack;
use crate::win2mac;

#[derive(Debug, Deserialize)]
pub struct EditClientArgs {
    pub tibia_exe: String,
    pub config_path: Option<String>,
    pub config_values: Option<HashMap<String, String>>,
    pub source_tibia_exe: Option<String>,
    pub strict_client_check: Option<bool>,
    pub aggressive_client_check: Option<bool>,
}

#[tauri::command]
pub fn edit_client(args: EditClientArgs) -> Result<edit::EditOutput, String> {
    let tibia_exe = PathBuf::from(&args.tibia_exe);
    let config_path = args.config_path.as_ref().map(PathBuf::from);
    let source_path = args.source_tibia_exe.as_ref().map(|p| PathBuf::from(p));
    edit::edit_client(
        tibia_exe.as_path(),
        config_path.as_ref().map(|p| p.as_path()),
        args.config_values,
        source_path.as_ref().map(|p| p.as_path()),
        args.strict_client_check.unwrap_or(false),
        args.aggressive_client_check.unwrap_or(false),
    )
    .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct DiagnoseClientArgs {
    pub tibia_exe: String,
    pub compare_with: Option<String>,
    pub strict_client_check: Option<bool>,
}

#[tauri::command]
pub fn diagnose_client(args: DiagnoseClientArgs) -> Result<edit::DiagnoseOutput, String> {
    let tibia_exe = PathBuf::from(&args.tibia_exe);
    let compare_path = args.compare_with.as_ref().map(|p| PathBuf::from(p));
    edit::diagnose_client(
        tibia_exe.as_path(),
        compare_path.as_ref().map(|p| p.as_path()),
        args.strict_client_check.unwrap_or(false),
    )
    .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct LoadConfigArgs {
    pub config_path: String,
}

#[tauri::command]
pub fn load_config_file(args: LoadConfigArgs) -> Result<config_loader::LoadedAppConfig, String> {
    config_loader::load_app_config(PathBuf::from(&args.config_path).as_path())
}

#[derive(Debug, Deserialize)]
pub struct RepackArgs {
    pub src: String,
    pub dst: String,
    pub platform: String,
}

#[tauri::command]
pub fn repack_client(args: RepackArgs) -> Result<repack::RepackResult, String> {
    repack::repack(
        PathBuf::from(&args.src).as_path(),
        PathBuf::from(&args.dst).as_path(),
        &args.platform,
    )
    .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct Win2MacArgs {
    pub src: String,
    pub dst: String,
}

#[tauri::command]
pub fn win2mac_assets(args: Win2MacArgs) -> Result<win2mac::Win2MacResult, String> {
    win2mac::win2mac(
        PathBuf::from(&args.src).as_path(),
        PathBuf::from(&args.dst).as_path(),
    )
    .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct AppearancesArgs {
    pub appearances_path: String,
    pub config_path: Option<String>,
    pub edits: Option<Vec<AppearanceEdit>>,
    pub output_path: Option<String>,
}

#[tauri::command]
pub fn edit_appearances(args: AppearancesArgs) -> Result<appearances::AppearancesResult, String> {
    let appearances_path = PathBuf::from(&args.appearances_path);
    let config_path = args.config_path.as_ref().map(PathBuf::from);
    let output_path = args.output_path.as_ref().map(PathBuf::from);
    appearances::edit_appearances(
        appearances_path.as_path(),
        config_path.as_ref().map(|p| p.as_path()),
        args.edits,
        output_path.as_ref().map(|p| p.as_path()),
    )
    .map_err(|e| e.to_string())
}
