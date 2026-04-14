// Run as a GUI app on Windows so the user-facing binary never spawns a console window.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() {
    vibecal_lib::run()
}
