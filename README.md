# Apple Calendar Desktop

Apple Calendar Desktop is a Windows 11 desktop wrapper for Apple Calendar built with Tauri 2 and WebView2.

This project keeps the Apple Calendar web experience intact and adds desktop-focused behavior such as tray support, single-instance launching, auto start, window state restore, and an optional desktop layer mode.

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
- `scripts/reset-local-state.ps1` removes local runtime data from `%LOCALAPPDATA%` and clears the Windows auto start entry for this app.
- `node_modules`, build outputs, bootstrap caches, and temporary toolchain folders are ignored and should not be committed.

## Known Constraints

- Apple sign-in and Apple web compatibility are external dependencies.
- Apple may change login, cookie, or embedded web behavior in ways that require app updates.
- Native reminder parity depends on what Apple exposes inside the WebView-based experience.

## License

No license has been added yet. If you plan to publish this repository publicly, add a license before inviting contributions or reuse.
