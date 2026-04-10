use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::{application::bootstrap_snapshot::BootstrapSnapshot, application::state::AppState};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    pub app_name: String,
    pub status: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugWindowEvent {
    pub source: String,
    pub event: String,
    pub data: serde_json::Value,
}

#[tauri::command]
pub fn health_check(app: AppHandle) -> HealthStatus {
    HealthStatus {
        app_name: "Apollo".to_string(),
        status: "Runtime pronto para bootstrap incremental".to_string(),
        version: format!("v{}", app.package_info().version),
    }
}

#[tauri::command]
pub fn bootstrap_summary(state: State<'_, AppState>) -> BootstrapSnapshot {
    state.snapshot().clone()
}

#[tauri::command]
pub fn log_debug_window_event(payload: DebugWindowEvent) {
    tracing::info!(
        source = %payload.source,
        event = %payload.event,
        data = %payload.data,
        "window-debug"
    );
}

#[tauri::command]
pub fn quit_application(app: AppHandle) {
    app.exit(0);
}
