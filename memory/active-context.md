# Active Context

## Current Phase
- Version 0.4.0 implements five user-feedback items: small-window calendar scaling, opacity, export, external links, and fewer tray clicks.

## Stable Architecture
- Windows 11 desktop wrapper built with Tauri 2 and WebView2.
- Calendar, Reminders, and Notes remain independent windows with a shared persisted session profile.
- China iCloud is the default fallback; saved cookie signals select the global domain when appropriate.
- Keep free movement/resizing in normal mode and the established fixed, chrome-free bottom-layer behavior when pinned.
- Preserve visibility, geometry, session persistence, autostart, single-instance launch, notifications, printing, and signed updater behavior.

## Current Changes
- Added a local settings window that stays open during consecutive changes, using bundled assets in controls/.
- Added saved per-window zoom (25% to 150%), optional resize-based automatic fit, and opacity (30% to 100%). Existing preferences remain fully opaque at 100% zoom.
- Added one-click pin-all and normal-all actions and synchronized native tray checkmarks.
- Added external navigation/popup routing to the default browser, retaining embedded iCloud and known Apple sign-in origins.
- Added native-save TXT/CSV exports of currently loaded page text. Unopened notes, unloaded virtualized entries, hidden events, and attachments are outside this backup scope.
- Added local ICS conversion through pinned ICAL.js. Handles recurrence/exclusions/modified occurrences, ranges, and embedded timezone definitions; missing definitions fail visibly.
- Calendar CSV flattens recurrence and uses local machine time. Original ICS remains the migration format for full recurrence/alarms/attachments.
- Added npm test, Rust regression checks, and npm run build:local for installer generation without updater signing keys.

## Validation And Delivery
- Rust preference/zoom/link/CSV regression tests and strict Clippy checks passed during development.
- Five calendar-conversion tests passed, covering Unicode, all-day end dates, recurrence exclusions/cancellations/modifications, timezones, formula handling, and moved exceptions.
- Browser fixtures passed settings interactions, batch modes, export dispatch/save, narrow-window layout, cross-origin frame collection, and password-page exclusion.
- Final Rust tests, strict Clippy, browser fixtures, and the Windows release installer build passed after the last code edits.
- Isolated native WebView2 validation confirmed actual settings IPC, local-origin checks, persisted preferences, 70% Win32 alpha (179), and automatic zoom (0.5 for a 600x450 fixture).
- Validation details and account-dependent limits are in VERIFICATION.md. No changes have been pushed to GitHub.

## Immediate Next Step
- Deliver the completed executable/installer with explicit export limits.
- User requested distribution through the existing updater. The original public key and application identifier match v0.3.0; the current live updater feed is still v0.3.0. GitHub repository access and both original signing secret names were verified without reading secret values.
- Prepare v0.4.0 through the existing GitHub signing pipeline, verify draft installer signatures against the embedded key, and only then publish it as the latest release.
- Validate the actual signed-in Apple calendar view, reminder links, and Windows opacity behavior with the user's account and preferred desktop dimensions. The existing running app has not been stopped or replaced.
