# Requirements

## Objective
- Build a personal Windows 11 desktop client that makes Apple Calendar feel like a native desktop app while still relying on Apple's web experience.

## Target User
- Single personal user.

## Product Direction
- Use a wrapper-style desktop client instead of rebuilding calendar functionality.
- Prefer Tauri + WebView2 for v1.
- Allow system-browser-assisted sign-in if that is more reliable than embedded login.

## v1 Scope
- Open directly into Apple Calendar from a desktop application.
- Keep the user signed in across launches when possible.
- Remember window geometry and window state.
- Provide tray integration.
- Support launching at system startup.
- Prevent multiple active instances.
- Support Windows native notifications.
- Fit normal Windows 11 usage patterns, including pinning to taskbar or desktop shortcuts.
- Support separate Calendar, Reminders, and Notes windows that can all stay open at the same time.
- Provide a tray-driven way to print the current Calendar view through the Apple web print flow.
- Automatically detect and install newer signed releases, while showing update notes to the user before installation.
- Keep regular Calendar, Reminders, and Notes windows fully movable and resizable by the user without enforced docking or ratio logic.
- Provide a persistent desktop-layer mode that can be toggled independently for each window.
- Provide a persistent always-on-top mode that can be toggled independently for each window.
- Default to opening Calendar only on a fresh setup.
- Remember which windows were visible or closed so the next launch restores the user's last workspace state exactly.
- Treat recurring items inside a dedicated `Habits` reminders list as the first implementation path for daily habits.
- Prefer the China iCloud domain by default when no existing cookie domain is detected.

## Constraints
- Windows 11 only.
- Personal-use quality bar, not public distribution quality.
- Generated repository content should remain in English.
- User-facing chat and explanations should remain in Chinese.
- External Apple login and web behavior may limit implementation details.
- Public-facing naming and branding should not imply affiliation with Apple.

## Risks And Unknowns
- Embedded Apple sign-in may be blocked or unstable.
- Session persistence behavior may differ across WebView and browser-assisted flows.
- Notification support may depend on how the wrapper integrates with Windows APIs and the web content.
- Browser-assisted login should not be assumed to hydrate the app's WebView session automatically.
- Apple data should not be assumed to be safely available to custom local widgets without additional integration work.
- A fake local dashboard shell may look polished but can still feel worse than separate real Apple pages if interaction fidelity drops.
- Desktop-layer behavior depends on Windows shell internals and may require adjustment across Windows updates.

## Definition Of Done
- On Windows 11, the user can launch the app and reach Apple Calendar with a durable signed-in experience as the primary target.
- Core desktop features for v1 are represented in the implementation plan and later delivered incrementally.
