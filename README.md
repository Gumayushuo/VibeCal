# VibeCal

VibeCal is an unofficial Windows 11 desktop wrapper compatible with Apple Calendar, built with Tauri 2 and WebView2.

It keeps Apple-owned web content intact and wraps it in independent desktop windows with tray support, single-instance launching, auto start, window state restore, and optional pinning modes.

## Disclaimer

- This project is unofficial and is not affiliated with, sponsored by, or endorsed by Apple Inc.
- Apple and iCloud are trademarks of Apple Inc.
- This project only provides a desktop wrapper around the public Apple Calendar web experience.

## Features

- Independent Calendar, Reminders, and Notes windows
- Shared WebView2 session profile across all three windows
- Free manual moving and resizing for each regular window without enforced docking or panel ratios
- Per-window `Pin to Desktop Layer` and `Always On Top` controls from the tray menu
- Window visibility is remembered across launches
- Persistent WebView2 session data across launches
- Window size, position, and state restore
- Hide to tray on close
- Single-instance behavior
- Auto start support
- Native Windows notification plumbing
- Tray action to print the current Calendar view through Apple's web print flow
- Default fallback to the China iCloud domain when no previous cookie domain is known
- Optional desktop layer mode per window
- Optional always-on-top mode per window
- No extra console window when launching the app directly

## Privacy And Local State

This repository does not store or ship your Apple account session.

Runtime state is stored on each local machine, outside the repository:

- Session and WebView data: `%LOCALAPPDATA%\\com.vibecal.desktop\\webview`
- Local app preferences: `%LOCALAPPDATA%\\com.vibecal.desktop\\settings.json`
- Auto start entry: `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\\VibeCal`

That means:

- Cloning this repository on another machine will not sign the other user into your Apple account.
- Uploading this repository to GitHub will not upload your Apple login state as long as ignored files stay untracked.

Legacy local builds may still have state under `%LOCALAPPDATA%\\com.local.applecalendardesktop`. On first launch, the renamed app attempts to migrate that local state into the new VibeCal directory, and the reset script clears both the current and legacy local state locations.

If you want to clear your own local session and auto start entry, run:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\reset-local-state.ps1
```

## Requirements

- Windows 11
- Node.js
- Rust toolchain
- Visual Studio Build Tools for Rust MSVC builds
- Microsoft Edge WebView2 Runtime

Official setup guide: [Tauri prerequisites](https://tauri.app/start/prerequisites/)

## Development

Install dependencies:

```bash
npm install
```

Run the app in development mode:

```bash
npm run dev
```

Build a release bundle:

```bash
npm run build
```

## Repository Notes

- `src-tauri/` contains the Rust and Tauri application.
- `memory/` and `AGENTS.md` keep project context for Codex-assisted iteration.
- `scripts/reset-local-state.ps1` removes current and legacy local runtime data from `%LOCALAPPDATA%` and clears the corresponding Windows auto start entries.
- `node_modules`, build outputs, bootstrap caches, and temporary toolchain folders are ignored and should not be committed.
- The app now defaults to Calendar only unless the user explicitly opens Reminders or Notes.
- The windows share one persisted WebView profile but do not force-follow each other for size or position.
- Tray submenus let you independently show, hide, pin, and top-pin each window.
- The tray menu now includes `Print Calendar...`, which opens the current Apple Calendar print dialog for the Calendar window.
- Closing or hiding a window updates the remembered workspace, so the next launch restores the last page set instead of always reopening every page.
- On a fresh setup, only Calendar is visible by default; after that, relaunch follows the last remembered window set exactly.
- Desktop-layer mode intentionally changes the pinned window into a chrome-free fixed surface, while regular mode keeps full free resize and movement.

## Known Constraints

- Apple sign-in and Apple web compatibility are external dependencies.
- Apple may change login, cookie, or embedded web behavior in ways that require app updates.
- Reminders and Notes are still rendered from Apple web pages rather than a custom local data layer.
- This project intentionally avoids reconstructing Apple data as fake local widgets and instead keeps the real Apple web pages visible in independent windows.

## License

No license has been added yet. If you plan to publish this repository publicly, add a license before inviting contributions or reuse.
