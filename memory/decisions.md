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

### Public Branding
- The public project name is renamed to `VibeCal`.
- The public codename and package naming should use `vibecal` rather than `apple-calendar-desktop`.
- Public-facing naming should describe Apple Calendar compatibility without presenting the project as an Apple product.
- The app should include an explicit disclaimer that it is unofficial and not affiliated with, sponsored by, or endorsed by Apple.
- The app identifier should be neutral and use `com.vibecal.desktop`.
- The renamed build should attempt to migrate legacy local state from `com.local.applecalendardesktop` so the existing personal user does not lose their signed-in session during the rebrand.

### Multi-Surface Workspace Direction
- The next product step is not a generic widget system first; it is a Calendar main window plus a Reminders companion window.
- Calendar and Reminders should open together by default, while still allowing the user to close either one independently afterward.
- The Reminders companion should support multiple presentation modes, with right-dock mode as the default.
- Floating mini and bottom-dock layouts remain supported modes, but they are secondary to the default right-dock experience.
- Daily habits should initially be modeled inside Apple Reminders through a dedicated `Habits` list with recurring reminders, rather than through a separate custom habit-tracking data model.
- The initial implementation should keep both surfaces as separate Tauri webview windows that share one persisted WebView data directory, instead of refactoring into a custom local shell with embedded sub-webviews.

### Widget Direction
- The first macOS-widget-inspired step should be a local `Today Board` window rather than a custom Apple data widget.
- The `Today Board` should provide quick actions, layout switching, and habit guidance while leaving actual Apple data inside the Calendar and Reminders web surfaces.
- The app should not scrape or mirror Apple reminder content into a local widget until there is a clearer and safer integration path.

### UX Correction After Validation
- The experimental `Today Board` widget should be removed because it does not meaningfully help the primary workflow.
- The workspace should center on Apple-owned surfaces only: Calendar, Reminders, and Notes.
- When no persisted cookie domain exists yet, the app should default to `https://www.icloud.com.cn/`-based pages rather than the global iCloud domain.
- Right-dock layout should no longer resize the main Calendar window; companion panels should follow the Calendar window instead of controlling its size.
- The right-dock experience should stack Reminders and Notes vertically as the default sidebar arrangement.

## 2026-04-15

### Dashboard Shell Direction
- The separate companion-window approach is no longer the preferred UX direction.
- The workspace should move to a single dashboard shell that contains Calendar, Reminders, and Notes in one desktop window.
- The default layout is Calendar on the left, Reminders on the upper right, and Notes on the lower right.
- The dashboard should allow direct dragging of both the left/right boundary and the Reminders/Notes divider.

### Window Pinning Modes
- The whole dashboard, not just one surface, should support both desktop-layer mode and always-on-top mode.
- Desktop-layer mode and always-on-top mode are mutually exclusive in persisted state.

### Quick Add Direction
- The first Quick Add pass should target Reminders and Notes only.
- Calendar should keep a `New Event` shortcut path instead of promising native direct event creation.
- The Quick Add implementation is allowed to drive the signed-in Apple web panels through injected scripts because there is no established Windows-native Apple API path in this project for directly writing iCloud Calendar, Reminders, or Notes data.
- This “no native Windows API path” conclusion is an inference from Apple’s public platform support and web-product documentation, not from a special private source.

### Implementation Direction
- The new local dashboard frontend should live in `dashboard/` and be bundled as the app frontend.
- Apple web content should live in Tauri child webviews that share a single persisted WebView data directory.
- The dashboard shell should provide the resize handles and local control chrome, while the Apple pages remain the source of truth for actual content.

### Dashboard Reversal After User Validation
- The dashboard-shell experiment did not meet the user experience target and should not remain the primary direction.
- The app should return to independent top-level Calendar, Reminders, and Notes windows.
- The windows should not auto-dock, auto-follow, or enforce panel ratios once opened.
- User-controlled free resizing is more important than simulated widget chrome for this project.
- The local dashboard frontend and Quick Add controls should be removed rather than carried forward as a parallel UX path.
- Desktop-layer mode and always-on-top mode should remain available, but they must not disable free resizing of the windows.

### Per-Window Pinning Controls
- Desktop-layer mode and always-on-top mode should no longer be global toggles.
- Calendar, Reminders, and Notes should each have their own independent `Pin to Desktop Layer` and `Always On Top` controls.
- These per-window controls should be exposed from the tray menu so the user can manage any window without reintroducing a local dashboard shell.
- Legacy global pinning settings should be treated as migration-only input and normalized into the new per-window preference model.
- `Always On Top` should be reinforced with direct Win32 z-order calls on Windows instead of relying only on higher-level wrapper methods.

### Desktop-Layer UX
- Desktop-layer mode should return to the older feel the user preferred: the pinned window drops its normal title-bar chrome and no longer behaves like a regular freely movable window.
- A direct native Windows desktop-host attachment was tested and then rolled back because it caused launch-time invisibility and poor accessibility for real use.
- The accepted desktop-layer behavior is the earlier, more stable implementation: chrome-free window, fixed size, skip-taskbar, and bottom-layer ordering without desktop-host reparenting.

### Visibility Memory
- The app should no longer reopen every page on every launch.
- Calendar should be the only default visible page on a fresh setup.
- Reminders and Notes should become opt-in windows that reopen only if the user left them visible in the last remembered workspace.
- Once the new visibility fields exist in local settings, the app should respect the remembered workspace exactly, including the case where the user previously hid every content window and only left the tray running.

### Snap Reversal
- Automatic snap alignment should be removed again because the user prefers unconstrained manual positioning over proximity snapping.

### Startup Stability
- Window creation should be deferred out of the Tauri `setup` hook onto the main thread because eager WebView window creation during `setup` triggered a Windows `os error 183` failure in validation.
- Showing a pinned window should apply its window mode before `show()` so the app does not visibly flash a normal window and then appear to disappear.

## 2026-04-16

### Calendar Printing
- The new release should add a tray-level `Print Calendar...` action rather than a custom export format.
- Printing should be delegated to the Apple Calendar web surface through `window.print()` so the app keeps Apple-owned layout and avoids rebuilding calendar rendering logic locally.
- The print action should bring the Calendar window to the front before opening the print dialog.

### Release Version
- The first public release that includes calendar printing is version `0.2.0`.

### UI Language
- The tray-menu control surface and window titles should use Chinese labels because the user's primary operating language for day-to-day app controls is Chinese.
