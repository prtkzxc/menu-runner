use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};
use tauri_plugin_shell::ShellExt;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ActionsFile {
    actions: Vec<Action>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Action {
    id: String,
    label: String,
    command: String,
    #[serde(default)]
    platforms: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    working_directory: Option<String>,
}

fn current_platform() -> &'static str {
    if cfg!(target_os = "macos") { "macos" }
    else if cfg!(target_os = "windows") { "windows" }
    else { "linux" }
}

fn config_path(app: &AppHandle) -> PathBuf {
    app.path().app_config_dir().expect("config directory unavailable").join("actions.json")
}

fn read_actions(app: &AppHandle) -> Result<Vec<Action>, String> {
    let path = config_path(app);
    if !path.exists() {
        fs::create_dir_all(path.parent().expect("config parent unavailable")).map_err(|e| e.to_string())?;
        let seed = include_str!("../resources/actions.json");
        fs::write(&path, seed).map_err(|e| e.to_string())?;
    }
    let contents = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let config: ActionsFile = serde_json::from_str(&contents).map_err(|e| format!("Invalid actions.json: {e}"))?;
    Ok(config.actions)
}

fn load_actions(app: &AppHandle) -> Result<Vec<Action>, String> {
    Ok(read_actions(app)?.into_iter().filter(|a| a.platforms.is_empty() || a.platforms.iter().any(|p| p == current_platform())).collect())
}

fn run_action(app: AppHandle, id: String) {
    tauri::async_runtime::spawn(async move {
        let action = load_actions(&app).ok().and_then(|actions| actions.into_iter().find(|a| a.id == id));
        let Some(action) = action else { eprintln!("Action '{id}' was not found"); return; };

        #[cfg(target_os = "windows")]
        let command = app.shell().command("cmd").args(["/C", &action.command]);
        #[cfg(not(target_os = "windows"))]
        let command = app.shell().command("sh").args(["-lc", &action.command]);
        let command = match action.working_directory.filter(|directory| !directory.trim().is_empty()) {
            Some(directory) => command.current_dir(directory),
            None => command,
        };
        let result = command.output().await;

        match result {
            Ok(output) if output.status.success() => println!("{} completed", action.label),
            Ok(output) => eprintln!("{} failed: {}", action.label, String::from_utf8_lossy(&output.stderr)),
            Err(error) => eprintln!("{} could not start: {error}", action.label),
        }
    });
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let items: Vec<MenuItem<tauri::Wry>> = load_actions(app).unwrap_or_else(|error| {
        eprintln!("Could not load actions: {error}");
        vec![]
    }).into_iter().map(|action| MenuItem::with_id(app, format!("action:{}", action.id), action.label, true, None::<&str>)).collect::<Result<Vec<_>, _>>()?;
    let manage = MenuItem::with_id(app, "manage", "Manage actions…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let mut references: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = items.iter().map(|item| item as &dyn tauri::menu::IsMenuItem<tauri::Wry>).collect();
    references.push(&separator);
    references.push(&manage);
    references.push(&quit);
    Menu::with_items(app, &references)
}

fn refresh_tray(app: &AppHandle) -> Result<(), String> {
    let menu = build_menu(app).map_err(|error| error.to_string())?;
    app.tray_by_id("main").ok_or("tray icon unavailable")?.set_menu(Some(menu)).map_err(|error| error.to_string())
}

fn show_manager(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app)?;
    let icon = app.default_window_icon().expect("application icon missing").clone();
    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("Menu Runner")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| {
        let id = event.id().as_ref();
        if id == "quit" { app.exit(0); }
        else if id == "manage" { show_manager(app); }
        else if let Some(action_id) = id.strip_prefix("action:") { run_action(app.clone(), action_id.to_string()); }
        })
        .build(app)?;
    Ok(())
}

#[tauri::command]
fn get_actions(app: AppHandle) -> Result<Vec<Action>, String> { read_actions(&app) }

#[tauri::command]
fn save_actions(app: AppHandle, actions: Vec<Action>) -> Result<(), String> {
    for action in &actions {
        if action.id.trim().is_empty() || action.label.trim().is_empty() || action.command.trim().is_empty() { return Err("Each action needs a name and command.".into()); }
    }
    let contents = serde_json::to_string_pretty(&ActionsFile { actions }).map_err(|error| error.to_string())?;
    let path = config_path(&app);
    fs::create_dir_all(path.parent().expect("config parent unavailable")).map_err(|error| error.to_string())?;
    fs::write(path, format!("{contents}\n")).map_err(|error| error.to_string())?;
    refresh_tray(&app)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| { build_tray(app.handle())?; Ok(()) })
        .invoke_handler(tauri::generate_handler![get_actions, save_actions])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event { api.prevent_close(); let _ = window.hide(); }
        })
        .run(tauri::generate_context!())
        .expect("error while running Menu Runner");
}
