# VibeCal

VibeCal is an unofficial Windows 11 desktop wrapper compatible with Apple Calendar, built with Tauri 2 and WebView2.

It keeps the Apple Calendar web experience intact and adds desktop-focused behavior such as tray support, single-instance launching, auto start, window state restore, and an optional desktop layer mode.

## Disclaimer

- This project is unofficial and is not affiliated with, sponsored by, or endorsed by Apple Inc.
- Apple and iCloud are trademarks of Apple Inc.
- This project only provides a desktop wrapper around the public Apple Calendar web experience.

## Features

- Direct Apple Calendar window on Windows 11
- Persistent WebView2 session data across launches
- Window size, position, and state restore
- Hide to tray on close
- Single-instance behavior
- Auto start support
- Native Windows notification plumbing
- Optional desktop layer mode that can sit behind fullscreen apps
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

## Known Constraints

- Apple sign-in and Apple web compatibility are external dependencies.
- Apple may change login, cookie, or embedded web behavior in ways that require app updates.
- Native reminder parity depends on what Apple exposes inside the WebView-based experience.

## License

No license has been added yet. If you plan to publish this repository publicly, add a license before inviting contributions or reuse.
