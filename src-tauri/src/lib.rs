pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

use tauri::{Emitter, Manager};

use crate::{
    application::state::AppState,
    domain::app_metadata::AppMetadata,
    infrastructure::{database::bootstrap_database, logging::init_logging},
};

const APP_WINDOW_LABEL: &str = "app";

fn shortcut_surface_target(action: &str) -> Option<&'static str> {
    let action_lower = action.to_lowercase();

    if action_lower.contains("history") || action_lower.contains("histor") {
        Some("history")
    } else if action_lower.contains("settings") || action_lower.contains("config") {
        Some("settings")
    } else {
        None
    }
}

fn reveal_app_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window(APP_WINDOW_LABEL) else {
        return;
    };

    if let Err(error) = window.show() {
        tracing::warn!(error = %error, "failed to show app window from shortcut");
    }

    if let Err(error) = window.unminimize() {
        tracing::warn!(error = %error, "failed to unminimize app window from shortcut");
    }

    if let Err(error) = window.set_focus() {
        tracing::warn!(error = %error, "failed to focus app window from shortcut");
    }
}

/// Route a fired shortcut action to the appropriate backend or frontend behavior.
fn dispatch_shortcut_action(app: &tauri::AppHandle, action: &str) {
    let action_lower = action.to_lowercase();

    if action_lower.contains("capture") || action_lower.contains("captura") {
        // Ask the frontend to open the area-selection overlay so the user can
        // pick exactly the region they want to capture.
        let _ = app.emit("apollo:start-area-capture", ());
    } else {
        if shortcut_surface_target(action).is_some() {
            reveal_app_window(app);
        }

        // Emit the action name so the frontend can handle it (e.g. navigate)
        let _ = app.emit("apollo:shortcut-action", action);
    }
}

#[cfg(test)]
mod tests {
    use super::shortcut_surface_target;

    #[test]
    fn maps_settings_and_history_shortcuts_to_app_surfaces() {
        assert_eq!(shortcut_surface_target("open_settings"), Some("settings"));
        assert_eq!(shortcut_surface_target("open_history"), Some("history"));
        assert_eq!(shortcut_surface_target("abrir_historico"), Some("history"));
    }

    #[test]
    fn ignores_shortcuts_that_do_not_navigate_the_main_window() {
        assert_eq!(shortcut_surface_target("capture_screen"), None);
        assert_eq!(shortcut_surface_target("anything_else"), None);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let accel = shortcut.to_string();
                    let state = app.state::<AppState>();
                    if let Some(action) = state.find_action_for_accelerator(&accel) {
                        dispatch_shortcut_action(app, &action);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let metadata = AppMetadata::new(app.package_info().version.to_string());
            let app_data_dir = app.path().app_data_dir()?;
            let snapshot = bootstrap_database(&app_data_dir, &metadata)?;

            app.manage(AppState::new(snapshot));

            // On Linux/GTK the WebKitGTK widget has a natural preferred height
            // (~200 px) that overrides the config value before JS can run.
            // Enforcing the size here, while the window is still pre-render,
            // prevents the OS window from expanding beyond the tray bar height.
            if let Some(tray_win) = app.get_webview_window("tray") {
                let tray_size = tauri::LogicalSize::new(300.0_f64, 48.0_f64);
                let _ = tray_win.set_min_size(None::<tauri::LogicalSize<f64>>);
                let _ = tray_win.set_max_size(Some(tray_size));
                let _ = tray_win.set_size(tray_size);
                let _ = tray_win.set_min_size(Some(tray_size));
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::analysis::analyze_capture,
            commands::analysis::continue_conversation,
            commands::analysis::extract_text_from_capture,
            commands::capture::trigger_screen_capture,
            commands::capture::capture_screen_region,
            commands::capture::run_ocr_on_image,
            commands::capture::apply_global_shortcuts,
            commands::history::list_history,
            commands::history::load_conversation_messages,
            commands::history::delete_history_session,
            commands::history::clear_history,
            commands::providers::list_provider_models,
            commands::providers::list_provider_models_for,
            commands::settings::load_settings,
            commands::settings::save_settings,
            commands::system::health_check,
            commands::system::bootstrap_summary,
            commands::system::log_debug_window_event,
            commands::system::quit_application
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Apollo desktop application");
}
