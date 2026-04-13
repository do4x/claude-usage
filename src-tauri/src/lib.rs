mod api;
mod config;

use api::UsageData;
use config::AppConfig;
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, State,
};

struct AppState {
    config: Mutex<AppConfig>,
    current_usage: Mutex<Option<UsageData>>,
}

#[tauri::command]
async fn save_config(
    config: AppConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut app_config = state.config.lock().unwrap();
    *app_config = config;
    Ok(())
}

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().unwrap();
    Ok(config.clone())
}

#[tauri::command]
async fn get_usage(state: State<'_, AppState>) -> Result<Option<UsageData>, String> {
    let usage = state.current_usage.lock().unwrap();
    Ok(usage.clone())
}

#[tauri::command]
async fn refresh_usage(state: State<'_, AppState>) -> Result<UsageData, String> {
    let config = state.config.lock().unwrap();
    if config.session_cookie.is_empty() || config.org_id.is_empty() {
        return Err("Please configure cookie and organization ID".to_string());
    }

    match api::fetch_usage(&config.session_cookie, &config.org_id).await {
        Ok(usage) => {
            let mut current = state.current_usage.lock().unwrap();
            *current = Some(usage.clone());
            Ok(usage)
        }
        Err(e) => Err(e.to_string()),
    }
}

fn get_icon_path(percent: f32) -> String {
    if percent <= 50.0 {
        "green".to_string()
    } else if percent <= 80.0 {
        "yellow".to_string()
    } else {
        "red".to_string()
    }
}

fn format_tooltip(usage: &UsageData) -> String {
    let session_hours = usage.session_reset_minutes / 60;
    let session_mins = usage.session_reset_minutes % 60;
    let weekly_hours = usage.weekly_reset_minutes / 60;
    let weekly_mins = usage.weekly_reset_minutes % 60;

    format!(
        "Claude {} Usage\nSession: {:.0}% used (resets in {}h {}m)\nWeekly: {:.0}% used (resets in {}h {}m)",
        usage.plan_type, usage.session_percent, session_hours, session_mins,
        usage.weekly_percent, weekly_hours, weekly_mins
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState {
        config: Mutex::new(AppConfig::default()),
        current_usage: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            save_config,
            get_config,
            get_usage,
            refresh_usage
        ])
        .setup(|app| {
            let quit_menu = MenuItem::Quit;
            let settings_menu = MenuItem::new("Settings", "settings");
            let refresh_menu = MenuItem::new("Refresh Now", "refresh");

            let menu = Menu::with_items(vec![
                refresh_menu.into(),
                settings_menu.into(),
                quit_menu.into(),
            ]);

            let tray = TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Claude Usage")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "settings" => {
                            if let Some(window) = app.get_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "refresh" => {
                            let app_handle = app.app_handle().clone();
                            tauri::async_runtime::spawn(async move {
                                let state = app_handle.state::<AppState>();
                                let _ = refresh_usage(state).await;
                            });
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Start polling loop
            let app_handle = app.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    let state = app_handle.state::<AppState>();
                    let interval = {
                        let config = state.config.lock().unwrap();
                        config.poll_interval_secs
                    };

                    if let Ok(usage) = refresh_usage(state.clone()).await {
                        if let Ok(mut tray) = tray.lock() {
                            let _ = tray.set_tooltip(Some(format_tooltip(&usage)));
                        }
                    }

                    tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
