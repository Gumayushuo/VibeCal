# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2026-04-16

### Added
- Added automatic update checks on startup.
- Added a tray-level manual update-check action.
- Added native update prompts that display the release notes before download and installation.
- Added signed updater artifact generation for Windows releases.
- Added a GitHub Actions publishing workflow that builds signed updater artifacts for GitHub Releases.

### Changed
- Bumped the desktop app and installer version to `0.3.0`.
- Wired the app to the GitHub Releases `latest.json` updater endpoint.
- Updated release documentation so changelog sections can be reused as release notes.

## [0.3.1] - 2026-04-21

### Added
- Added an explicit `Normal Window` mode for each tray submenu so a page can return to a regular movable and resizable window in one click.
- Added a separate `build:win7` packaging path that swaps the WebView2 installer mode to an embedded bootstrapper for best-effort Windows 7 compatibility testing.

### Changed
- Bumped the desktop app and installer version to `0.3.1`.
- Enabled the notification plugin's `windows7-compat` feature for the experimental Windows 7 packaging path.

## [0.2.0] - 2026-04-16

### Added
- Added a tray-level `Print Calendar...` action that opens the Apple Calendar print dialog for the current Calendar window.

### Changed
- Bumped the desktop app and installer version to `0.2.0`.
- Localized the tray-menu control surface and window titles to Chinese.
- Updated project documentation and memory to track the new printing capability and release milestone.

## [0.1.0] - 2026-04-15

### Added
- Added independent Calendar, Reminders, and Notes windows.
- Added a shared persisted WebView2 profile across all content windows.
- Added tray support, single-instance behavior, and auto-start support.
- Added window geometry restore, hide-to-tray behavior, and remembered window visibility.
- Added per-window `Pin to Desktop Layer` and `Always On Top` controls.
- Added persistent local settings storage and legacy local-state migration into the `VibeCal` app identity.
- Added a reset script for clearing local session state and auto-start entries.

### Changed
- Switched the published app identity and packaging to `VibeCal`.
- Defaulted the wrapper to the China iCloud domain when no existing cookie domain is known.
