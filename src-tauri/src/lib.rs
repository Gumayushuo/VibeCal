use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as AutostartManagerExt};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::{Update, UpdaterExt};
use tauri_plugin_window_state::{
    AppHandleExt as WindowStateAppHandleExt, StateFlags, WindowExt as WindowStateWindowExt,
};
mod desktop_controls;
use desktop_controls::{apply_appearance, open_controls};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_BOTTOM, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOOWNERZORDER, SWP_NOSIZE,
};

const APP_TITLE: &str = "VibeCal";
const LEGACY_APP_ID: &str = "com.local.applecalendardesktop";

const CALENDAR_WINDOW_LABEL: &str = "calendar";
const REMINDERS_WINDOW_LABEL: &str = "reminders";
const NOTES_WINDOW_LABEL: &str = "notes";
const UPDATE_ENDPOINT: &str =
    "https://github.com/Gumayushuo/VibeCal/releases/latest/download/latest.json";

const ICLOUD_GLOBAL_CALENDAR_URL: &str = "https://www.icloud.com/calendar/";
const ICLOUD_CHINA_CALENDAR_URL: &str = "https://www.icloud.com.cn/calendar/";
const ICLOUD_GLOBAL_REMINDERS_URL: &str = "https://www.icloud.com/reminders/";
const ICLOUD_CHINA_REMINDERS_URL: &str = "https://www.icloud.com.cn/reminders/";
const ICLOUD_GLOBAL_NOTES_URL: &str = "https://www.icloud.com/notes/";
const ICLOUD_CHINA_NOTES_URL: &str = "https://www.icloud.com.cn/notes/";

type SharedState = Arc<AppState>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CloudPage {
    Calendar,
    Reminders,
    Notes,
}

#[derive(Debug, Clone, Copy)]
struct WindowGeometry {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct WindowPreferences {
    #[serde(default)]
    desktop_mode: bool,
    #[serde(default)]
    always_on_top: bool,
    #[serde(default)]
    visible: bool,
    #[serde(default = "default_zoom")]
    zoom: f64,
    #[serde(default = "default_opacity")]
    opacity: u8,
    #[serde(default)]
    auto_fit: bool,
}

fn default_zoom() -> f64 {
    1.0
}
fn default_opacity() -> u8 {
    100
}

impl Default for WindowPreferences {
    fn default() -> Self {
        Self {
            desktop_mode: false,
            always_on_top: false,
            visible: false,
            zoom: default_zoom(),
            opacity: default_opacity(),
            auto_fit: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppPreferences {
    #[serde(default)]
    calendar: WindowPreferences,
    #[serde(default)]
    reminders: WindowPreferences,
    #[serde(default)]
    notes: WindowPreferences,
    #[serde(default)]
    desktop_mode: bool,
    #[serde(default)]
    always_on_top: bool,
}

impl Default for AppPreferences {
    fn default() -> Self {
        let visible = WindowPreferences {
            visible: true,
            ..Default::default()
        };
        Self {
            calendar: visible,
            reminders: visible,
            notes: visible,
            desktop_mode: false,
            always_on_top: false,
        }
    }
}

impl AppPreferences {
    fn normalized(mut self) -> Self {
        if self.desktop_mode || self.always_on_top {
            for page_preferences in [&mut self.calendar, &mut self.reminders, &mut self.notes] {
                if !page_preferences.desktop_mode && !page_preferences.always_on_top {
                    page_preferences.desktop_mode = self.desktop_mode;
                    page_preferences.always_on_top = self.always_on_top && !self.desktop_mode;
                }
            }
        }

        for page_preferences in [&mut self.calendar, &mut self.reminders, &mut self.notes] {
            page_preferences.zoom = if page_preferences.zoom.is_finite() {
                page_preferences.zoom.clamp(0.25, 1.5)
            } else {
                1.0
            };
            page_preferences.opacity = page_preferences.opacity.clamp(30, 100);
            if page_preferences.desktop_mode {
                page_preferences.always_on_top = false;
            }
        }

        self.desktop_mode = false;
        self.always_on_top = false;
        self
    }

    fn page(self, page: CloudPage) -> WindowPreferences {
        match page {
            CloudPage::Calendar => self.calendar,
            CloudPage::Reminders => self.reminders,
            CloudPage::Notes => self.notes,
        }
    }

    fn page_mut(&mut self, page: CloudPage) -> &mut WindowPreferences {
        match page {
            CloudPage::Calendar => &mut self.calendar,
            CloudPage::Reminders => &mut self.reminders,
            CloudPage::Notes => &mut self.notes,
        }
    }
}

#[derive(Debug)]
struct AppState {
    quitting: AtomicBool,
    checking_updates: AtomicBool,
    preferences: Mutex<AppPreferences>,
}

impl AppState {
    fn new(preferences: AppPreferences) -> Self {
        Self {
            quitting: AtomicBool::new(false),
            checking_updates: AtomicBool::new(false),
            preferences: Mutex::new(preferences.normalized()),
        }
    }
}

#[derive(Clone)]
struct WindowMenuControls {
    calendar_normal: CheckMenuItem<tauri::Wry>,
    calendar_desktop: CheckMenuItem<tauri::Wry>,
    calendar_top: CheckMenuItem<tauri::Wry>,
    reminders_normal: CheckMenuItem<tauri::Wry>,
    reminders_desktop: CheckMenuItem<tauri::Wry>,
    reminders_top: CheckMenuItem<tauri::Wry>,
    notes_normal: CheckMenuItem<tauri::Wry>,
    notes_desktop: CheckMenuItem<tauri::Wry>,
    notes_top: CheckMenuItem<tauri::Wry>,
}

fn app_state(app: &AppHandle) -> SharedState {
    app.state::<SharedState>().inner().clone()
}

fn snapshot_preferences(app: &AppHandle) -> AppPreferences {
    app_state(app)
        .preferences
        .lock()
        .expect("preferences mutex poisoned")
        .clone()
        .normalized()
}

fn page_preferences(app: &AppHandle, page: CloudPage) -> WindowPreferences {
    snapshot_preferences(app).page(page)
}

fn is_normal_window(preferences: WindowPreferences) -> bool {
    !preferences.desktop_mode && !preferences.always_on_top
}

fn visible_pages(app: &AppHandle) -> Vec<CloudPage> {
    pages()
        .into_iter()
        .filter(|page| page_preferences(app, *page).visible)
        .collect()
}

fn update_page_preferences(
    app: &AppHandle,
    page: CloudPage,
    mutate: impl FnOnce(&mut WindowPreferences),
) -> WindowPreferences {
    let state = app_state(app);
    let mut preferences = state
        .preferences
        .lock()
        .expect("preferences mutex poisoned");
    mutate(preferences.page_mut(page));
    let normalized = preferences.clone().normalized();
    *preferences = normalized.clone();
    drop(preferences);
    persist_preferences(app);
    if let Some(controls) = app.try_state::<WindowMenuControls>() {
        let _ = sync_window_menu_controls(&controls, app);
    }
    normalized.page(page)
}

fn page_label(page: CloudPage) -> &'static str {
    match page {
        CloudPage::Calendar => CALENDAR_WINDOW_LABEL,
        CloudPage::Reminders => REMINDERS_WINDOW_LABEL,
        CloudPage::Notes => NOTES_WINDOW_LABEL,
    }
}

fn page_title(page: CloudPage) -> &'static str {
    match page {
        CloudPage::Calendar => "VibeCal 日历",
        CloudPage::Reminders => "VibeCal 提醒事项",
        CloudPage::Notes => "VibeCal 备忘录",
    }
}

fn page_name(page: CloudPage) -> &'static str {
    match page {
        CloudPage::Calendar => "日历",
        CloudPage::Reminders => "提醒事项",
        CloudPage::Notes => "备忘录",
    }
}

fn page_default_geometry(page: CloudPage) -> WindowGeometry {
    match page {
        CloudPage::Calendar => WindowGeometry {
            x: 72.0,
            y: 64.0,
            width: 1180.0,
            height: 900.0,
        },
        CloudPage::Reminders => WindowGeometry {
            x: 1284.0,
            y: 64.0,
            width: 420.0,
            height: 430.0,
        },
        CloudPage::Notes => WindowGeometry {
            x: 1284.0,
            y: 526.0,
            width: 420.0,
            height: 438.0,
        },
    }
}

fn pages() -> [CloudPage; 3] {
    [CloudPage::Calendar, CloudPage::Reminders, CloudPage::Notes]
}

fn app_data_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_local_data_dir().ok()
}

fn legacy_app_data_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .local_data_dir()
        .ok()
        .map(|dir| dir.join(LEGACY_APP_ID))
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
    let Some(path) = preferences_path(app) else {
        return AppPreferences::default();
    };

    let Ok(bytes) = fs::read(path) else {
        return AppPreferences::default();
    };

    let mut preferences = serde_json::from_slice::<AppPreferences>(&bytes).unwrap_or_default();

    if !contains_bytes(&bytes, b"\"visible\"")
        && !preferences.calendar.visible
        && !preferences.reminders.visible
        && !preferences.notes.visible
    {
        preferences.calendar.visible = true;
        preferences.reminders.visible = true;
        preferences.notes.visible = true;
    }

    if !preferences.calendar.visible && !preferences.reminders.visible && !preferences.notes.visible
    {
        preferences.calendar.visible = true;
        preferences.reminders.visible = true;
        preferences.notes.visible = true;
    }

    preferences.normalized()
}

fn persist_preferences(app: &AppHandle) {
    let preferences = snapshot_preferences(app);

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
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
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

fn prefers_china_domain(app: &AppHandle) -> bool {
    let Some(path) = cookies_path(app) else {
        return true;
    };

    let Ok(bytes) = fs::read(path) else {
        return true;
    };

    if contains_bytes(&bytes, b".icloud.com.cn") {
        return true;
    }

    if contains_bytes(&bytes, b".icloud.com") {
        return false;
    }

    true
}

fn detect_page_url(app: &AppHandle, page: CloudPage) -> &'static str {
    let use_china_domain = prefers_china_domain(app);

    match (page, use_china_domain) {
        (CloudPage::Calendar, true) => ICLOUD_CHINA_CALENDAR_URL,
        (CloudPage::Calendar, false) => ICLOUD_GLOBAL_CALENDAR_URL,
        (CloudPage::Reminders, true) => ICLOUD_CHINA_REMINDERS_URL,
        (CloudPage::Reminders, false) => ICLOUD_GLOBAL_REMINDERS_URL,
        (CloudPage::Notes, true) => ICLOUD_CHINA_NOTES_URL,
        (CloudPage::Notes, false) => ICLOUD_GLOBAL_NOTES_URL,
    }
}

fn webview_data_dir(app: &AppHandle) -> tauri::Result<PathBuf> {
    Ok(app.path().app_local_data_dir()?.join("webview"))
}

#[cfg(target_os = "windows")]
fn apply_native_window_z_order(window: &WebviewWindow, preferences: WindowPreferences) {
    let Ok(hwnd) = window.hwnd() else {
        return;
    };

    let insert_after = if preferences.desktop_mode {
        HWND_BOTTOM
    } else if preferences.always_on_top {
        HWND_TOPMOST
    } else {
        HWND_NOTOPMOST
    };

    let flags = SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOOWNERZORDER;

    unsafe {
        if let Err(error) = SetWindowPos(hwnd, Some(insert_after), 0, 0, 0, 0, flags) {
            eprintln!(
                "failed to apply native z-order for {}: {error}",
                window.label()
            );
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn apply_native_window_z_order(_window: &WebviewWindow, _preferences: WindowPreferences) {}

fn apply_window_modes(window: &WebviewWindow, preferences: WindowPreferences) -> tauri::Result<()> {
    window.set_decorations(!preferences.desktop_mode)?;
    window.set_resizable(!preferences.desktop_mode)?;
    window.set_skip_taskbar(preferences.desktop_mode)?;
    if preferences.desktop_mode {
        window.set_always_on_top(false)?;
        window.set_always_on_bottom(true)?;
    } else {
        window.set_always_on_bottom(false)?;
        window.set_always_on_top(preferences.always_on_top)?;
    }

    apply_native_window_z_order(window, preferences);
    apply_appearance(window, preferences)?;
    Ok(())
}

fn refresh_window_modes(app: &AppHandle, page: CloudPage) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(page_label(page)) {
        apply_window_modes(&window, page_preferences(app, page))?;
    }

    Ok(())
}

fn create_window(app: &AppHandle, page: CloudPage) -> tauri::Result<WebviewWindow> {
    let geometry = page_default_geometry(page);
    let preferences = page_preferences(app, page);
    let popup_app = app.clone();
    let navigation_app = app.clone();
    let window = WebviewWindowBuilder::new(
        app,
        page_label(page),
        WebviewUrl::External(
            detect_page_url(app, page)
                .parse()
                .expect("page url should be valid"),
        ),
    )
    .title(page_title(page))
    .inner_size(geometry.width, geometry.height)
    .min_inner_size(360.0, 280.0)
    .position(geometry.x, geometry.y)
    .resizable(!preferences.desktop_mode)
    .decorations(!preferences.desktop_mode)
    .skip_taskbar(preferences.desktop_mode)
    .always_on_bottom(preferences.desktop_mode)
    .always_on_top(preferences.always_on_top && !preferences.desktop_mode)
    .visible(false)
    .data_directory(webview_data_dir(app)?)
    .initialization_script_for_all_frames(include_str!("page-export.js"))
    .on_new_window(move |url, _features| {
        if !desktop_controls::is_external_link(&url) && matches!(url.scheme(), "http" | "https") {
            return tauri::webview::NewWindowResponse::Allow;
        }
        desktop_controls::open_external_link(&popup_app, &url);
        tauri::webview::NewWindowResponse::Deny
    })
    .on_navigation(move |url| {
        if desktop_controls::is_external_link(url) {
            desktop_controls::open_external_link(&navigation_app, url);
            false
        } else {
            true
        }
    })
    .on_page_load(move |window, _payload| {
        let _ = apply_appearance(&window, page_preferences(window.app_handle(), page));
    })
    .build()?;

    let managed_window = window.clone();
    let managed_app = app.clone();
    let managed_state = app_state(app);
    window.on_window_event(move |event| match event {
        WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
            let _ = apply_appearance(&managed_window, page_preferences(&managed_app, page));
        }
        WindowEvent::CloseRequested { api, .. } => {
            if managed_state.quitting.load(Ordering::Relaxed) {
                let _ = managed_app.save_window_state(StateFlags::all());
                return;
            }

            api.prevent_close();
            let _ = managed_app.save_window_state(StateFlags::all());
            let _ = update_page_preferences(&managed_app, page, |page_preferences| {
                page_preferences.visible = false;
            });
            let _ = managed_window.hide();
        }
        WindowEvent::Focused(false) if page_preferences(&managed_app, page).desktop_mode => {
            let _ = apply_window_modes(&managed_window, page_preferences(&managed_app, page));
        }
        WindowEvent::Focused(true) => {
            let _ = apply_window_modes(&managed_window, page_preferences(&managed_app, page));
        }
        _ => {}
    });

    Ok(window)
}

fn ensure_window(app: &AppHandle, page: CloudPage) -> tauri::Result<WebviewWindow> {
    if let Some(existing) = app.get_webview_window(page_label(page)) {
        return Ok(existing);
    }

    create_window(app, page)
}

fn restore_and_prepare_window(app: &AppHandle, page: CloudPage) -> tauri::Result<WebviewWindow> {
    let window = ensure_window(app, page)?;
    let _ = window.restore_state(StateFlags::all());
    apply_window_modes(&window, page_preferences(app, page))?;
    Ok(window)
}

fn show_window(
    app: &AppHandle,
    page: CloudPage,
    focus: bool,
    remember_visible: bool,
) -> tauri::Result<()> {
    if remember_visible {
        let _ = update_page_preferences(app, page, |page_preferences| {
            page_preferences.visible = true;
        });
    }

    let window = restore_and_prepare_window(app, page)?;
    let preferences = page_preferences(app, page);

    apply_window_modes(&window, preferences)?;

    if window.is_minimized()? {
        window.unminimize()?;
    }

    if !window.is_visible()? {
        window.show()?;
    }

    if focus && !preferences.desktop_mode {
        window.set_focus()?;
    }

    Ok(())
}

fn hide_window(app: &AppHandle, page: CloudPage, remember_hidden: bool) -> tauri::Result<()> {
    if remember_hidden {
        let _ = update_page_preferences(app, page, |page_preferences| {
            page_preferences.visible = false;
        });
    }

    if let Some(window) = app.get_webview_window(page_label(page)) {
        let _ = app.save_window_state(StateFlags::all());
        window.hide()?;
    }

    Ok(())
}

fn show_workspace(app: &AppHandle, focus_preferred: bool) -> tauri::Result<()> {
    let pages_to_show = visible_pages(app);
    let focus_target = if focus_preferred {
        if pages_to_show.contains(&CloudPage::Calendar) {
            Some(CloudPage::Calendar)
        } else {
            pages_to_show.first().copied()
        }
    } else {
        None
    };

    for page in pages_to_show {
        show_window(app, page, focus_target == Some(page), false)?;
    }

    Ok(())
}

fn hide_workspace(app: &AppHandle) -> tauri::Result<()> {
    for page in pages() {
        let _ = hide_window(app, page, false);
    }

    Ok(())
}

fn toggle_desktop_mode(app: &AppHandle, page: CloudPage) -> tauri::Result<WindowPreferences> {
    let preferences = if page_preferences(app, page).desktop_mode {
        set_normal_mode(app, page)?
    } else {
        update_page_preferences(app, page, |page_preferences| {
            page_preferences.desktop_mode = true;
            page_preferences.always_on_top = false;
        })
    };
    refresh_window_modes(app, page)?;
    Ok(preferences)
}

fn toggle_always_on_top(app: &AppHandle, page: CloudPage) -> tauri::Result<WindowPreferences> {
    let preferences = if page_preferences(app, page).always_on_top {
        set_normal_mode(app, page)?
    } else {
        update_page_preferences(app, page, |page_preferences| {
            page_preferences.always_on_top = true;
            page_preferences.desktop_mode = false;
        })
    };
    refresh_window_modes(app, page)?;
    Ok(preferences)
}

fn set_normal_mode(app: &AppHandle, page: CloudPage) -> tauri::Result<WindowPreferences> {
    let preferences = update_page_preferences(app, page, |page_preferences| {
        page_preferences.desktop_mode = false;
        page_preferences.always_on_top = false;
    });
    refresh_window_modes(app, page)?;
    Ok(preferences)
}

fn show_shell_notification(app: &AppHandle, title: &str, body: &str) {
    if let Err(error) = app.notification().builder().title(title).body(body).show() {
        eprintln!("failed to show notification: {error}");
    }
}

fn update_notes(update: &Update) -> String {
    update
        .body
        .as_deref()
        .map(str::trim)
        .filter(|notes| !notes.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "此版本没有附带更新说明。".to_string())
}

fn update_dialog_message(update: &Update) -> String {
    format!(
        "发现新版本 v{}（当前版本 v{}）。\n\n更新内容：\n{}\n\n是否现在下载安装？",
        update.version,
        update.current_version,
        update_notes(update)
    )
}

fn begin_update_check(app: &AppHandle) -> bool {
    app_state(app)
        .checking_updates
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_ok()
}

fn finish_update_check(app: &AppHandle) {
    app_state(app)
        .checking_updates
        .store(false, Ordering::Release);
}

async fn install_update(app: AppHandle, update: Update) {
    show_shell_notification(
        &app,
        "VibeCal 更新",
        "正在下载并安装更新。Windows 会在安装前自动退出应用。",
    );

    if let Err(error) = update.download_and_install(|_, _| {}, || {}).await {
        eprintln!("failed to install update: {error}");
        app.dialog()
            .message(format!("自动更新失败：{error}"))
            .title("更新失败")
            .kind(MessageDialogKind::Error)
            .buttons(MessageDialogButtons::Ok)
            .show(|_| {});
        #[cfg(not(target_os = "windows"))]
        return;
    }

    #[cfg(not(target_os = "windows"))]
    app.restart();
}

fn prompt_for_update(app: &AppHandle, update: Update) {
    let app_handle = app.clone();
    app.dialog()
        .message(update_dialog_message(&update))
        .title("发现新版本")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "立即更新".to_string(),
            "稍后".to_string(),
        ))
        .show(move |should_install| {
            if should_install {
                let install_app = app_handle.clone();
                let update_to_install = update.clone();
                tauri::async_runtime::spawn(async move {
                    install_update(install_app, update_to_install).await;
                });
            }
        });
}

async fn check_for_updates(app: AppHandle, interactive: bool) {
    if !begin_update_check(&app) {
        if interactive {
            show_shell_notification(&app, "VibeCal 更新", "已经在检查更新，请稍候。");
        }
        return;
    }

    let before_exit_app = app.clone();
    let updater_result = app
        .updater_builder()
        .endpoints(vec![UPDATE_ENDPOINT
            .parse()
            .expect("update endpoint should be valid")])
        .expect("update endpoints should be valid")
        .timeout(Duration::from_secs(20))
        .on_before_exit(move || {
            let state = app_state(&before_exit_app);
            state.quitting.store(true, Ordering::Relaxed);
            let _ = before_exit_app.save_window_state(StateFlags::all());
        })
        .build();

    let update_result = match updater_result {
        Ok(updater) => updater.check().await,
        Err(error) => Err(error),
    };

    finish_update_check(&app);

    match update_result {
        Ok(Some(update)) => prompt_for_update(&app, update),
        Ok(None) if interactive => {
            app.dialog()
                .message("当前已经是最新版本。")
                .title("检查更新")
                .kind(MessageDialogKind::Info)
                .buttons(MessageDialogButtons::Ok)
                .show(|_| {});
        }
        Ok(None) => {}
        Err(error) => {
            eprintln!("failed to check for updates: {error}");
            if interactive {
                app.dialog()
                    .message(format!("检查更新失败：{error}"))
                    .title("检查更新失败")
                    .kind(MessageDialogKind::Error)
                    .buttons(MessageDialogButtons::Ok)
                    .show(|_| {});
            }
        }
    }
}

fn browser_url_for(page: CloudPage, app: &AppHandle) -> &'static str {
    detect_page_url(app, page)
}

fn print_page(app: &AppHandle, page: CloudPage) -> tauri::Result<()> {
    show_window(app, page, true, true)?;

    let Some(window) = app.get_webview_window(page_label(page)) else {
        let body = format!("找不到可打印的{}窗口。", page_name(page));
        show_shell_notification(app, APP_TITLE, &body);
        return Ok(());
    };

    let print_script = r#"
if (document.readyState === "loading") {
  window.addEventListener("load", () => window.print(), { once: true });
} else {
  window.print();
}
"#;

    if let Err(error) = window.eval(print_script) {
        eprintln!("failed to trigger print for {}: {error}", page_label(page));
        let body = format!("无法打开{}的打印对话框。", page_name(page));
        show_shell_notification(app, APP_TITLE, &body);
        return Err(error);
    }

    Ok(())
}

fn quit_app(app: &AppHandle) {
    let state = app_state(app);
    state.quitting.store(true, Ordering::Relaxed);

    let _ = app.save_window_state(StateFlags::all());

    for page in pages() {
        if let Some(window) = app.get_webview_window(page_label(page)) {
            let _ = window.close();
        }
    }

    let app_handle = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(350));
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

fn sync_window_menu_controls(controls: &WindowMenuControls, app: &AppHandle) -> tauri::Result<()> {
    let calendar = page_preferences(app, CloudPage::Calendar);
    let reminders = page_preferences(app, CloudPage::Reminders);
    let notes = page_preferences(app, CloudPage::Notes);

    controls
        .calendar_normal
        .set_checked(is_normal_window(calendar))?;
    controls
        .calendar_desktop
        .set_checked(calendar.desktop_mode)?;
    controls.calendar_top.set_checked(calendar.always_on_top)?;
    controls
        .reminders_normal
        .set_checked(is_normal_window(reminders))?;
    controls
        .reminders_desktop
        .set_checked(reminders.desktop_mode)?;
    controls
        .reminders_top
        .set_checked(reminders.always_on_top)?;
    controls.notes_normal.set_checked(is_normal_window(notes))?;
    controls.notes_desktop.set_checked(notes.desktop_mode)?;
    controls.notes_top.set_checked(notes.always_on_top)?;

    Ok(())
}

type WindowSubmenuBundle = (
    Submenu<tauri::Wry>,
    CheckMenuItem<tauri::Wry>,
    CheckMenuItem<tauri::Wry>,
    CheckMenuItem<tauri::Wry>,
);

fn build_window_submenu(app: &AppHandle, page: CloudPage) -> tauri::Result<WindowSubmenuBundle> {
    let show_item = MenuItem::with_id(
        app,
        format!("show_{}", page_label(page)),
        format!("显示{}", page_name(page)),
        true,
        None::<&str>,
    )?;
    let hide_item = MenuItem::with_id(
        app,
        format!("hide_{}", page_label(page)),
        format!("隐藏{}", page_name(page)),
        true,
        None::<&str>,
    )?;
    let normal_item = CheckMenuItem::with_id(
        app,
        format!("{}_normal_mode", page_label(page)),
        "正常窗口",
        true,
        is_normal_window(page_preferences(app, page)),
        None::<&str>,
    )?;
    let desktop_item = CheckMenuItem::with_id(
        app,
        format!("{}_desktop_mode", page_label(page)),
        "固定到桌面层",
        true,
        page_preferences(app, page).desktop_mode,
        None::<&str>,
    )?;
    let top_item = CheckMenuItem::with_id(
        app,
        format!("{}_always_on_top", page_label(page)),
        "窗口置顶",
        true,
        page_preferences(app, page).always_on_top,
        None::<&str>,
    )?;
    let browser_item = MenuItem::with_id(
        app,
        format!("browser_{}", page_label(page)),
        "在浏览器中打开",
        true,
        None::<&str>,
    )?;
    let settings_item = MenuItem::with_id(
        app,
        format!("settings_{}", page_label(page)),
        "缩放、透明度与导出...",
        true,
        None::<&str>,
    )?;

    let submenu = Submenu::with_items(
        app,
        page_name(page),
        true,
        &[
            &show_item,
            &hide_item,
            &normal_item,
            &desktop_item,
            &top_item,
            &browser_item,
            &settings_item,
        ],
    )?;

    Ok((submenu, normal_item, desktop_item, top_item))
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let settings_item = MenuItem::with_id(
        app,
        "window_settings",
        "窗口设置与导出...",
        true,
        None::<&str>,
    )?;
    let pin_all_item =
        MenuItem::with_id(app, "pin_all", "一键固定全部窗口到桌面", true, None::<&str>)?;
    let normal_all_item =
        MenuItem::with_id(app, "normal_all", "全部恢复正常窗口", true, None::<&str>)?;
    let show_all_item =
        MenuItem::with_id(app, "show_workspace", "显示全部窗口", true, None::<&str>)?;
    let hide_all_item =
        MenuItem::with_id(app, "hide_workspace", "隐藏全部窗口", true, None::<&str>)?;
    let check_updates_item =
        MenuItem::with_id(app, "check_updates", "检查更新", true, None::<&str>)?;
    let print_calendar_item =
        MenuItem::with_id(app, "print_calendar", "打印日历...", true, None::<&str>)?;
    let notify_item = MenuItem::with_id(app, "notify", "测试通知", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let (calendar_submenu, calendar_normal, calendar_desktop, calendar_top) =
        build_window_submenu(app, CloudPage::Calendar)?;
    let (reminders_submenu, reminders_normal, reminders_desktop, reminders_top) =
        build_window_submenu(app, CloudPage::Reminders)?;
    let (notes_submenu, notes_normal, notes_desktop, notes_top) =
        build_window_submenu(app, CloudPage::Notes)?;

    let controls = WindowMenuControls {
        calendar_normal,
        calendar_desktop,
        calendar_top,
        reminders_normal,
        reminders_desktop,
        reminders_top,
        notes_normal,
        notes_desktop,
        notes_top,
    };
    sync_window_menu_controls(&controls, app)?;
    app.manage(controls.clone());

    let event_controls = controls.clone();
    let mut tray = TrayIconBuilder::new()
        .tooltip(APP_TITLE)
        .menu(&Menu::with_items(
            app,
            &[
                &show_all_item,
                &settings_item,
                &pin_all_item,
                &normal_all_item,
                &check_updates_item,
                &print_calendar_item,
                &calendar_submenu,
                &reminders_submenu,
                &notes_submenu,
                &hide_all_item,
                &separator,
                &notify_item,
                &quit_item,
            ],
        )?)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "window_settings" | "settings_calendar" | "settings_reminders"
                | "settings_notes" => {
                    let _ = open_controls(app);
                }
                "pin_all" | "normal_all" => {
                    if let Err(error) =
                        desktop_controls::set_workspace_mode(app, event.id.as_ref() == "pin_all")
                    {
                        desktop_controls::show_error(app, &error);
                    }
                }
                "show_workspace" => {
                    let _ = show_workspace(app, true);
                }
                "check_updates" => {
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        check_for_updates(app_handle, true).await;
                    });
                }
                "print_calendar" => {
                    let _ = print_page(app, CloudPage::Calendar);
                }
                "show_calendar" => {
                    let _ = show_window(app, CloudPage::Calendar, true, true);
                }
                "hide_calendar" => {
                    let _ = hide_window(app, CloudPage::Calendar, true);
                }
                "calendar_normal_mode" => {
                    let _ = set_normal_mode(app, CloudPage::Calendar);
                    let _ = show_window(app, CloudPage::Calendar, false, true);
                }
                "calendar_desktop_mode" => {
                    let _ = toggle_desktop_mode(app, CloudPage::Calendar);
                    let _ = show_window(app, CloudPage::Calendar, false, true);
                }
                "calendar_always_on_top" => {
                    let _ = toggle_always_on_top(app, CloudPage::Calendar);
                    let _ = show_window(app, CloudPage::Calendar, false, true);
                }
                "browser_calendar" => {
                    let _ = app
                        .opener()
                        .open_url(browser_url_for(CloudPage::Calendar, app), None::<&str>);
                }
                "show_reminders" => {
                    let _ = show_window(app, CloudPage::Reminders, true, true);
                }
                "hide_reminders" => {
                    let _ = hide_window(app, CloudPage::Reminders, true);
                }
                "reminders_normal_mode" => {
                    let _ = set_normal_mode(app, CloudPage::Reminders);
                    let _ = show_window(app, CloudPage::Reminders, false, true);
                }
                "reminders_desktop_mode" => {
                    let _ = toggle_desktop_mode(app, CloudPage::Reminders);
                    let _ = show_window(app, CloudPage::Reminders, false, true);
                }
                "reminders_always_on_top" => {
                    let _ = toggle_always_on_top(app, CloudPage::Reminders);
                    let _ = show_window(app, CloudPage::Reminders, false, true);
                }
                "browser_reminders" => {
                    let _ = app
                        .opener()
                        .open_url(browser_url_for(CloudPage::Reminders, app), None::<&str>);
                }
                "show_notes" => {
                    let _ = show_window(app, CloudPage::Notes, true, true);
                }
                "hide_notes" => {
                    let _ = hide_window(app, CloudPage::Notes, true);
                }
                "notes_normal_mode" => {
                    let _ = set_normal_mode(app, CloudPage::Notes);
                    let _ = show_window(app, CloudPage::Notes, false, true);
                }
                "notes_desktop_mode" => {
                    let _ = toggle_desktop_mode(app, CloudPage::Notes);
                    let _ = show_window(app, CloudPage::Notes, false, true);
                }
                "notes_always_on_top" => {
                    let _ = toggle_always_on_top(app, CloudPage::Notes);
                    let _ = show_window(app, CloudPage::Notes, false, true);
                }
                "browser_notes" => {
                    let _ = app
                        .opener()
                        .open_url(browser_url_for(CloudPage::Notes, app), None::<&str>);
                }
                "hide_workspace" => {
                    let _ = hide_workspace(app);
                }
                "notify" => {
                    show_shell_notification(app, APP_TITLE, "原生通知功能已启用。");
                }
                "quit" => {
                    quit_app(app);
                }
                _ => {}
            }

            let _ = sync_window_menu_controls(&event_controls, app);
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = show_workspace(tray.app_handle(), true);
            }
        });

    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }

    tray.build(app)?;
    Ok(())
}

fn initialize_windows(app: &AppHandle) -> tauri::Result<()> {
    let launch_minimized = std::env::args().any(|arg| arg == "--minimized");
    let pages_to_show = visible_pages(app);

    for page in pages_to_show {
        let _ = restore_and_prepare_window(app, page)?;
    }

    if !launch_minimized {
        show_workspace(app, true)?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            desktop_controls::get_window_preferences,
            desktop_controls::set_window_preferences,
            desktop_controls::set_all_window_modes,
            desktop_controls::export_current_page,
            desktop_controls::save_calendar_export,
        ])
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = show_workspace(app, true);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            ensure_autostart(&app_handle);

            let deferred_app = app_handle.clone();
            let startup_update_app = app_handle.clone();
            app.run_on_main_thread(move || {
                if let Err(error) = initialize_windows(&deferred_app) {
                    eprintln!("failed to initialize windows: {error}");
                }
                tauri::async_runtime::spawn(async move {
                    check_for_updates(startup_update_app, false).await;
                });
            })?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
