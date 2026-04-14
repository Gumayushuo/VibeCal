use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as AutostartManagerExt};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_window_state::{AppHandleExt as WindowStateAppHandleExt, StateFlags, WindowExt as WindowStateWindowExt};

const APP_TITLE: &str = "VibeCal";
const LEGACY_APP_ID: &str = "com.local.applecalendardesktop";
const ICLOUD_GLOBAL_URL: &str = "https://www.icloud.com/calendar/";
const ICLOUD_CHINA_URL: &str = "https://www.icloud.com.cn/calendar/";

type SharedState = Arc<AppState>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AppPreferences {
    #[serde(default)]
    desktop_mode: bool,
}

#[derive(Debug)]
struct AppState {
    quitting: AtomicBool,
    desktop_mode: AtomicBool,
}

impl AppState {
    fn new(preferences: AppPreferences) -> Self {
        Self {
            quitting: AtomicBool::new(false),
            desktop_mode: AtomicBool::new(preferences.desktop_mode),
        }
    }
}

fn app_state(app: &AppHandle) -> SharedState {
    app.state::<SharedState>().inner().clone()
}

fn is_desktop_mode(app: &AppHandle) -> bool {
    app_state(app).desktop_mode.load(Ordering::Relaxed)
}

fn app_data_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_local_data_dir().ok()
}

fn legacy_app_data_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path().local_data_dir().ok().map(|dir| dir.join(LEGACY_APP_ID))
}

fn preferences_path(app: &AppHandle) -> Option<PathBuf> {
    app_data_dir(app).map(|dir| dir.join("settings.json"))
}

fn cookies_path(app: &AppHandle) -> Option<PathBuf> {
    app_data_dir(app).map(|dir| {
        dir.join("webview")
            .join("EBWebView")
            .join("Default")
            .join("Network")
            .join("Cookies")
    })
}

fn load_preferences(app: &AppHandle) -> AppPreferences {
    preferences_path(app)
        .and_then(|path| fs::read(path).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn persist_preferences(app: &AppHandle) {
    let preferences = AppPreferences {
        desktop_mode: is_desktop_mode(app),
    };

    let Some(path) = preferences_path(app) else {
        return;
    };

    let Some(parent) = path.parent() else {
        return;
    };

    if let Err(error) = fs::create_dir_all(parent) {
        eprintln!("failed to create preferences directory: {error}");
        return;
    }

    match serde_json::to_vec_pretty(&preferences) {
        Ok(bytes) => {
            if let Err(error) = fs::write(path, bytes) {
                eprintln!("failed to persist preferences: {error}");
            }
        }
        Err(error) => eprintln!("failed to serialize preferences: {error}"),
    }
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            fs::copy(source_path, destination_path)?;
        }
    }

    Ok(())
}

fn migrate_legacy_local_state(app: &AppHandle) {
    let Some(current_dir) = app_data_dir(app) else {
        return;
    };

    if current_dir.exists() {
        return;
    }

    let Some(legacy_dir) = legacy_app_data_dir(app) else {
        return;
    };

    if !legacy_dir.exists() {
        return;
    }

    if let Some(parent) = current_dir.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("failed to prepare current app data directory: {error}");
            return;
        }
    }

    if let Err(error) = copy_dir_recursive(&legacy_dir, &current_dir) {
        eprintln!("failed to migrate legacy local state: {error}");
    }
}

fn detect_calendar_url(app: &AppHandle) -> &'static str {
    let Some(path) = cookies_path(app) else {
        return ICLOUD_GLOBAL_URL;
    };

    let Ok(bytes) = fs::read(path) else {
        return ICLOUD_GLOBAL_URL;
    };

    if contains_bytes(&bytes, b".icloud.com.cn") {
        return ICLOUD_CHINA_URL;
    }

    if contains_bytes(&bytes, b".icloud.com") {
        return ICLOUD_GLOBAL_URL;
    }

    ICLOUD_GLOBAL_URL
}

fn webview_data_dir(app: &AppHandle) -> tauri::Result<std::path::PathBuf> {
    Ok(app.path().app_local_data_dir()?.join("webview"))
}

fn create_main_window(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let state = app_state(app);
    let desktop_mode = state.desktop_mode.load(Ordering::Relaxed);
    let app_url = detect_calendar_url(app);
    let window = WebviewWindowBuilder::new(
        app,
        "main",
        WebviewUrl::External(app_url.parse().expect("app url should be valid")),
    )
    .title(APP_TITLE)
    .inner_size(1366.0, 900.0)
    .min_inner_size(1024.0, 700.0)
    .resizable(!desktop_mode)
    .decorations(!desktop_mode)
    .skip_taskbar(desktop_mode)
    .always_on_bottom(desktop_mode)
    .visible(false)
    .data_directory(webview_data_dir(app)?)
    .build()?;

    let managed_window = window.clone();
    let managed_app = app.clone();
    let managed_state = state.clone();
    window.on_window_event(move |event| {
        match event {
            WindowEvent::CloseRequested { api, .. } => {
                if managed_state.quitting.load(Ordering::Relaxed) {
                    let _ = managed_app.save_window_state(StateFlags::all());
                    return;
                }

                api.prevent_close();
                let _ = managed_app.save_window_state(StateFlags::all());
                let _ = managed_window.hide();
            }
            WindowEvent::Focused(false) if managed_state.desktop_mode.load(Ordering::Relaxed) => {
                let _ = managed_window.set_always_on_bottom(true);
            }
            _ => {}
        }
    });

    Ok(window)
}

fn ensure_main_window(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    if let Some(existing) = app.get_webview_window("main") {
        return Ok(existing);
    }

    create_main_window(app)
}

fn show_main_window(app: &AppHandle) -> tauri::Result<()> {
    let window = ensure_main_window(app)?;

    if window.is_minimized()? {
        window.unminimize()?;
    }

    if !window.is_visible()? {
        window.show()?;
    }

    if !is_desktop_mode(app) {
        window.set_focus()?;
    }

    Ok(())
}

fn hide_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = app.save_window_state(StateFlags::all());
        window.hide()?;
    }

    Ok(())
}

fn apply_desktop_mode(app: &AppHandle, enabled: bool) -> tauri::Result<()> {
    let state = app_state(app);
    state.desktop_mode.store(enabled, Ordering::Relaxed);
    persist_preferences(app);

    let window = ensure_main_window(app)?;

    if window.is_maximized()? {
        let _ = window.unmaximize();
    }

    let _ = app.save_window_state(StateFlags::all());
    window.set_decorations(!enabled)?;
    window.set_resizable(!enabled)?;
    window.set_skip_taskbar(enabled)?;
    window.set_always_on_bottom(enabled)?;

    if enabled {
        window.show()?;
    } else {
        window.show()?;
        window.set_focus()?;
    }

    Ok(())
}

fn show_shell_notification(app: &AppHandle, title: &str, body: &str) {
    if let Err(error) = app
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show()
    {
        eprintln!("failed to show notification: {error}");
    }
}

fn quit_app(app: &AppHandle) {
    let state = app_state(app);
    state.quitting.store(true, Ordering::Relaxed);

    let _ = app.save_window_state(StateFlags::all());

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.close();
    }

    let app_handle = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(400));
        app_handle.exit(0);
    });
}

fn ensure_autostart(app: &AppHandle) {
    let autostart_manager = app.autolaunch();

    match autostart_manager.is_enabled() {
        Ok(true) => {}
        Ok(false) => {
            if let Err(error) = autostart_manager.enable() {
                eprintln!("failed to enable autostart: {error}");
            }
        }
        Err(error) => eprintln!("failed to read autostart state: {error}"),
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open_item = MenuItem::with_id(app, "open", "Open Calendar", true, None::<&str>)?;
    let desktop_mode_item = CheckMenuItem::with_id(
        app,
        "desktop_mode",
        "Pin to Desktop Layer",
        true,
        is_desktop_mode(app),
        None::<&str>,
    )?;
    let hide_item = MenuItem::with_id(app, "hide", "Hide to Tray", true, None::<&str>)?;
    let browser_item = MenuItem::with_id(app, "browser", "Open in Browser", true, None::<&str>)?;
    let notify_item = MenuItem::with_id(app, "notify", "Test Notification", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &open_item,
            &desktop_mode_item,
            &hide_item,
            &browser_item,
            &notify_item,
            &separator,
            &quit_item,
        ],
    )?;

    let mut tray = TrayIconBuilder::new()
        .tooltip(APP_TITLE)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                let _ = show_main_window(app);
            }
            "desktop_mode" => {
                let _ = apply_desktop_mode(app, !is_desktop_mode(app));
            }
            "hide" => {
                let _ = hide_main_window(app);
            }
            "browser" => {
                let _ = app.opener().open_url(detect_calendar_url(app), None::<&str>);
            }
            "notify" => {
                show_shell_notification(
                    app,
                    APP_TITLE,
                    "Native notification plumbing is active.",
                );
            }
            "quit" => {
                quit_app(app);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }

    tray.build(app)?;

    Ok(())
}

fn initialize_main_window(app: &AppHandle) -> tauri::Result<()> {
    let launch_minimized = std::env::args().any(|arg| arg == "--minimized");
    let window = ensure_main_window(app)?;
    let _ = window.restore_state(StateFlags::all());
    let _ = apply_desktop_mode(app, is_desktop_mode(app));

    if !launch_minimized {
        show_main_window(app)?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = show_main_window(app);
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            migrate_legacy_local_state(&app_handle);
            let state = Arc::new(AppState::new(load_preferences(&app_handle)));

            app.manage::<SharedState>(state);

            app_handle.plugin(tauri_plugin_autostart::init(
                MacosLauncher::LaunchAgent,
                Some(vec!["--minimized"]),
            ))?;

            build_tray(&app_handle)?;
            initialize_main_window(&app_handle)?;
            ensure_autostart(&app_handle);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
