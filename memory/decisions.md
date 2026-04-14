# Decisions

## 2026-04-13

### Project Identity
- Chosen product name: `Apple Calendar Desktop`
- Chosen internal codename: `apple-calendar-desktop`

### Product Direction
- The project is for personal use only.
- The selected direction is a desktop wrapper plus desktop-oriented enhancements, not a fully custom calendar client.

### Technology Direction
- Preferred v1 stack: `Tauri + WebView2`
- `Electron` remains a fallback option, not the default.

### Authentication Direction
- Embedded sign-in is not required.
- Browser-assisted first-time sign-in is acceptable if it leads to a more stable persisted session.

### v1 Feature Priorities
- Persistent login
- Window state memory
- Tray support
- Auto start
- Single instance behavior
- Windows native notifications
- Easy desktop-style launching and pinning

### Accepted Constraint
- The project may need adjustment if Apple changes login, cookie handling, or embedded web compatibility.

### Implementation Direction
- The first implementation pass uses a remote Apple Calendar WebView as the main app surface instead of a local HTML shell.
- Desktop features are implemented from the Rust/Tauri side to avoid relying on injected APIs inside Apple web content.
- The app should not assume that a browser-assisted login flow will transfer session state into the app's WebView profile.

### Session Strategy
- Persistent login should be attempted through a dedicated WebView data directory so cookies and related browsing state survive app restarts.
- Startup domain selection should follow the persisted cookie domain, preferring `icloud.com.cn` when China-domain Apple cookies are detected.

### Notification Scope
- The first skeleton wires native shell notifications for desktop integration.
- Full parity with Apple reminder notifications remains an open integration question and depends on Apple web behavior inside WebView2.

### Environment Baseline
- The repository can now rely on a system-level Rust MSVC toolchain and Visual Studio Build Tools instead of workspace-local bootstrap installs.

### First Validation Result
- The first implementation pass compiles and links successfully.
- Verified runtime behavior currently includes app launch, auto-start registration, single-instance enforcement, creation of a dedicated WebView profile directory, and window-state restore across relaunch.
- Apple account session persistence is still unverified because it depends on a real sign-in flow.

### Desktop Mode
- Desktop mode is implemented as a persistent window mode rather than a separate app variant.
- In desktop mode, the window is configured for bottom-layer behavior and loads from local settings on startup.

## 2026-04-14

### Validation Result
- User validation confirmed that the signed-in Apple Calendar session now persists across full app restarts.
- User validation confirmed that desktop mode behaves as intended: fullscreen applications can cover the calendar window and the window reappears in place afterward.

### Launch Experience
- The Windows entry binary should use the GUI subsystem in both debug and release builds so direct launches do not show an unnecessary console window.

### Publication And Local State
- Repository publication should not include runtime session state because Apple login data is stored in `%LOCALAPPDATA%\com.local.applecalendardesktop`, not inside the repository.
- Windows auto start for this app is represented by a user-level registry entry under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\Apple Calendar Desktop`.
- The repository should include an explicit local reset script and README guidance so users can clear machine-specific state without guessing which directory or registry value to remove.
