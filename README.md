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
- Explicit per-window `Normal Window` restore action from the tray menu
- Window visibility is remembered across launches
- Persistent WebView2 session data across launches
- Window size, position, and state restore
- Hide to tray on close
- Single-instance behavior
- Auto start support
- Native Windows notification plumbing
- Tray action to print the current Calendar view through Apple's web print flow
- Automatic update checks on startup, with in-app update notes and installation prompts
- Default fallback to the China iCloud domain when no previous cookie domain is known
- Optional desktop layer mode per window
- Optional always-on-top mode per window
- No extra console window when launching the app directly
- Persistent per-window content zoom (25% to 150%) and optional automatic fit while resizing
- Persistent per-window opacity (30% to 100%)
- A local settings panel that stays open for consecutive adjustments
- One-click desktop pinning and normal-window restore for all three windows
- External links open in the default browser, while iCloud navigation and Apple sign-in stay embedded
- Export of currently loaded page text to UTF-8 TXT or CSV
- Local ICS-to-TXT and ICS-to-calendar-CSV conversion with recurrence expansion over a selected date range

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

## Windows 7 Experimental Build

- The normal release remains a Windows 11-targeted build.
- A best-effort Windows 7 installer can be produced with an embedded WebView2 bootstrapper:

```bash
npm run build:win7
```

- This path is experimental only. Microsoft no longer lists Windows 7 as a supported platform for current Edge/WebView2 releases, and Apple may reject or degrade older Chromium/WebView2 builds used on Windows 7.
- The Windows 7 build should therefore be treated as compatibility testing, not as an officially supported target.

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

Build a local installer without generating signed updater artifacts:

```powershell
npm run build:local
```

Run export regression tests and Rust checks:

```powershell
npm test
npm run prepare:controls
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

The local settings assets are bundled from `controls/`. The pre-build step copies the pinned ICAL.js dependency and its MPL-2.0 license into the bundle; no runtime CDN connection is needed. The three iCloud windows continue to load Apple-owned pages with the existing shared WebView profile.

## Window Settings And Export

Open `Window Settings and Export...` from the tray menu (the actual controls are in Chinese). This panel remains open while changing multiple windows. `Pin All Windows to Desktop` also performs the common three-window operation in one click.

Content zoom scales the whole web layout, including fonts. Automatic fit uses the original window dimensions as a reference and reduces zoom as the window becomes smaller, up to the selected maximum. Apple can change its page layout, so a particular calendar view may still need a smaller manual zoom. Switch to normal mode to move or resize a pinned window, then pin it again. Opacity applies to the entire window, including text; 100% is fully opaque. Existing settings keep their original 100% zoom and opacity when upgraded.

Page-text exports include rendered text from currently loaded visible iCloud frames. They do not enumerate the account, load unopened notes, scroll virtualized lists, fetch hidden events, or export attachments. TXT preserves the captured text; CSV uses `Source, Page Title, Line, Text` columns with a UTF-8 BOM for spreadsheet readability and neutralizes formula-like text. This CSV is a text backup and is not an Outlook calendar import.

For calendar migration, choose an existing ICS file in the settings panel and select the export date range. The converter generates TXT or calendar CSV with `Subject, Start Date, Start Time, End Date, End Time, All day event, Description, Location, Private` columns. It handles event exclusions, moved/cancelled occurrences, and supplied `VTIMEZONE` definitions. Missing timezone definitions produce an error instead of silently changing times. Times are converted to the computer's local timezone. CSV flattens recurrence into separate events within the selected range; alarms, attachments, and recurrence rules remain available in the original ICS only. Classic Outlook CSV import may require mapping columns and matching the regional date format; verify a small import first. For Outlook on the web or the new Outlook, import the original ICS.

Apple documents calendar export from Mac and a download workflow from iCloud.com that requires temporarily public sharing. Public sharing makes the calendar readable by anyone holding its link until sharing is disabled. VibeCal does not enable sharing or collect account credentials. See [Apple's archive instructions](https://support.apple.com/en-us/108306) and [Microsoft's calendar import instructions](https://support.microsoft.com/en-us/outlook/import-or-subscribe-to-a-calendar-in-outlook-com-or-outlook-on-the-web).

Build the Windows 7 experimental bundle:

```bash
npm run build:win7
```

Build a signed updater bundle locally:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content C:\path\to\vibecal.key -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "your-password"
npm run build
```

## Auto Updates

VibeCal uses the Tauri updater plugin with a GitHub Releases `latest.json` endpoint.

Client behavior:

- The app checks for updates on startup.
- The tray menu includes a manual update-check action.
- When a newer release exists, the app shows the release notes and asks whether to download and install it.

Release setup:

1. Generate an updater signing key with `npm run tauri signer generate -- --ci -w <path-to-private-key>`.
2. Add `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` to your GitHub repository secrets.
3. Push a version tag such as `v0.3.0`.
  4. GitHub Actions builds a draft release containing the Windows installer, signatures, executable, and `latest.json`.
  5. The workflow verifies the downloaded installer signature against the app's embedded public key for both Windows updater targets, then publishes the verified release as latest. Failed verification leaves the release as a draft and keeps the previous live update unchanged.

Existing users who already have the updater can upgrade directly to a newer release without installing intermediate versions. Keep the original updater public/private key pair, app identifier, and update endpoint. A local build made with `build:local` is suitable for manual installation, but must go through the signing workflow before it can serve as an automatic update.

## Repository Notes

- `src-tauri/` contains the Rust and Tauri application.
- `memory/` and `AGENTS.md` keep project context for Codex-assisted iteration.
- `.github/workflows/publish.yml` builds and publishes signed update artifacts to GitHub Releases when you push a version tag.
- `scripts/reset-local-state.ps1` removes current and legacy local runtime data from `%LOCALAPPDATA%` and clears the corresponding Windows auto start entries.
- `scripts/get-release-notes.ps1` extracts the current version section from `CHANGELOG.md` for release publishing.
- `node_modules`, build outputs, bootstrap caches, and temporary toolchain folders are ignored and should not be committed.
- Fresh installs now open Calendar, Reminders, and Notes together by default.
- The windows share one persisted WebView profile but do not force-follow each other for size or position.
- Tray submenus let you independently show, hide, pin, and top-pin each window.
- The tray menu now includes `Print Calendar...`, which opens the current Apple Calendar print dialog for the Calendar window.
- The tray menu also includes a manual update-check action, and the update prompt shows the release notes supplied by the updater feed.
- Closing or hiding a window updates the remembered workspace, so the next launch restores the last page set instead of always reopening every page.
- If a local settings file would otherwise reopen with every page hidden, the app now restores all three windows instead.
- Desktop-layer mode intentionally changes the pinned window into a chrome-free fixed surface, while regular mode keeps full free resize and movement.

## Known Constraints

- Apple sign-in and Apple web compatibility are external dependencies.
- Apple may change login, cookie, or embedded web behavior in ways that require app updates.
- Reminders and Notes are still rendered from Apple web pages rather than a custom local data layer.
- This project intentionally avoids reconstructing Apple data as fake local widgets and instead keeps the real Apple web pages visible in independent windows.

## License

No license has been added yet. If you plan to publish this repository publicly, add a license before inviting contributions or reuse.
