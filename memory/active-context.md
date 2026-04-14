# Active Context

## Current Phase
- The personal-use v1 desktop client is complete for the current scope and the repository is now cleaned up for GitHub publication.

## Project Snapshot
- Project name: Apple Calendar Desktop
- Internal codename: apple-calendar-desktop
- Product direction: desktop wrapper around Apple Calendar web experience
- Primary user: single personal Windows 11 user
- Preferred stack: Tauri + WebView2

## Confirmed v1 Focus
- Stable access to Apple Calendar from a desktop app
- Persistent login state when technically possible
- Window state memory
- Tray support
- Auto start
- Single instance behavior
- Windows native notifications
- Easy desktop or taskbar pinning

## Key Constraint
- Apple sign-in and web compatibility are external dependencies. If Apple changes login or embedding behavior, the client may require adjustment.

## Current Implementation Snapshot
- The app now uses a Tauri-managed external WebView aimed at `https://www.icloud.com/calendar/`.
- Desktop behaviors are wired from Rust, not from a custom frontend shell.
- Session persistence is currently designed around a dedicated WebView data directory.
- Tray, single-instance behavior, autostart wiring, window-state restore, and shell-level notifications are included in the first skeleton.
- The system Rust MSVC toolchain and Visual Studio Build Tools are now installed and recognized by `tauri info`.
- `cargo check` and `cargo build` both succeed for the current Rust app.
- `npm run dev` was validated to the point where the Tauri dev runner launched the desktop process successfully.
- Runtime checks confirmed app launch, single-instance behavior, autostart registration, WebView data-directory creation, and window-state restore across relaunch.
- Startup URL selection is now cookie-aware and prefers the China domain when the persisted WebView profile contains `.icloud.com.cn` cookies.
- The app now supports a persistent desktop mode that loads from local settings and applies bottom-layer window behavior.
- User validation confirmed that login now survives full app restart and that desktop mode behaves correctly with fullscreen browsers covering and then revealing the window again.
- Both debug and release Windows executables are now built as GUI-subsystem binaries, so launching them directly does not spawn an extra console window.
- The repository now includes a reset script for clearing local runtime state and a public-facing README that explains where session data and autostart entries live on Windows.
- An initial local Git repository has been created on the `main` branch, and the publishable working tree excludes ignored caches, bootstrap leftovers, `node_modules`, and build output.

## Current Blockers
- No functional blockers remain for the current personal-use v1 target.

## Immediate Next Step
- Optional future work: add a license, create the first commit, and publish to GitHub.
