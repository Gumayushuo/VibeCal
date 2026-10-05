# Version 0.4.0 Verification

Date: 2026-10-05. Platform: Windows x64, Tauri 2.10.3, WebView2, Rust 1.94.1.

## Completed Checks
- `npm test`: five calendar-export regression tests passed.
- `cargo test --locked`: four Rust regression tests passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `git diff --check`: passed.
- `npm run build:local`: release executable and NSIS installer generated successfully.
- Added a release-asset verifier using the same Minisign verification implementation as Tauri's updater. The publish workflow now checks both Windows updater entries against the embedded public key before promoting a draft release to latest.
- The verifier accepted the original signed v0.3.0 installer and rejected a deliberately modified copy.
- GitHub Actions [run 37324524366](https://github.com/Gumayushuo/VibeCal/actions/runs/37324524366) succeeded. Both Windows updater entries passed installer signature/URL verification before [v0.4.0](https://github.com/Gumayushuo/VibeCal/releases/tag/v0.4.0) was published as latest.
- The live updater endpoint returns version 0.4.0. An isolated Tauri client reporting version 0.3.0 detected the new release and downloaded the 3,313,758-byte installer with the original public key signature verified. This exercised the real updater download path; installation was not started.
- Browser fixture verification: settings load; zoom/opacity/automatic-fit controls; pin/restore workspace actions; TXT dispatch; ICS-to-calendar-CSV save; export completion status; no horizontal overflow at a 460px viewport; no page-script errors.
- Cross-origin iCloud fixtures: loaded text from main and child frames collected without missing frames; password-frame content excluded from export.
- Isolated native Tauri/WebView2 verification used the actual production Rust controls module and bundled HTML. Local-origin command checks, preference writes, and settings IPC succeeded. Native WebView2 zoom read back as 0.5 for a 600x450 calendar fixture with automatic fit. Windows layered-window alpha read back as 179 for 70% opacity.

The isolated native check used identifier `com.vibecal.validation`, hidden local fixture windows, and no autostart, single-instance, updater, or cloud session. The existing running VibeCal process was left intact. A WebView2 class-unregister message was emitted during fixture teardown after the assertions passed; the validation process exited successfully.

## Export Coverage
- Unicode/folded ICS text and multiline descriptions.
- Inclusive CSV end dates for all-day events with exclusive ICS end dates.
- Recurrence expansion, EXDATE exclusions, cancellations, modified occurrences, and exceptions moved into the selected range.
- Embedded timezone conversion and explicit errors for missing timezone definitions.
- UTF-8 BOM, quoted CSV fields, and spreadsheet-formula neutralization.
- Empty/reversed date ranges and export limits.

## Remaining Integration Validation
- The actual signed-in iCloud calendar layout at the user's preferred small window dimensions. Automatic fit uses reference dimensions; Apple layout changes may require adjusting manual zoom.
- Actual reminder hyperlinks, Apple sign-in popups, and visual whole-window transparency on the user's desktop. URL-routing decisions and native alpha/zoom were verified; account-specific integration was not accessed.
- Actual Outlook import with the user's version and regional date settings. The original ICS is the preferred migration format for new/web Outlook and for preserving recurrence, alarms, and attachments.
- Actual update installation over an existing user's app, followed by signed-in session/geometry validation. Publication, live detection, download, and signature verification are complete; the user's running app was not replaced during testing.

Page-text backup only exports currently loaded rendered text. It does not enumerate an entire iCloud account or fetch unopened notes, hidden events, virtualized entries, or attachments. Calendar conversion needs an existing ICS file; it does not automatically publish or download private calendars.
