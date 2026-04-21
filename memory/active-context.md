# Active Context

## Current Phase
- The personal-use desktop client has been redirected back to a pure multi-window workspace after the dashboard-shell experiment failed the user's UX bar.

## Project Snapshot
- Project name: VibeCal
- Internal codename: vibecal
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
- Calendar and Reminders visible at the same time
- Calendar, Reminders, and Notes visible as independent windows
- No forced docking, panel ratios, or dashboard chrome
- Free manual resizing and moving for each regular window
- Per-window always-on-top mode in addition to per-window desktop-layer mode
- Calendar, Reminders, and Notes visible together on a fresh setup
- Window visibility remembered across launches

## Key Constraint
- Apple sign-in and web compatibility are external dependencies. If Apple changes login or embedding behavior, the client may require adjustment.

## Current Implementation Snapshot
- The app now uses three top-level Tauri webview windows again: Calendar, Reminders, and Notes.
- Desktop behaviors are wired from Rust without a local dashboard frontend.
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
- The public-facing brand is being changed to `VibeCal`, with a neutral app identifier and explicit unofficial Apple compatibility disclaimers.
- The renamed app now attempts a one-time migration from the legacy `com.local.applecalendardesktop` local data directory into the new `com.vibecal.desktop` directory.
- The renamed Rust/Tauri workspace compiles successfully as `vibecal`.
- Calendar, Reminders, and Notes now share one persisted WebView profile while remaining independent windows.
- Default URL fallback now prefers the China iCloud domain when there is no existing cookie signal pointing to the global domain.
- The experimental local `Today Board` widget surface has already been removed.
- The dashboard-shell experiment has been removed from the repo along with the local `dashboard/` frontend.
- The current implementation now opens Calendar, Reminders, and Notes as separate windows and leaves size and position entirely under user control.
- Tray controls are now split per window, so Calendar, Reminders, and Notes can each independently toggle `Pin to Desktop Layer` and `Always On Top`.
- Window snapping has been removed again because it made manual adjustment feel worse instead of better.
- Workspace visibility is now persisted per window so reopened sessions match the user's last chosen page set instead of reopening everything.
- Fresh setups now default to showing Calendar, Reminders, and Notes together, and a fully hidden remembered workspace is automatically reset back to those three visible windows on startup.
- Desktop-layer mode has been restored to the older stable implementation: chrome-free, fixed, skip-taskbar, and bottom-layer behavior without desktop-host reparenting.
- Window initialization has been deferred onto the main thread because direct window creation inside `setup` triggered a Windows `os error 183` crash path during validation.
- The window show path now applies pinned mode before showing the window so startup no longer flashes a normal top-level window and then appears to lose it.
- Release builds and installer output compile successfully for the reverted multi-window architecture.
- The tray now includes a `Print Calendar...` action that triggers the Apple Calendar web print dialog from the Calendar window.
- The app now includes startup and manual update checks wired to GitHub Releases through Tauri's updater plugin.
- The next public release is being prepared as version `0.3.2`.
- The tray menu and window titles are now being localized into Chinese for the user's day-to-day controls.
- A separate best-effort Windows 7 packaging path is now being prepared with an embedded WebView2 bootstrapper, while the officially supported target remains Windows 11.
- Each window tray submenu is gaining an explicit normal-window mode so users can leave `Always On Top` or desktop-layer mode in one click.

## Current Blockers
- No build blocker remains for the multi-window direction.
- Runtime UX validation is still needed to confirm that updater prompts, release-note display, and signed install flow behave correctly against a real GitHub Release.

## Immediate Next Step
- User validation should confirm whether the experimental Windows 7 installer can launch with the older WebView2 path, while the mainline release process continues to target Windows 11.
