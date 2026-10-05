use super::*;
use tauri::{Emitter, Url};

const SETTINGS_LABEL: &str = "settings";

pub(super) fn show_error(app: &AppHandle, error: &str) {
    app.dialog()
        .message(error)
        .title("VibeCal")
        .kind(MessageDialogKind::Error)
        .show(|_| {});
}

fn require_settings(window: &WebviewWindow) -> Result<(), String> {
    let local_origin = window
        .url()
        .map(|url| {
            (url.scheme() == "tauri" && url.host_str() == Some("localhost"))
                || (matches!(url.scheme(), "http" | "https")
                    && url.host_str() == Some("tauri.localhost"))
        })
        .unwrap_or(false);
    if window.label() == SETTINGS_LABEL && local_origin {
        Ok(())
    } else {
        Err("此操作只能在本地窗口设置中使用。".into())
    }
}

fn parse_page(page: &str) -> Result<CloudPage, String> {
    match page {
        "calendar" => Ok(CloudPage::Calendar),
        "reminders" => Ok(CloudPage::Reminders),
        "notes" => Ok(CloudPage::Notes),
        _ => Err("未知窗口。".into()),
    }
}

pub(super) fn is_external_link(url: &Url) -> bool {
    if !matches!(url.scheme(), "http" | "https" | "mailto" | "tel") {
        return false;
    }
    let host = url.host_str().unwrap_or("");
    ![
        "icloud.com",
        "icloud.com.cn",
        "appleid.apple.com",
        "idmsa.apple.com",
        "account.apple.com",
    ]
    .iter()
    .any(|allowed| host == *allowed || host.ends_with(&format!(".{allowed}")))
}

pub(super) fn open_external_link(app: &AppHandle, url: &Url) {
    if matches!(url.scheme(), "http" | "https" | "mailto" | "tel") {
        if let Err(error) = app.opener().open_url(url.as_str(), None::<&str>) {
            show_error(app, &format!("无法打开链接：{error}"));
        }
    }
}

fn fitted_zoom(width: f64, height: f64, label: &str, manual: f64, auto_fit: bool) -> f64 {
    if !auto_fit {
        return manual;
    }
    let (reference_width, reference_height) = match label {
        "calendar" => (1180.0, 900.0),
        "reminders" => (420.0, 430.0),
        _ => (420.0, 438.0),
    };
    (width / reference_width)
        .min(height / reference_height)
        .clamp(0.25, manual)
}

pub(super) fn apply_appearance(
    window: &WebviewWindow,
    preferences: WindowPreferences,
) -> tauri::Result<()> {
    let size = window.inner_size()?;
    let scale = window.scale_factor()?;
    window.set_zoom(fitted_zoom(
        size.width as f64 / scale,
        size.height as f64 / scale,
        window.label(),
        preferences.zoom,
        preferences.auto_fit,
    ))?;
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::{
            Foundation::{GetLastError, SetLastError, COLORREF, WIN32_ERROR},
            UI::WindowsAndMessaging::{
                GetWindowLongPtrW, SetLayeredWindowAttributes, SetWindowLongPtrW, GWL_EXSTYLE,
                LWA_ALPHA, WS_EX_LAYERED,
            },
        };
        let hwnd = window.hwnd()?;
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        if preferences.opacity == 100 && style & WS_EX_LAYERED.0 as isize == 0 {
            return Ok(());
        }
        // Keep the layered flag at 100% too; clearing it can disrupt WebView2 compositing.
        if style & WS_EX_LAYERED.0 as isize == 0 {
            SetLastError(WIN32_ERROR(0));
            if SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_LAYERED.0 as isize) == 0
                && GetLastError().0 != 0
            {
                return Err(tauri::Error::Anyhow(
                    windows::core::Error::from_win32().into(),
                ));
            }
        }
        SetLayeredWindowAttributes(
            hwnd,
            COLORREF(0),
            ((preferences.opacity as u16 * 255 + 50) / 100) as u8,
            LWA_ALPHA,
        )
        .map_err(|error| tauri::Error::Anyhow(error.into()))?;
    }
    Ok(())
}

pub(super) fn open_controls(app: &AppHandle) -> tauri::Result<()> {
    let window = if let Some(window) = app.get_webview_window(SETTINGS_LABEL) {
        window
    } else {
        let links_app = app.clone();
        WebviewWindowBuilder::new(app, SETTINGS_LABEL, WebviewUrl::App("index.html".into()))
            .title("VibeCal 窗口设置与导出")
            .inner_size(600.0, 790.0)
            .min_inner_size(460.0, 500.0)
            .on_new_window(move |url, _| {
                open_external_link(&links_app, &url);
                tauri::webview::NewWindowResponse::Deny
            })
            .build()?
    };
    window.unminimize()?;
    window.show()?;
    window.set_focus()?;
    Ok(())
}

#[tauri::command]
pub(super) fn get_window_preferences(
    window: WebviewWindow,
    app: AppHandle,
) -> Result<AppPreferences, String> {
    require_settings(&window)?;
    Ok(snapshot_preferences(&app))
}

#[tauri::command]
pub(super) fn set_window_preferences(
    window: WebviewWindow,
    app: AppHandle,
    page: String,
    zoom: f64,
    opacity: u8,
    auto_fit: bool,
    mode: String,
) -> Result<AppPreferences, String> {
    require_settings(&window)?;
    let page = parse_page(&page)?;
    if !zoom.is_finite() || !(0.25..=1.5).contains(&zoom) || !(30..=100).contains(&opacity) {
        return Err("缩放应为 25%–150%，不透明度应为 30%–100%。".into());
    }
    if !["normal", "desktop", "top"].contains(&mode.as_str()) {
        return Err("未知窗口模式。".into());
    }
    let old_preferences = page_preferences(&app, page);
    let preferences = WindowPreferences {
        zoom,
        opacity,
        auto_fit,
        desktop_mode: mode == "desktop",
        always_on_top: mode == "top",
        ..old_preferences
    };
    let target = ensure_window(&app, page).map_err(|error| error.to_string())?;
    if let Err(error) = apply_window_modes(&target, preferences) {
        let _ = apply_window_modes(&target, old_preferences);
        return Err(error.to_string());
    }
    update_page_preferences(&app, page, |stored| *stored = preferences);
    Ok(snapshot_preferences(&app))
}

pub(super) fn set_workspace_mode(app: &AppHandle, desktop: bool) -> Result<(), String> {
    for page in pages() {
        let mut preferences = page_preferences(app, page);
        preferences.desktop_mode = desktop;
        preferences.always_on_top = false;
        let window = ensure_window(app, page).map_err(|error| error.to_string())?;
        apply_window_modes(&window, preferences).map_err(|error| error.to_string())?;
        update_page_preferences(app, page, |stored| *stored = preferences);
        show_window(app, page, false, true).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub(super) fn set_all_window_modes(
    window: WebviewWindow,
    app: AppHandle,
    desktop: bool,
) -> Result<AppPreferences, String> {
    require_settings(&window)?;
    set_workspace_mode(&app, desktop)?;
    Ok(snapshot_preferences(&app))
}

#[derive(Deserialize)]
struct PageText {
    title: String,
    text: String,
}
#[derive(Deserialize)]
struct PageCapture {
    records: Vec<PageText>,
    missing: usize,
}

fn csv_field(value: &str) -> String {
    // Neutralize spreadsheet formulas, including those preceded by whitespace.
    let prefix = if value.trim_start().starts_with(['=', '+', '-', '@']) {
        "'"
    } else {
        ""
    };
    format!("\"{prefix}{}\"", value.replace('"', "\"\""))
}

fn serialize_capture(capture: &PageCapture, page: CloudPage, format: &str) -> String {
    if format == "csv" {
        let mut output = String::from("\u{feff}Source,Page Title,Line,Text\r\n");
        for record in &capture.records {
            for (index, line) in record.text.lines().enumerate() {
                output.push_str(&format!(
                    "{},{},{},{}\r\n",
                    csv_field(page_name(page)),
                    csv_field(&record.title),
                    index + 1,
                    csv_field(line)
                ));
            }
        }
        output
    } else {
        capture
            .records
            .iter()
            .map(|record| format!("{}\r\n{}", record.title, record.text))
            .collect::<Vec<_>>()
            .join("\r\n\r\n")
    }
}

#[cfg(target_os = "windows")]
fn evaluate(
    window: &WebviewWindow,
    script: &str,
    callback: impl FnOnce(Result<String, String>) + Send + 'static,
) -> tauri::Result<()> {
    let script = script.to_string();
    let callback = Arc::new(Mutex::new(Some(callback)));
    window.with_webview(move |webview| unsafe {
        use webview2_com::ExecuteScriptCompletedHandler;
        use windows::core::HSTRING;
        let completion = callback.clone();
        let result = webview.controller().CoreWebView2().and_then(|core| {
            core.ExecuteScript(
                &HSTRING::from(script),
                &ExecuteScriptCompletedHandler::create(Box::new(move |result, json| {
                    if let Some(callback) = completion.lock().unwrap().take() {
                        callback(result.map(|_| json).map_err(|error| error.to_string()));
                    }
                    Ok(())
                })),
            )
        });
        if let Err(error) = result {
            if let Some(callback) = callback.lock().unwrap().take() {
                callback(Err(error.to_string()));
            }
        }
    })
}

#[tauri::command]
pub(super) fn export_current_page(
    window: WebviewWindow,
    app: AppHandle,
    page: String,
    format: String,
) -> Result<(), String> {
    require_settings(&window)?;
    let page = parse_page(&page)?;
    if !["txt", "csv"].contains(&format.as_str()) {
        return Err("未知导出格式。".into());
    }
    let target = ensure_window(&app, page).map_err(|error| error.to_string())?;
    #[cfg(target_os = "windows")]
    {
        let finish_app = app.clone();
        let error_app = app.clone();
        evaluate(&target.clone(), "window.__vibecalExport?.start() || false", move |result| {
            if !matches!(result.as_deref(), Ok("true")) {
                export_status(&finish_app, "页面尚未就绪，请先登录并打开要导出的内容。");
                return;
            }
            // Frame replies are asynchronous. Read after a short delay without blocking the UI thread.
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(800));
                let callback_app = finish_app.clone();
                let read_error_app = finish_app.clone();
                if let Err(error) = evaluate(&target, "window.__vibecalExport.finish()", move |result| {
                    match result.and_then(|json| serde_json::from_str::<PageCapture>(&json).map_err(|error| error.to_string())) {
                        Ok(capture) if capture.missing > 0 => export_status(&callback_app,
                            "部分页面尚未响应，无法完整读取。请等待页面加载后重试，或在浏览器中复制文字。"),
                        Ok(capture) if capture.records.is_empty() => export_status(&callback_app,
                            "没有读到页面文字。请先登录，并打开需要导出的日历、列表或备忘录。"),
                        Ok(capture) => save_capture(&callback_app, capture, page, &format),
                        Err(error) => export_status(&callback_app, &format!("读取页面失败：{error}")),
                    }
                }) { export_status(&read_error_app, &error.to_string()); }
            });
        }).map_err(|error| {
            export_status(&error_app, &error.to_string());
            error.to_string()
        })?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = target;
        Err("此导出功能仅支持 Windows。".into())
    }
}

fn export_status(app: &AppHandle, message: &str) {
    let _ = app.emit_to(SETTINGS_LABEL, "export-status", message);
}

fn save_capture(app: &AppHandle, capture: PageCapture, page: CloudPage, format: &str) {
    let output = serialize_capture(&capture, page, format);
    save_output(
        app,
        output,
        format!("VibeCal-{}.{}", page_label(page), format),
        format,
        "导出已加载的页面文字（不包含未加载的条目）",
        "导出完成。仅包含当前页面已加载的文字。",
    );
}

#[tauri::command]
pub(super) fn save_calendar_export(
    window: WebviewWindow,
    app: AppHandle,
    text: String,
    format: String,
) -> Result<(), String> {
    require_settings(&window)?;
    if !["txt", "csv"].contains(&format.as_str()) || text.is_empty() || text.len() > 20_000_000 {
        return Err("导出内容或格式无效。".into());
    }
    save_output(
        &app,
        text,
        format!("VibeCal-calendar.{}", format),
        &format,
        "保存从 ICS 转换的日历",
        "日历转换并保存完成。",
    );
    Ok(())
}

fn save_output(
    app: &AppHandle,
    output: String,
    filename: String,
    format: &str,
    title: &str,
    success: &str,
) {
    let success = success.to_string();
    let app = app.clone();
    let callback_app = app.clone();
    let mut picker = app
        .dialog()
        .file()
        .set_title(title)
        .set_file_name(filename)
        .add_filter(format.to_uppercase(), &[format]);
    if let Some(window) = app.get_webview_window(SETTINGS_LABEL) {
        picker = picker.set_parent(&window);
    }
    picker.save_file(move |path| {
        if let Some(path) = path {
            let result = path
                .into_path()
                .map_err(|error| error.to_string())
                .and_then(|path| fs::write(path, &output).map_err(|error| error.to_string()));
            match result {
                Ok(()) => export_status(&callback_app, &success),
                Err(error) => export_status(&callback_app, &format!("保存失败：{error}")),
            }
        } else {
            export_status(&callback_app, "已取消导出。");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_settings_keep_full_opacity_and_zoom() {
        let prefs: AppPreferences =
            serde_json::from_str(r#"{"calendar":{"visible":true}}"#).unwrap();
        let prefs = prefs.normalized();
        assert_eq!(prefs.calendar.zoom, 1.0);
        assert_eq!(prefs.calendar.opacity, 100);
        assert_eq!(AppPreferences::default().normalized().notes.opacity, 100);
    }
    #[test]
    fn zoom_fits_logical_window_bounds() {
        assert_eq!(fitted_zoom(590.0, 450.0, "calendar", 1.0, true), 0.5);
        assert_eq!(fitted_zoom(1180.0, 900.0, "calendar", 0.8, true), 0.8);
        assert_eq!(fitted_zoom(360.0, 280.0, "calendar", 1.0, false), 1.0);
    }
    #[test]
    fn links_preserve_login_and_reject_lookalike_hosts() {
        assert!(!is_external_link(
            &Url::parse("https://idmsa.apple.com/login").unwrap()
        ));
        assert!(!is_external_link(
            &Url::parse("https://www.icloud.com.cn/reminders/").unwrap()
        ));
        assert!(is_external_link(
            &Url::parse("https://icloud.com.evil.test/").unwrap()
        ));
        assert!(is_external_link(
            &Url::parse("https://example.com/").unwrap()
        ));
        assert!(is_external_link(
            &Url::parse("mailto:person@example.com").unwrap()
        ));
    }
    #[test]
    fn csv_preserves_multilingual_content_and_quotes_without_executing_formulas() {
        assert_eq!(csv_field("中文,\"note\""), "\"中文,\"\"note\"\"\"");
        assert_eq!(csv_field("  =1+1"), "\"'  =1+1\"");
        let capture = PageCapture {
            records: vec![PageText {
                title: "Memo".into(),
                text: "中文\nnext".into(),
            }],
            missing: 0,
        };
        assert!(serialize_capture(&capture, CloudPage::Notes, "csv").starts_with('\u{feff}'));
        assert!(serialize_capture(&capture, CloudPage::Notes, "txt").contains("中文\nnext"));
    }
}
