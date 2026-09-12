use serde::Deserialize;
use std::{fs, path::PathBuf};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};
use tauri_plugin_shell::ShellExt;

#[derive(Debug, Clone, Deserialize)]
struct ActionsFile {
    actions: Vec<Action>,
}

#[derive(Debug, Clone, Deserialize)]
struct Action {
    id: String,
    label: String,
    command: String,
    #[serde(default)]
    platforms: Vec<String>,
}

fn current_platform() -> &'static str {
    if cfg!(target_os = "macos") { "macos" }
    else if cfg!(target_os = "windows") { "windows" }
    else { "linux" }
}

fn config_path(app: &AppHandle) -> PathBuf {
    app.path().app_config_dir().expect("config directory unavailable").join("actions.json")
}

fn load_actions(app: &AppHandle) -> Result<Vec<Action>, String> {
    let path = config_path(app);
    if !path.exists() {
        fs::create_dir_all(path.parent().expect("config parent unavailable")).map_err(|e| e.to_string())?;
        let seed = include_str!("../resources/actions.json");
        fs::write(&path, seed).map_err(|e| e.to_string())?;
    }
    let contents = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let config: ActionsFile = serde_json::from_str(&contents).map_err(|e| format!("Invalid actions.json: {e}"))?;
    Ok(config.actions.into_iter().filter(|a| a.platforms.is_empty() || a.platforms.iter().any(|p| p == current_platform())).collect())
}

fn run_action(app: AppHandle, id: String) {
    tauri::async_runtime::spawn(async move {
        let action = load_actions(&app).ok().and_then(|actions| actions.into_iter().find(|a| a.id == id));
        let Some(action) = action else { eprintln!("Action '{id}' was not found"); return; };

        #[cfg(target_os = "windows")]
        let result = app.shell().command("cmd").args(["/C", &action.command]).output().await;
        #[cfg(not(target_os = "windows"))]
        let result = app.shell().command("sh").args(["-lc", &action.command]).output().await;

        match result {
            Ok(output) if output.status.success() => println!("{} completed", action.label),
            Ok(output) => eprintln!("{} failed: {}", action.label, String::from_utf8_lossy(&output.stderr)),
            Err(error) => eprintln!("{} could not start: {error}", action.label),
        }
    });
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let items: Vec<MenuItem<tauri::Wry>> = load_actions(app).unwrap_or_else(|error| {
        eprintln!("Could not load actions: {error}");
        vec![]
    }).into_iter().map(|action| MenuItem::with_id(app, format!("action:{}", action.id), action.label, true, None::<&str>)).collect::<Result<Vec<_>, _>>()?;
    let reload = MenuItem::with_id(app, "reload", "Reload actions", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let mut references: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = items.iter().map(|item| item as &dyn tauri::menu::IsMenuItem<tauri::Wry>).collect();
    references.push(&separator);
    references.push(&reload);
    references.push(&quit);
    let menu = Menu::with_items(app, &references)?;
    let icon = app.default_window_icon().expect("application icon missing").clone();
    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("Menu Runner")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| {
        let id = event.id().as_ref();
        if id == "quit" { app.exit(0); }
        else if id == "reload" { eprintln!("Reload actions requires restarting this version of Menu Runner."); }
        else if let Some(action_id) = id.strip_prefix("action:") { run_action(app.clone(), action_id.to_string()); }
        })
        .build(app)?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| { build_tray(app.handle())?; Ok(()) })
        .run(tauri::generate_context!())
        .expect("error while running Menu Runner");
}
