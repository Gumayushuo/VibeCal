// Run as a GUI app on Windows so the user-facing binary never spawns a console window.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() {
    apple_calendar_desktop_lib::run()
}
